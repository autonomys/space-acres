#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(const_option, trait_alias, try_blocks)]

mod backend;
mod frontend;

use crate::backend::config::RawConfig;
use crate::backend::{BackendAction, BackendNotification};
use crate::frontend::configuration::{ConfigurationInput, ConfigurationOutput, ConfigurationView};
use crate::frontend::loading::{LoadingInput, LoadingView};
use crate::frontend::running::{RunningInput, RunningView};
use atomic::Atomic;
use clap::Parser;
use duct::cmd;
use file_rotate::compression::Compression;
use file_rotate::suffix::AppendCount;
use file_rotate::{ContentLimit, FileRotate};
use futures::channel::mpsc;
use futures::{select, FutureExt, SinkExt, StreamExt};
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::RELM_THREADS;
use std::future::Future;
use std::io::{Read, Write};
use std::process::{ExitCode, Termination};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread::available_parallelism;
use std::{env, fs, io, process};
use subspace_farmer::utils::{run_future_in_dedicated_thread, AsyncJoinOnDrop};
use subspace_proof_of_space::chia::ChiaTable;
use tracing::{info, warn};
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

/// Number of log files to keep
const LOG_FILE_LIMIT_COUNT: usize = 5;
/// Size of one log file
const LOG_FILE_LIMIT_SIZE: usize = 1024 * 1024 * 10;
const LOG_READ_BUFFER: usize = 1024 * 1024;

#[derive(Debug, Copy, Clone)]
enum AppStatusCode {
    Exit,
    Restart,
    Unknown(i32),
}

impl AppStatusCode {
    fn from_status_code(status_code: i32) -> Self {
        match status_code {
            0 => Self::Exit,
            100 => Self::Restart,
            code => Self::Unknown(code),
        }
    }

    fn into_status_code(self) -> i32 {
        match self {
            AppStatusCode::Exit => 0,
            AppStatusCode::Restart => 100,
            AppStatusCode::Unknown(code) => code,
        }
    }
}

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const GLOBAL_CSS: &str = include_str!("../res/app.css");
const ABOUT_IMAGE: &[u8] = include_bytes!("../res/about.png");

type PosTable = ChiaTable;

#[derive(Debug)]
enum AppInput {
    BackendNotification(BackendNotification),
    Configuration(ConfigurationOutput),
    OpenReconfiguration,
    ShowAboutDialog,
    Restart,
}

enum View {
    Loading,
    Configuration,
    Reconfiguration,
    Running,
    Stopped(Option<anyhow::Error>),
    Error(anyhow::Error),
}

impl View {
    fn title(&self) -> &'static str {
        match self {
            Self::Loading => "Loading",
            Self::Configuration => "Configuration",
            Self::Reconfiguration => "Reconfiguration",
            Self::Running => "Running",
            Self::Stopped(_) => "Stopped",
            Self::Error(_) => "Error",
        }
    }
}

#[derive(Debug, Default)]
enum StatusBarNotification {
    #[default]
    None,
    Warning {
        message: String,
        /// Whether to show restart button
        restart: bool,
    },
    Error(String),
}

impl StatusBarNotification {
    fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    fn css_classes() -> &'static [&'static str] {
        &["label", "warning-label", "error-label"]
    }

    fn css_class(&self) -> &'static str {
        match self {
            Self::None => "label",
            Self::Warning { .. } => "warning-label",
            Self::Error(_) => "error-label",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::None => "",
            Self::Warning { message, .. } | Self::Error(message) => message.as_str(),
        }
    }

    fn restart_button(&self) -> bool {
        match self {
            Self::Warning { restart, .. } => *restart,
            _ => false,
        }
    }
}

struct AppInit {
    exit_status_code: Arc<Atomic<AppStatusCode>>,
    minimize_on_start: bool,
}

