mod farm;

use crate::backend::config::RawConfig;
use crate::backend::farmer::{PlottingKind, PlottingState};
use crate::backend::node::{SyncKind, SyncState};
use crate::backend::{FarmerNotification, NodeNotification};
use crate::frontend::running::farm::{FarmWidget, FarmWidgetInit, FarmWidgetInput};
use gtk::prelude::*;
use relm4::factory::FactoryHashMap;
use relm4::prelude::*;
use subspace_core_primitives::BlockNumber;
use tracing::warn;

/// Maximum blocks to store in the import queue.
// HACK: This constant comes from Substrate's sync, but it is not public in there
const MAX_IMPORTING_BLOCKS: BlockNumber = 2048;

#[derive(Debug)]
pub enum RunningInput {
    Initialize {
        best_block_number: BlockNumber,
        initial_plotting_states: Vec<PlottingState>,
        raw_config: RawConfig,
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
    piece_cache_sync_progress: f32,
}

#[derive(Debug)]
pub struct RunningView {
    node_state: NodeState,
    farmer_state: FarmerState,
    farms: FactoryHashMap<usize, FarmWidget>,
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
                            set_spacing: 5,

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
                if model.farmer_state.piece_cache_sync_progress < 100.0 {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,

                        gtk::Box {
                            set_spacing: 5,
                            set_tooltip: "Plotting starts after piece cache sync is complete",

                            gtk::Label {
                                set_halign: gtk::Align::Start,

                                #[watch]
                                set_label: &format!(
                                    "Piece cache sync {:.2}%",
                                    model.farmer_state.piece_cache_sync_progress
                                ),
                            },

                            gtk::Spinner {
                                start: (),
                            },
                        },

                        gtk::ProgressBar {
                            #[watch]
                            set_fraction: model.farmer_state.piece_cache_sync_progress as f64 / 100.0,
                        },
                    }
                } else {
                    gtk::Label {
                        set_halign: gtk::Align::Start,
                        #[watch]
                        set_label: &{
                            if matches!(model.node_state.sync_state, SyncState::Idle) {
                                let mut statuses = Vec::new();
                                let plotting = model.farmer_state.plotting_state.iter().any(|plotting_state| {
                                    matches!(plotting_state, PlottingState::Plotting { kind: PlottingKind::Initial, .. })
                                });
                                let replotting = model.farmer_state.plotting_state.iter().any(|plotting_state| {
                                    matches!(plotting_state, PlottingState::Plotting { kind: PlottingKind::Replotting, .. })
                                });
                                let idle = model.farmer_state.plotting_state.iter().any(|plotting_state| {
                                    matches!(plotting_state, PlottingState::Idle)
                                });
                                if plotting {
                                    statuses.push(if statuses.is_empty() {
                                        "Plotting"
                                    } else {
                                        "plotting"
                                    });
                                }
                                if matches!(model.node_state.sync_state, SyncState::Idle) && (replotting || idle) {
                                    statuses.push(if statuses.is_empty() {
                                        "Farming"
                                    } else {
                                        "farming"
                                    });
                                }
                                if replotting {
                                    statuses.push(if statuses.is_empty() {
                                        "Replotting"
                                    } else {
                                        "replotting"
                                    });
                                }

                                statuses.join(", ")
                            } else {
                                "Waiting for node to sync first".to_string()
                            }
                        },
                        // TODO: Show summarized state of all farms: Plotting, Replotting, Farming
                    }
                },

                model.farms.widget(),
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let farms = FactoryHashMap::builder()
            .launch(
                gtk::Box::builder()
                    .margin_start(10)
                    .margin_end(10)
                    .orientation(gtk::Orientation::Vertical)
                    .spacing(10)
                    .build(),
            )
            .detach();

        let model = Self {
            node_state: NodeState::default(),
            farmer_state: FarmerState::default(),
            farms,
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
                initial_plotting_states,
                raw_config,
            } => {
                for (farm_index, (initial_plotting_state, farm)) in initial_plotting_states
                    .iter()
                    .copied()
                    .zip(raw_config.farms().iter().cloned())
                    .enumerate()
                {
                    self.farms.insert(
                        farm_index,
                        FarmWidgetInit {
                            initial_plotting_state,
                            farm,
                        },
                    );
                }

                self.node_state = NodeState {
                    best_block_number,
                    sync_state: SyncState::default(),
                };
                self.farmer_state = FarmerState {
                    plotting_state: initial_plotting_states,
                    piece_cache_sync_progress: 0.0,
                };
            }
            RunningInput::NodeNotification(node_notification) => match node_notification {
                NodeNotification::SyncStateUpdate(mut sync_state) => {
                    if let SyncState::Syncing {
                        target: new_target, ..
                    } = &mut sync_state
                    {
                        *new_target = (*new_target).max(self.node_state.best_block_number);

                        // Ensure target is never below current block
                        if let SyncState::Syncing {
                            target: old_target, ..
                        } = &self.node_state.sync_state
                        {
                            // If old target was within `MAX_IMPORTING_BLOCKS` from new target, keep old target
                            if old_target
                                .checked_sub(*new_target)
                                .map(|diff| diff <= MAX_IMPORTING_BLOCKS)
                                .unwrap_or_default()
                            {
                                *new_target = *old_target;
                            }
                        }
                    }

                    let old_synced = matches!(self.node_state.sync_state, SyncState::Idle);
                    let new_synced = matches!(sync_state, SyncState::Idle);
                    if old_synced != new_synced {
                        self.farms
                            .broadcast(FarmWidgetInput::NodeSynced(new_synced));
                    }
                    self.node_state.sync_state = sync_state;
                }
                NodeNotification::BlockImported { number } => {
                    self.node_state.best_block_number = number;

                    // Ensure target is never below current block
                    if let SyncState::Syncing { target, .. } = &mut self.node_state.sync_state {
                        *target = (*target).max(self.node_state.best_block_number);
                    }
                }
            },
            RunningInput::FarmerNotification(farmer_notification) => match farmer_notification {
                FarmerNotification::PlottingStateUpdate { farm_index, state } => {
                    self.farms
                        .send(&farm_index, FarmWidgetInput::PlottingStateUpdate(state));

                    if let Some(plotting_state) =
                        self.farmer_state.plotting_state.get_mut(farm_index)
                    {
                        *plotting_state = state;
                    } else {
                        warn!(%farm_index, "Unexpected plotting farm index");
                    }
                }
                FarmerNotification::PieceCacheSyncProgress { progress } => {
                    let old_synced = self.farmer_state.piece_cache_sync_progress == 100.0;
                    let new_synced = progress == 100.0;
                    if old_synced != new_synced {
                        self.farms
                            .broadcast(FarmWidgetInput::PieceCacheSynced(new_synced));
                    }

                    self.farmer_state.piece_cache_sync_progress = progress;
                }
            },
        }
    }
}
