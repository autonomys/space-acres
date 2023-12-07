#![windows_subsystem = "windows"]
#![feature(const_option, trait_alias, try_blocks)]

mod backend;
mod frontend;

use crate::backend::{BackendAction, BackendNotification, LoadingStep};
use crate::frontend::configuration::{ConfigurationOutput, ConfigurationView};
use crate::frontend::running::{RunningInput, RunningView};
use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{set_global_css, RELM_THREADS};
use std::thread::available_parallelism;
use subspace_proof_of_space::chia::ChiaTable;
use tokio::runtime::Handle;
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
    ShowAboutDialog,
}

enum View {
    Loading(String),
    Configuration,
    Running,
    Stopped(Option<anyhow::Error>),
    Error(anyhow::Error),
}

impl View {
    fn title(&self) -> &'static str {
        match self {
            Self::Loading(_) => "Loading",
            Self::Configuration => "Configuration",
            Self::Running => "Running",
            Self::Stopped(_) => "Stopped",
            Self::Error(_) => "Error",
        }
    }
}

// TODO: Efficient updates with tracker
struct App {
    current_view: View,
    backend_action_sender: mpsc::Sender<BackendAction>,
    configuration_view: AsyncController<ConfigurationView>,
    running_view: AsyncController<RunningView>,
    menu_popover: gtk::Popover,
    about_dialog: gtk::AboutDialog,
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
                                    set_label: "About",
                                    connect_clicked[sender] => move |_| {
                                        sender.input(AppInput::ShowAboutDialog);
                                    },
                                },
                            },
                        },
                    },
                },

                gtk::Box {
                    set_margin_all: 10,
                    set_orientation: gtk::Orientation::Vertical,

                    #[transition = "SlideLeftRight"]
                    match &model.current_view {
                        View::Loading(what) => gtk::Box {
                            set_halign: gtk::Align::Center,
                            set_valign: gtk::Align::Center,
                            set_vexpand: true,
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Spinner {
                                start: (),
                                set_size_request: (50, 50),
                            },

                            gtk::Label {
                                #[watch]
                                set_label: what,
                            },
                        },
                        View::Configuration => model.configuration_view.widget().clone(),
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
                },
            }
        }
    }

    async fn init(
        _init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let (action_sender, action_receiver) = mpsc::channel(1);
        let (notification_sender, mut notification_receiver) = mpsc::channel(100);

        // Create backend in dedicated thread
        tokio::task::spawn_blocking(move || {
            Handle::current().block_on(backend::create(action_receiver, notification_sender));
        });

        // Forward backend notifications as application inputs
        tokio::spawn({
            let sender = sender.clone();

            async move {
                while let Some(notification) = notification_receiver.next().await {
                    sender.input(AppInput::BackendNotification(notification));
                }
            }
        });

        let configuration_view = ConfigurationView::builder()
            .launch(())
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
            current_view: View::Loading(String::new()),
            backend_action_sender: action_sender,
            configuration_view,
            running_view,
            // Hack to initialize a field before this data structure is used
            menu_popover: gtk::Popover::default(),
            about_dialog,
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
                self.set_loading_string(match step {
                    LoadingStep::LoadingConfiguration => "Loading configuration...",
                    LoadingStep::ReadingConfiguration => "Reading configuration...",
                    LoadingStep::ConfigurationReadSuccessfully { .. } => {
                        "Configuration read successfully"
                    }
                    LoadingStep::CheckingConfiguration => "Checking configuration...",
                    LoadingStep::ConfigurationIsValid => "Configuration is valid",
                    LoadingStep::DecodingChainSpecification => "Decoding chain specification...",
                    LoadingStep::DecodedChainSpecificationSuccessfully => {
                        "Decoded chain specification successfully"
                    }
                    LoadingStep::CheckingNodePath => "Checking node path...",
                    LoadingStep::CreatingNodePath => "Creating node path...",
                    LoadingStep::NodePathReady => "Node path ready",
                    LoadingStep::PreparingNetworkingStack => "Preparing networking stack...",
                    LoadingStep::ReadingNetworkKeypair => "Reading network keypair...",
                    LoadingStep::GeneratingNetworkKeypair => "Generating network keypair...",
                    LoadingStep::WritingNetworkKeypair => "Writing network keypair to disk...",
                    LoadingStep::InstantiatingNetworkingStack => {
                        "Instantiating networking stack..."
                    }
                    LoadingStep::NetworkingStackCreatedSuccessfully => {
                        "Networking stack created successfully"
                    }
                    LoadingStep::CreatingConsensusNode => "Creating consensus node...",
                    LoadingStep::ConsensusNodeCreatedSuccessfully => {
                        "Consensus node created successfully"
                    }
                    LoadingStep::CreatingFarmer => "Creating farmer...",
                    LoadingStep::FarmerCreatedSuccessfully => "Farmer created successfully",
                });
            }
            BackendNotification::NotConfigured => {
                // TODO: Welcome screen first
                self.current_view = View::Configuration;
            }
            BackendNotification::ConfigurationIsInvalid { .. } => {
                // TODO: Toast with configuration error, render old values with corresponding validity status once
                //  notification has that information
                self.current_view = View::Configuration;
            }
            BackendNotification::Running {
                config,
                best_block_number,
            } => {
                let num_farms = config.farms.len();
                self.current_view = View::Running;
                self.running_view.emit(RunningInput::Initialize {
                    best_block_number,
                    num_farms,
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

    fn set_loading_string(&mut self, s: &'static str) {
        self.current_view = View::Loading(s.to_string());
    }

    async fn process_configuration_output(&mut self, configuration_output: ConfigurationOutput) {
        match configuration_output {
            ConfigurationOutput::StartWithNewConfig(config) => {
                if let Err(error) = self
                    .backend_action_sender
                    .send(BackendAction::NewConfig { config })
                    .await
                {
                    self.current_view =
                        View::Error(anyhow::anyhow!("Failed to send config to backend: {error}"));
                }
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

    set_global_css(GLOBAL_CSS);

    // Prefer dark theme in cross-platform way if environment is configured that way
    if let Some(settings) = gtk::Settings::default() {
        settings.set_gtk_application_prefer_dark_theme(!matches!(
            dark_light::detect(),
            dark_light::Mode::Light
        ));
    }

    app.run_async::<App>(());
}