// TODO: Efficient updates with tracker
struct App {
    current_view: View,
    current_raw_config: Option<RawConfig>,
    status_bar_notification: StatusBarNotification,
    backend_action_sender: mpsc::Sender<BackendAction>,
    loading_view: Controller<LoadingView>,
    configuration_view: Controller<ConfigurationView>,
    running_view: Controller<RunningView>,
    menu_popover: gtk::Popover,
    about_dialog: gtk::AboutDialog,
    exit_status_code: Arc<Atomic<AppStatusCode>>,
    // Stored here so `Drop` is called on this future as well, preventing exit until everything shuts down gracefully
    _background_tasks: Box<dyn Future<Output = ()>>,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = AppInit;
    type Input = AppInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        gtk::Window {
            set_decorated: false,
            set_resizable: false,
            set_size_request: (800, 600),
            #[watch]
            set_title: Some(&format!("{} - Space Acres {}", model.current_view.title(), env!("CARGO_PKG_VERSION"))),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::HeaderBar {
                    pack_end = &gtk::MenuButton {
                        set_direction: gtk::ArrowType::None,
                        set_icon_name: "open-menu-symbolic",
                        #[wrap(Some)]
                        set_popover: menu_popover = &gtk::Popover {
                            set_halign: gtk::Align::End,
                            set_position: gtk::PositionType::Bottom,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 5,

                                gtk::Button {
                                    connect_clicked => AppInput::OpenReconfiguration,
                                    set_label: "Update configuration",
                                    #[watch]
                                    set_visible: model.current_raw_config.is_some(),
                                },

                                gtk::Button {
                                    connect_clicked => AppInput::ShowAboutDialog,
                                    set_label: "About",
                                },
                            },
                        },
                    },
                },

