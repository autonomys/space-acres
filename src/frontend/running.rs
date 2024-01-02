mod farm;

use crate::backend::config::RawConfig;
use crate::backend::farmer::{PlottingKind, PlottingState};
use crate::backend::node::{ChainInfo, SyncKind, SyncState};
use crate::backend::{FarmerNotification, NodeNotification};
use crate::frontend::running::farm::{FarmWidget, FarmWidgetInit, FarmWidgetInput};
use gtk::prelude::*;
use relm4::factory::FactoryHashMap;
use relm4::prelude::*;
use subspace_core_primitives::BlockNumber;
use subspace_runtime_primitives::{Balance, SSC};
use tracing::warn;

/// Maximum blocks to store in the import queue.
// HACK: This constant comes from Substrate's sync, but it is not public in there
const MAX_IMPORTING_BLOCKS: BlockNumber = 2048;

#[derive(Debug)]
pub enum RunningInput {
    Initialize {
        best_block_number: BlockNumber,
        reward_address_balance: Balance,
        initial_plotting_states: Vec<PlottingState>,
        farm_during_initial_plotting: bool,
        raw_config: RawConfig,
        chain_info: ChainInfo,
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
    initial_reward_address_balance: Balance,
    reward_address_balance: Balance,
    /// One entry per farm
    plotting_state: Vec<PlottingState>,
    farm_during_initial_plotting: bool,
    piece_cache_sync_progress: f32,
    reward_address: String,
}

#[derive(Debug)]
pub struct RunningView {
    node_state: NodeState,
    farmer_state: FarmerState,
    farms: FactoryHashMap<usize, FarmWidget>,
    chain_info: ChainInfo,
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
                    #[watch]
                    set_label: &format!(
                        "{} consensus node",
                        model.chain_info.chain_name.strip_prefix("Subspace ").unwrap_or(&model.chain_info.chain_name)
                    ),
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
                    gtk::Box {
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
                                    if matches!(model.node_state.sync_state, SyncState::Idle) && (model.farmer_state.farm_during_initial_plotting || replotting || idle) {
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
                        },

                        gtk::Box {
                            set_halign: gtk::Align::End,
                            set_hexpand: true,

                            gtk::LinkButton {
                                remove_css_class: "link",
                                set_tooltip: "Total account balance and coins farmed since application started, click to see details in Astral",
                                #[watch]
                                // TODO: Would be great to have `gemini-3g` in chain spec, but it is
                                //  not available in there in clean form
                                set_uri: &format!(
                                    "https://explorer.subspace.network/#/{}/consensus/accounts/{}",
                                    model.chain_info.protocol_id.strip_prefix("subspace-").unwrap_or(&model.chain_info.protocol_id),
                                    model.farmer_state.reward_address
                                ),
                                set_use_underline: false,

                                gtk::Label {
                                    #[watch]
                                    set_label: &{
                                        let current_balance = model.farmer_state.reward_address_balance;
                                        let balance_increase = model.farmer_state.reward_address_balance - model.farmer_state.initial_reward_address_balance;
                                        let current_balance = (current_balance / (SSC / 100)) as f32 / 100.0;
                                        let balance_increase = (balance_increase / (SSC / 100)) as f32 / 100.0;
                                        let token_symbol = &model.chain_info.token_symbol;

                                        format!(
                                            "{current_balance:.2}<span color=\"#3bbf2c\"><sup>+{balance_increase:.2}</sup></span> {token_symbol}"
                                        )
                                    },
                                    set_use_markup: true,
                                },
                            }
                        },
                    }
                },

                #[local_ref]
                farms_box -> gtk::Box {
                    set_margin_start: 10,
                    set_margin_end: 10,
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let farms = FactoryHashMap::builder()
            .launch(gtk::Box::default())
            .detach();

        let model = Self {
            node_state: NodeState::default(),
            farmer_state: FarmerState::default(),
            farms,
            chain_info: ChainInfo::default(),
        };

        let farms_box = model.farms.widget();
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
                reward_address_balance,
                initial_plotting_states,
                farm_during_initial_plotting,
                raw_config,
                chain_info,
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
                            farm_during_initial_plotting,
                        },
                    );
                }

                self.node_state = NodeState {
                    best_block_number,
                    sync_state: SyncState::default(),
                };
                self.farmer_state = FarmerState {
                    initial_reward_address_balance: reward_address_balance,
                    reward_address_balance,
                    reward_address: raw_config.reward_address().to_string(),
                    plotting_state: initial_plotting_states,
                    farm_during_initial_plotting,
                    piece_cache_sync_progress: 0.0,
                };
                self.chain_info = chain_info;
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
                NodeNotification::BlockImported(imported_block) => {
                    self.node_state.best_block_number = imported_block.number;
                    if !matches!(self.node_state.sync_state, SyncState::Idle) {
                        // Do not count balance increase during sync as increase related to farming,
                        // but preserve accumulated diff
                        let previous_diff = self.farmer_state.reward_address_balance
                            - self.farmer_state.initial_reward_address_balance;
                        self.farmer_state.initial_reward_address_balance =
                            imported_block.reward_address_balance - previous_diff;
                    }
                    // In case balance decreased, subtract it from initial balance to ignore, this
                    // typically happens due to chain reorg when reward is "disappears"
                    if let Some(decreased_by) = self
                        .farmer_state
                        .reward_address_balance
                        .checked_sub(imported_block.reward_address_balance)
                    {
                        self.farmer_state.initial_reward_address_balance -= decreased_by;
                    }
                    self.farmer_state.reward_address_balance =
                        imported_block.reward_address_balance;

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
