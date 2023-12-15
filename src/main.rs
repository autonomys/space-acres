#![windows_subsystem = "windows"]
#![feature(const_option, trait_alias, try_blocks)]

mod backend;
mod frontend;

use crate::backend::config::RawConfig;
use crate::backend::{BackendAction, BackendNotification};
use crate::frontend::configuration::{ConfigurationInput, ConfigurationOutput, ConfigurationView};
use crate::frontend::loading::{LoadingInput, LoadingView};
use crate::frontend::running::{RunningInput, RunningView};
use futures::channel::mpsc;
use futures::{select, FutureExt, SinkExt, StreamExt};
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::RELM_THREADS;
use std::future::Future;
use std::thread::available_parallelism;
use subspace_farmer::utils::{run_future_in_dedicated_thread, AsyncJoinOnDrop};
use subspace_proof_of_space::chia::ChiaTable;
use tracing::warn;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

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
    Warning(String),
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
            Self::Warning(_) => "warning-label",
            Self::Error(_) => "error-label",
        }
    }

    fn message(&self) -> &str {
        match self {
            Self::None => "",
            Self::Warning(message) | Self::Error(message) => message.as_str(),
        }
    }
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
    // Stored here so `Drop` is called on this future as well, preventing exit until everything shuts down gracefully
    _background_tasks: Box<dyn Future<Output = ()>>,
}

#[relm4::component(async)]
impl AsyncComponent for App {
    type Init = ();
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
                        #[watch]
                        set_visible: !model.status_bar_notification.is_none(),
                    },
                },
            }
        }
    }

    async fn init(
        _init: Self::Init,
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
            .comments(env!("CARGO_PKG_DESCRIPTION"))
            .logo(&gtk::gdk::Texture::for_pixbuf(
                &gtk::gdk_pixbuf::Pixbuf::from_read(ABOUT_IMAGE)
                    .expect("Statically correct image; qed"),
            ))
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
                    self.status_bar_notification = StatusBarNotification::Warning(
                        "Application restart is needed for configuration changes to take effect"
                            .to_string(),
                    );
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

fn main() {
    // TODO: Log into files
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

    // The default in `relm4` is `1`, set this back to Tokio's default
    RELM_THREADS
        .set(
            available_parallelism()
                .map(|cores| cores.get())
                .unwrap_or(1),
        )
        .expect("The first thing in the app, is not set; qed");

    let app = RelmApp::new("network.subspace.space_acres");

    app.set_global_css(GLOBAL_CSS);

    // Prefer dark theme in cross-platform way if environment is configured that way
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(!matches!(
            dark_light::detect(),
            dark_light::Mode::Light
        ));
    }

    app.run_async::<App>(());
}