                gtk::Box {
                    set_margin_all: 10,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    #[transition = "SlideLeftRight"]
                    match &model.current_view {
                        View::Loading => model.loading_view.widget().clone(),
                        View::Configuration | View::Reconfiguration => model.configuration_view.widget().clone(),
                        View::Running=> model.running_view.widget().clone(),
                        View::Stopped(Some(error)) => {
                            // TODO: Better error handling
                            gtk::Label {
                                #[watch]
                                set_label: &format!("Stopped with error: {error}"),
                            }
                        }
                        View::Stopped(None) => {
                            gtk::Label {
                                set_label: "Stopped 🛑",
                            }
                        }
                        View::Error(error) => {
                            // TODO: Better error handling
                            gtk::Label {
                                #[watch]
                                set_label: &format!("Error: {error}"),
                            }
                        },
                    },

                    gtk::Box {
                        set_halign: gtk::Align::Center,
                        set_spacing: 10,
                        #[watch]
                        set_visible: !model.status_bar_notification.is_none(),

                        #[name = "status_bar_notification_label"]
                        gtk::Label {
                            #[track = "!status_bar_notification_label.has_css_class(model.status_bar_notification.css_class())"]
                            add_css_class: {
                                for css_class in StatusBarNotification::css_classes() {
                                    status_bar_notification_label.remove_css_class(css_class);
                                }

                                model.status_bar_notification.css_class()
                            },
                            #[watch]
                            set_label: model.status_bar_notification.message(),
                        },

                        gtk::Button {
                            add_css_class: "suggested-action",
                            connect_clicked => AppInput::Restart,
                            set_label: "Restart",
                            #[watch]
                            set_visible: model.status_bar_notification.restart_button(),
                        },
                    },
                },
            }
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let (backend_action_sender, backend_action_receiver) = mpsc::channel(1);
        let (backend_notification_sender, mut backend_notification_receiver) = mpsc::channel(100);

        // Create and run backend in dedicated thread
        let backend_fut = run_future_in_dedicated_thread(
            move || backend::create(backend_action_receiver, backend_notification_sender),
            "backend".to_string(),
        )
        .expect("Must be able to spawn a thread");

        // Forward backend notifications as application inputs
        let message_forwarder_fut = AsyncJoinOnDrop::new(
            tokio::spawn({
                let sender = sender.clone();

                async move {
                    while let Some(notification) = backend_notification_receiver.next().await {
                        // TODO: This panics on shutdown because component is already shut down, this should be handled
                        //  more gracefully
                        sender.input(AppInput::BackendNotification(notification));
                    }
                }
            }),
            true,
        );

        let loading_view = LoadingView::builder().launch(()).detach();

        let configuration_view = ConfigurationView::builder()
            .launch(root.clone())
            .forward(sender.input_sender(), AppInput::Configuration);

        let running_view = RunningView::builder().launch(()).detach();

        let about_dialog = gtk::AboutDialog::builder()
            .title("About")
            .program_name("Space Acres")
            .version(env!("CARGO_PKG_VERSION"))
            .authors(env!("CARGO_PKG_AUTHORS").split(':').collect::<Vec<_>>())
            // TODO: Use https://gitlab.gnome.org/GNOME/gtk/-/merge_requests/6643 once available
            .license("Zero-Clause BSD: https://opensource.org/license/0bsd/")
            .website(env!("CARGO_PKG_REPOSITORY"))
            .website_label("GitHub")
            .comments(env!("CARGO_PKG_DESCRIPTION"))
            .logo(&gtk::gdk::Texture::for_pixbuf(
                &gtk::gdk_pixbuf::Pixbuf::from_read(ABOUT_IMAGE)
                    .expect("Statically correct image; qed"),
            ))
            .system_information({
                let config_directory = dirs::config_local_dir()
                    .map(|config_local_dir| {
                        config_local_dir
                            .join(env!("CARGO_PKG_NAME"))
                            .display()
                            .to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());
                let data_directory = dirs::data_local_dir()
                    .map(|data_local_dir| {
                        data_local_dir
                            .join(env!("CARGO_PKG_NAME"))
                            .display()
                            .to_string()
                    })
                    .unwrap_or_else(|| "Unknown".to_string());

                format!(
                    "Config directory: {config_directory}\n\
                    Data directory (including logs): {data_directory}",
                )
            })
            .transient_for(&root)
            .build();
        about_dialog.connect_close_request(|about_dialog| {
            about_dialog.hide();
            gtk::glib::Propagation::Stop
        });

        let mut model = Self {
            current_view: View::Loading,
            current_raw_config: None,
            status_bar_notification: StatusBarNotification::None,
            backend_action_sender,
            loading_view,
            configuration_view,
            running_view,
            // Hack to initialize a field before this data structure is used
            menu_popover: gtk::Popover::default(),
            about_dialog,
            exit_status_code: init.exit_status_code,
            _background_tasks: Box::new(async move {
                // Order is important here, if backend is dropped first, there will be an annoying panic in logs due to
                // notification forwarder sending notification to the component that is already shut down
                select! {
                    _ = message_forwarder_fut.fuse() => {
                        warn!("Message forwarder exited");
                    }
                    _ = backend_fut.fuse() => {
                        warn!("Backend exited");
                    }
                }
            }),
        };

        let widgets = view_output!();

        model.menu_popover = widgets.menu_popover.clone();

        if init.minimize_on_start {
            root.minimize();
        }

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        input: Self::Input,
        _sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match input {
            AppInput::BackendNotification(notification) => {
                self.process_backend_notification(notification);
            }
            AppInput::Configuration(configuration_output) => {
                self.process_configuration_output(configuration_output)
                    .await;
            }
            AppInput::OpenReconfiguration => {
                self.menu_popover.hide();
                if let Some(raw_config) = self.current_raw_config.clone() {
                    self.configuration_view
                        .emit(ConfigurationInput::Reconfigure(raw_config));
                    self.current_view = View::Reconfiguration;
                }
            }
            AppInput::ShowAboutDialog => {
                self.menu_popover.hide();
                self.about_dialog.show();
            }
            AppInput::Restart => {
                self.exit_status_code
                    .store(AppStatusCode::Restart, Ordering::Release);
                relm4::main_application().quit();
            }
        }
    }
}

