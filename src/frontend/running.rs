use crate::backend::farmer::{PlottingKind, PlottingState};
use crate::backend::node::{SyncKind, SyncState};
use crate::backend::{FarmerNotification, NodeNotification};
use gtk::prelude::*;
use relm4::prelude::*;
use subspace_core_primitives::BlockNumber;
use tracing::warn;

#[derive(Debug)]
pub enum RunningInput {
    Initialize {
        best_block_number: BlockNumber,
        num_farms: usize,
    },
    NodeNotification(NodeNotification),
    FarmerNotification(FarmerNotification),
}

#[derive(Debug, Default)]
struct NodeState {
    best_block_number: BlockNumber,
    sync_state: SyncState,
}

#[derive(Debug, Default)]
struct FarmerState {
    /// One entry per farm
    plotting_state: Vec<PlottingState>,
}

#[derive(Debug)]
pub struct RunningView {
    node_state: NodeState,
    farmer_state: FarmerState,
}

#[relm4::component(pub)]
impl Component for RunningView {
    type Init = ();
    type Input = RunningInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::Box {
                set_height_request: 100,
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                gtk::Label {
                    add_css_class: "heading",
                    set_halign: gtk::Align::Start,
                    set_label: "Consensus node",
                },

                // TODO: Match only because `if let Some(x) = y` is not yet supported here: https://github.com/Relm4/Relm4/issues/582
                #[transition = "SlideUpDown"]
                match model.node_state.sync_state {
                    SyncState::Unknown => gtk::Box {
                        gtk::Label {
                            #[watch]
                            set_label: &format!(
                                "Connecting to the network, best block #{}",
                                model.node_state.best_block_number
                            ),
                        }
                    },
                    SyncState::Syncing { kind, target, speed } => gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,

                        gtk::Box {
                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                #[watch]
                                set_label: &{
                                    let kind = match kind {
                                        SyncKind::Dsn => "Syncing from DSN",
                                        SyncKind::Regular => "Regular sync",
                                    };

                                    format!(
                                        "{} #{}/{}{}",
                                        kind,
                                        model.node_state.best_block_number,
                                        target,
                                        speed
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
                            set_fraction: model.node_state.best_block_number as f64 / target as f64,
                        },
                    },
                    SyncState::Idle => gtk::Box {
                        gtk::Label {
                            #[watch]
                            set_label: &format!("Synced, best block #{}", model.node_state.best_block_number),
                        }
                    },
                },
            },

            gtk::Separator {
                set_margin_all: 10,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,
                set_valign: gtk::Align::Start,

                gtk::Label {
                    add_css_class: "heading",
                    set_halign: gtk::Align::Start,
                    set_label: "Farmer",
                },

                // TODO: Render all farms, not just the first one
                // TODO: Match only because `if let Some(x) = y` is not yet supported here: https://github.com/Relm4/Relm4/issues/582
                #[transition = "SlideUpDown"]
                match (model.farmer_state.plotting_state[0], model.node_state.sync_state) {
                    (PlottingState::Plotting { kind, progress, speed }, SyncState::Idle) => gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,

                        gtk::Box {
                            set_spacing: 10,

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                #[watch]
                                set_label: &{
                                    let kind = match kind {
                                        PlottingKind::Initial => "Initial plotting, not farming",
                                        PlottingKind::Replotting => "Replotting, farming",
                                    };

                                    format!(
                                        "{} {:.2}%{}",
                                        kind,
                                        progress,
                                        speed
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
                            set_fraction: progress as f64 / 100.0,
                        },
                    },
                    (PlottingState::Idle, SyncState::Idle) => gtk::Box {
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
    }

    fn init(
        _init: Self::Init,
        _root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            node_state: NodeState::default(),
            farmer_state: FarmerState::default(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        self.process_input(input);
    }
}

impl RunningView {
    fn process_input(&mut self, input: RunningInput) {
        match input {
            RunningInput::Initialize {
                best_block_number,
                num_farms,
            } => {
                self.node_state = NodeState {
                    best_block_number,
                    sync_state: SyncState::default(),
                };
                self.farmer_state = FarmerState {
                    plotting_state: vec![PlottingState::default(); num_farms],
                };
            }
            RunningInput::NodeNotification(node_notification) => match node_notification {
                NodeNotification::SyncStateUpdate(sync_state) => {
                    self.node_state.sync_state = sync_state;
                }
                NodeNotification::BlockImported { number } => {
                    self.node_state.best_block_number = number;
                }
            },
            RunningInput::FarmerNotification(farmer_notification) => match farmer_notification {
                FarmerNotification::PlottingStateUpdate { farm_index, state } => {
                    if let Some(plotting_state) =
                        self.farmer_state.plotting_state.get_mut(farm_index)
                    {
                        *plotting_state = state;
                    } else {
                        warn!(%farm_index, "Unexpected plotting farm index");
                    }
                }
            },
        }
    }
}
