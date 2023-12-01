#![feature(const_option)]

mod backend;

use crate::backend::{BackendNotification, LoadingStep};
use futures::channel::mpsc;
use futures::StreamExt;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::RELM_THREADS;
use std::thread::available_parallelism;
use subspace_proof_of_space::chia::ChiaTable;
use tokio::runtime::Handle;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

type PosTable = ChiaTable;

#[derive(Debug)]
enum AppInput {
    BackendNotification(BackendNotification),
}

enum View {
    Loading(String),
    Running,
    Stopped(Option<anyhow::Error>),
    Error(anyhow::Error),
}

impl View {
    fn title(&self) -> &'static str {
        match self {
            Self::Loading(_) => "Loading",
            Self::Running => "Running",
            Self::Stopped(_) => "Stopped",
            Self::Error(_) => "Error",
        }
    }
}

struct App {
    current_view: View,
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
            set_size_request: (500, 500),
            #[watch]
            set_title: Some(&format!("{} - Space Acres {}", model.current_view.title(), env!("CARGO_PKG_VERSION"))),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::HeaderBar {
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

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
                        View::Running => {
                            gtk::Label {
                                set_label: "Running 🎉",
                            }
                        }
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
                    }
                }
            }
        }
    }

    async fn init(
        _init: (),
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let model = App {
            current_view: View::Loading("".to_string()),
        };
        let widgets = view_output!();

        let (notification_sender, mut notification_receiver) = mpsc::channel(1);

        // Create backend in dedicated thread
        tokio::task::spawn_blocking(move || {
            Handle::current().block_on(backend::create(notification_sender));
        });

        // Forward backend notifications as application inputs
        tokio::spawn(async move {
            while let Some(notification) = notification_receiver.next().await {
                sender.input(AppInput::BackendNotification(notification));
            }
        });

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
                // TODO: Configuration step
                self.current_view = View::Error(anyhow::anyhow!("Not configured"));
            }
            BackendNotification::ConfigurationIsInvalid { .. } => {
                // TODO: Configuration step
                self.current_view = View::Error(anyhow::anyhow!("Configuration is invalid"));
            }
            BackendNotification::Running => {
                self.current_view = View::Running;
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
    app.run_async::<App>(());
}