impl App {
    fn process_backend_notification(&mut self, notification: BackendNotification) {
        match notification {
            // TODO: Render progress
            BackendNotification::Loading { step, progress: _ } => {
                self.current_view = View::Loading;
                self.status_bar_notification = StatusBarNotification::None;
                self.loading_view.emit(LoadingInput::BackendLoading(step));
            }
            BackendNotification::NotConfigured => {
                // TODO: Welcome screen first
                self.current_view = View::Configuration;
            }
            BackendNotification::ConfigurationIsInvalid { error, .. } => {
                self.status_bar_notification =
                    StatusBarNotification::Error(format!("Configuration is invalid: {error}"));
            }
            BackendNotification::ConfigSaveResult(result) => match result {
                Ok(()) => {
                    self.status_bar_notification = StatusBarNotification::Warning {
                        message:
                            "Application restart is needed for configuration changes to take effect"
                                .to_string(),
                        restart: true,
                    };
                }
                Err(error) => {
                    self.status_bar_notification = StatusBarNotification::Error(format!(
                        "Failed to save configuration changes: {error}"
                    ));
                }
            },
            BackendNotification::Running {
                config: _,
                raw_config,
                best_block_number,
                initial_plotting_states,
            } => {
                self.current_raw_config.replace(raw_config);
                self.current_view = View::Running;
                self.running_view.emit(RunningInput::Initialize {
                    best_block_number,
                    initial_plotting_states,
                });
            }
            BackendNotification::Node(node_notification) => {
                self.running_view
                    .emit(RunningInput::NodeNotification(node_notification));
            }
            BackendNotification::Farmer(farmer_notification) => {
                self.running_view
                    .emit(RunningInput::FarmerNotification(farmer_notification));
            }
            BackendNotification::Stopped { error } => {
                self.current_view = View::Stopped(error);
            }
            BackendNotification::IrrecoverableError { error } => {
                self.current_view = View::Error(error);
            }
        }
    }

    async fn process_configuration_output(&mut self, configuration_output: ConfigurationOutput) {
        match configuration_output {
            ConfigurationOutput::StartWithNewConfig(raw_config) => {
                if let Err(error) = self
                    .backend_action_sender
                    .send(BackendAction::NewConfig { raw_config })
                    .await
                {
                    self.current_view =
                        View::Error(anyhow::anyhow!("Failed to send config to backend: {error}"));
                }
            }
            ConfigurationOutput::ConfigUpdate(raw_config) => {
                self.current_raw_config.replace(raw_config.clone());
                // Config is updated when application is already running, switch to corresponding screen
                self.current_view = View::Running;
                if let Err(error) = self
                    .backend_action_sender
                    .send(BackendAction::NewConfig { raw_config })
                    .await
                {
                    self.current_view =
                        View::Error(anyhow::anyhow!("Failed to send config to backend: {error}"));
                }
            }
            ConfigurationOutput::Close => {
                // Configuration view is closed when application is already running, switch to corresponding screen
                self.current_view = View::Running;
            }
        }
    }
}

#[derive(Debug, Parser)]
#[clap(about, version)]
struct Cli {
    /// Used for startup to minimize the window
    #[arg(long)]
    startup: bool,
    /// Used by child process such that supervisor parent process can control it
    #[arg(long)]
    child_process: bool,
    /// The rest of the arguments that will be sent to GTK4 as is
    #[arg(raw = true)]
    gtk_arguments: Vec<String>,
}

impl Cli {
    fn run(self) -> ExitCode {
        if self.child_process {
            ExitCode::from(self.app().into_status_code() as u8)
        } else {
            self.supervisor().report()
        }
    }

    fn app(self) -> AppStatusCode {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::fmt::layer()
                    // TODO: Workaround for https://github.com/tokio-rs/tracing/issues/2214, also on
                    //  Windows terminal doesn't support the same colors as bash does
                    .with_ansi(if cfg!(windows) {
                        false
                    } else {
                        supports_color::on(supports_color::Stream::Stderr).is_some()
                    })
                    .with_filter(
                        EnvFilter::builder()
                            .with_default_directive(LevelFilter::INFO.into())
                            .from_env_lossy(),
                    ),
            )
            .init();

        info!(
            "Starting {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );

        // The default in `relm4` is `1`, set this back to Tokio's default
        RELM_THREADS
            .set(
                available_parallelism()
                    .map(|cores| cores.get())
                    .unwrap_or(1),
            )
            .expect("The first thing in the app, is not set; qed");

        let app = RelmApp::new("network.subspace.space_acres");
        let app = app.with_args({
            let mut args = self.gtk_arguments;
            // Application itself is expected as the first argument
            args.insert(0, env::args().next().expect("Guaranteed to exist; qed"));
            args
        });

        app.set_global_css(GLOBAL_CSS);

        // Prefer dark theme in cross-platform way if environment is configured that way
        if let Some(settings) = gtk::Settings::default() {
            settings.set_gtk_application_prefer_dark_theme(!matches!(
                dark_light::detect(),
                dark_light::Mode::Light
            ));
        }

        let exit_status_code = Arc::new(Atomic::new(AppStatusCode::Exit));

        app.run_async::<App>(AppInit {
            exit_status_code: Arc::clone(&exit_status_code),
            minimize_on_start: self.startup,
        });

        let exit_status_code = exit_status_code.load(Ordering::Acquire);
        info!(
            ?exit_status_code,
            "Exiting {} {}",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION")
        );
        exit_status_code
    }

