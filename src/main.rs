#![feature(const_option, trait_alias)]

mod backend;

use crate::backend::farmer::{PlottingKind, PlottingState};
use crate::backend::node::{SyncKind, SyncState};
use crate::backend::{BackendNotification, FarmerNotification, LoadingStep, NodeNotification};
use futures::channel::mpsc;
use futures::StreamExt;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4::{set_global_css, RELM_THREADS};
use std::thread::available_parallelism;
use subspace_core_primitives::BlockNumber;
use subspace_proof_of_space::chia::ChiaTable;
use tokio::runtime::Handle;
use tracing::warn;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

const GLOBAL_CSS: &str = include_str!("../res/app.css");

type PosTable = ChiaTable;

#[derive(Debug)]
enum AppInput {
    BackendNotification(BackendNotification),
}

#[derive(Debug, Default)]
struct NodeState {
    best_block_number: BlockNumber,
    sync_state: Option<SyncState>,
}

#[derive(Debug, Default)]
struct FarmerState {
    /// One entry per farm
    plotting_state: Vec<Option<PlottingState>>,
}

enum View {
    Loading(String),
    Running {
        node_state: NodeState,
        farmer_state: FarmerState,
    },
    Stopped(Option<anyhow::Error>),
    Error(anyhow::Error),
}

impl View {
    fn title(&self) -> &'static str {
        match self {
            Self::Loading(_) => "Loading",
            Self::Running { .. } => "Running",
            Self::Stopped(_) => "Stopped",
            Self::Error(_) => "Error",
        }
    }
}

// TODO: Efficient updates with tracker
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
            set_size_request: (800, 600),
            #[watch]
            set_title: Some(&format!("{} - Space Acres {}", model.current_view.title(), env!("CARGO_PKG_VERSION"))),

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                gtk::HeaderBar {
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
                        View::Running { node_state, farmer_state } => gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,

                            gtk::Box {
                                set_height_request: 100,
                                set_orientation: gtk::Orientation::Vertical,

                                gtk::Label {
                                    add_css_class: "heading",
                                    set_halign: gtk::Align::Start,
                                    set_label: "Consensus node",
                                    set_margin_bottom: 10,
                                },

                                // Match only because `if let Some(x) = y` is not yet supported here
                                #[transition = "SlideUpDown"]
                                match &node_state.sync_state {
                                    Some(sync_state) => gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,

                                        gtk::Box {
                                            gtk::Label {
                                                set_halign: gtk::Align::Start,

                                                #[watch]
                                                set_label: &{
                                                    let kind = match sync_state.kind {
                                                        SyncKind::Dsn => "Syncing from DSN",
                                                        SyncKind::Regular => "Regular sync",
                                                    };

                                                    format!(
                                                        "{} #{}/{}{}",
                                                        kind,
                                                        node_state.best_block_number,
                                                        sync_state.target,
                                                        sync_state.speed
                                                            .map(|speed| format!(", {:.2} blocks/s", speed))
                                                            .unwrap_or_default(),
                                                    )
                                                },
                                            },

                                            gtk::Spinner {
                                                set_margin_start: 5,
                                                start: (),
                                            },
                                        },

                                        gtk::ProgressBar {
                                            #[watch]
                                            set_fraction: node_state.best_block_number as f64 / sync_state.target as f64,
                                            set_margin_bottom: 10,
                                            set_margin_top: 10,
                                        },
                                    },
                                    None => gtk::Box {
                                        gtk::Label {
                                            #[watch]
                                            set_label: &format!("Synced, best block #{}", node_state.best_block_number),
                                        }
                                    },
                                },
                            },

                            gtk::Separator {
                                set_margin_all: 10,
                            },

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_valign: gtk::Align::Start,

                                gtk::Label {
                                    add_css_class: "heading",
                                    set_halign: gtk::Align::Start,
                                    set_label: "Farmer",
                                    set_margin_bottom: 10,
                                },

                                // TODO: Render all farms, not just the first one
                                // Match only because `if let Some(x) = y` is not yet supported here
                                #[transition = "SlideUpDown"]
                                match (&farmer_state.plotting_state[0], node_state.sync_state.is_none()) {
                                    (Some(plotting_state), true) => gtk::Box {
                                        set_orientation: gtk::Orientation::Vertical,

                                        gtk::Box {
                                            gtk::Label {
                                                set_halign: gtk::Align::Start,

                                                #[watch]
                                                set_label: &{
                                                    let kind = match plotting_state.kind {
                                                        PlottingKind::Initial => "Initial plotting, not farming",
                                                        PlottingKind::Replotting => "Replotting, farming",
                                                    };

                                                    format!(
                                                        "{} {:.2}%{}",
                                                        kind,
                                                        plotting_state.progress,
                                                        plotting_state.speed
                                                            .map(|speed| format!(", {:.2} sectors/h", 3600.0 / speed))
                                                            .unwrap_or_default(),
                                                    )
                                                },
                                                set_tooltip: "Farming starts after initial plotting is complete",
                                            },

                                            gtk::Spinner {
                                                set_margin_start: 5,
                                                start: (),
                                            },
                                        },

                                        gtk::ProgressBar {
                                            #[watch]
                                            set_fraction: plotting_state.progress as f64 / 100.0,
                                            set_margin_bottom: 10,
                                            set_margin_top: 10,
                                        },
                                    },
                                    (None, true) => gtk::Box {
                                        gtk::Label {
                                            #[watch]
                                            set_label: "Farming",
                                        }
                                    },
                                    _ => gtk::Box {
                                        gtk::Label {
                                            #[watch]
                                            set_label: "Waiting for node to sync first",
                                        }
                                    },
                                },
                            },
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
                    },
                },
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

        let (notification_sender, mut notification_receiver) = mpsc::channel(100);

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
            BackendNotification::Running {
                config,
                best_block_number,
            } => {
                let num_farms = config.farms.len();
                self.current_view = View::Running {
                    node_state: NodeState {
                        best_block_number,
                        sync_state: None,
                    },
                    farmer_state: FarmerState {
                        plotting_state: vec![None; num_farms],
                    },
                };
            }
            BackendNotification::Node(node_notification) => {
                if let View::Running { node_state, .. } = &mut self.current_view {
                    match node_notification {
                        NodeNotification::Syncing(sync_state) => {
                            node_state.sync_state = Some(sync_state);
                        }
                        NodeNotification::Synced => {
                            node_state.sync_state = None;
                        }
                        NodeNotification::BlockImported { number } => {
                            node_state.best_block_number = number;
                        }
                    }
                }
            }
            BackendNotification::Farmer(farmer_notification) => {
                if let View::Running { farmer_state, .. } = &mut self.current_view {
                    match farmer_notification {
                        FarmerNotification::Plotting { farm_index, state } => {
                            if let Some(plotting_state) =
                                farmer_state.plotting_state.get_mut(farm_index)
                            {
                                plotting_state.replace(state);
                            } else {
                                warn!(%farm_index, "Unexpected plotting farm index");
                            }
                        }
                        FarmerNotification::Plotted { farm_index } => {
                            if let Some(plotting_state) =
                                farmer_state.plotting_state.get_mut(farm_index)
                            {
                                plotting_state.take();
                            } else {
                                warn!(%farm_index, "Unexpected plotted farm index");
                            }
                        }
                    }
                }
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
    set_global_css(GLOBAL_CSS);
    app.run_async::<App>(());
}