    fn supervisor(mut self) -> io::Result<()> {
        let maybe_app_data_dir = dirs::data_local_dir()
            .map(|data_local_dir| data_local_dir.join(env!("CARGO_PKG_NAME")))
            .and_then(|app_data_dir| {
                if !app_data_dir.exists() {
                    if let Err(error) = fs::create_dir_all(&app_data_dir) {
                        eprintln!(
                            "App data directory \"{}\" doesn't exist and can't be created: {}",
                            app_data_dir.display(),
                            error
                        );
                        return None;
                    }
                }

                Some(app_data_dir)
            });
        let mut maybe_logger = maybe_app_data_dir.as_ref().map(|app_data_dir| {
            FileRotate::new(
                app_data_dir.join("space-acres.log"),
                AppendCount::new(LOG_FILE_LIMIT_COUNT),
                ContentLimit::Bytes(LOG_FILE_LIMIT_SIZE),
                Compression::OnRotate(0),
                #[cfg(unix)]
                None,
            )
        });

        let mut log_read_buffer = vec![0u8; LOG_READ_BUFFER];

        loop {
            let mut args = vec!["--child-process".to_string()];
            if self.startup {
                // In case of restart we no longer want to minimize the app
                self.startup = false;

                args.push("--startup".to_string());
            }
            args.push("--".to_string());
            args.extend_from_slice(&self.gtk_arguments);

            let expression = cmd(env::current_exe()?, args)
                .stderr_to_stdout()
                // We use non-zero status codes and they don't mean error necessarily
                .unchecked();

            // if startup {
            //     expression.
            // }

            let exit_status = if let Some(logger) = maybe_logger.as_mut() {
                let mut expression = expression.reader()?;

                let mut stdout = io::stdout();
                loop {
                    match expression.read(&mut log_read_buffer) {
                        Ok(bytes_count) => {
                            if bytes_count == 0 {
                                break;
                            }

                            let write_result: io::Result<()> = try {
                                stdout.write_all(&log_read_buffer[..bytes_count])?;
                                logger.write_all(&log_read_buffer[..bytes_count])?;
                            };

                            if let Err(error) = write_result {
                                eprintln!("Error while writing output of child process: {error}");
                                break;
                            }
                        }
                        Err(error) => {
                            if error.kind() == io::ErrorKind::Interrupted {
                                // Try again
                                continue;
                            }
                            eprintln!("Error while reading output of child process: {error}");
                            break;
                        }
                    }
                }

                stdout.flush()?;
                if let Err(error) = logger.flush() {
                    eprintln!("Error while flushing logs: {error}");
                }

                match expression.try_wait()? {
                    Some(output) => output.status,
                    None => {
                        return Err(io::Error::new(
                            io::ErrorKind::Other,
                            "Logs writing ended before child process did, exiting",
                        ));
                    }
                }
            } else {
                eprintln!("App data directory doesn't exist, not creating log file");
                expression.run()?.status
            };

            match exit_status.code() {
                Some(status_code) => match AppStatusCode::from_status_code(status_code) {
                    AppStatusCode::Exit => {
                        eprintln!("Application exited gracefully");
                        break;
                    }
                    AppStatusCode::Restart => {
                        eprintln!("Restarting application");
                        continue;
                    }
                    AppStatusCode::Unknown(status_code) => {
                        eprintln!("Application exited with unexpected status code {status_code}");
                        process::exit(status_code);
                    }
                },
                None => {
                    eprintln!("Application terminated by signal");
                    break;
                }
            }
        }

        Ok(())
    }
}

fn main() -> ExitCode {
    Cli::parse().run()
}
