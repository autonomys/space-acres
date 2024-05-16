mod farm;
mod node;

use crate::backend::config::RawConfig;
use crate::backend::farmer::{FarmerNotification, InitialFarmState};
use crate::backend::node::ChainInfo;
use crate::backend::{FarmIndex, NodeNotification};
use crate::frontend::progress_bar::create_circular_progress_bar;
use crate::frontend::running::farm::{FarmWidget, FarmWidgetInit, FarmWidgetInput};
use crate::frontend::running::node::{NodeInput, NodeView};
use gtk::prelude::*;
use relm4::factory::FactoryHashMap;
use relm4::prelude::*;
use relm4_icons::icon_name;
use subspace_core_primitives::BlockNumber;
use subspace_runtime_primitives::{Balance, SSC};
use tracing::debug;

#[derive(Debug)]
pub struct RunningInit {
    pub plotting_paused: bool,
}

#[derive(Debug)]
pub enum RunningInput {
    Initialize {
        best_block_number: BlockNumber,
        reward_address_balance: Balance,
        initial_farm_states: Vec<InitialFarmState>,
        raw_config: RawConfig,
        chain_info: ChainInfo,
    },
    NodeNotification(NodeNotification),
    FarmerNotification(FarmerNotification<FarmIndex>),
    ToggleFarmDetails,
    TogglePausePlotting,
}

#[derive(Debug)]
pub enum RunningOutput {
    PausePlotting(bool),
}

#[derive(Debug, Default)]
struct FarmerState {
    initial_reward_address_balance: Balance,
    reward_address_balance: Balance,
    piece_cache_sync_progress: f32,
    reward_address_url: String,
    token_symbol: String,
}

#[derive(Debug)]
pub struct RunningView {
    node_view: Controller<NodeView>,
    node_synced: bool,
    farmer_state: FarmerState,
    farms: FactoryHashMap<u8, FarmWidget>,
    plotting_paused: bool,
    circular_progress_bar: gtk::DrawingArea,
}

#[relm4::component(pub)]
impl Component for RunningView {
    type Init = RunningInit;
    type Input = RunningInput;
    type Output = RunningOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            model.node_view.widget().clone(),


            gtk::Separator {
                set_margin_all: 10,
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                gtk::Box {
                    set_spacing: 10,

                    gtk::Label {
                        add_css_class: "heading",
                        set_halign: gtk::Align::Start,
                        set_label: "Farmer",
                    },
                    gtk::Box {
                        gtk::ToggleButton {
                            connect_clicked => RunningInput::ToggleFarmDetails,
                            set_has_frame: false,
                            set_icon_name: icon_name::GRID_FILLED,
                            set_tooltip: "Expand details about each farm",
                        },
                        gtk::ToggleButton {
                            connect_clicked => RunningInput::TogglePausePlotting,
                            set_active: model.plotting_paused,
                            set_has_frame: false,
                            set_icon_name: icon_name::PAUSE,
                            set_tooltip: "Pause plotting/replotting, note that currently encoding sectors will not be interrupted",
                        },
                    },
                    gtk::Box {
                        set_halign: gtk::Align::End,
                        set_hexpand: true,

                        model.circular_progress_bar.clone(),

                        gtk::LinkButton {
                            remove_css_class: "link",
                            set_tooltip: "Total account balance and coins farmed since application started, click to see details in Astral",
                            #[watch]
                            set_uri: &model.farmer_state.reward_address_url,
                            set_use_underline: false,

                            gtk::Label {
                                #[watch]
                                set_label: &{
                                    let current_balance = model.farmer_state.reward_address_balance;
                                    let balance_increase = model.farmer_state.reward_address_balance - model.farmer_state.initial_reward_address_balance;
                                    let current_balance = (current_balance / (SSC / 100)) as f32 / 100.0;
                                    let balance_increase = (balance_increase / (SSC / 100)) as f32 / 100.0;
                                    let token_symbol = &model.farmer_state.token_symbol;

                                    format!(
                                        "{current_balance:.2}<span color=\"#3bbf2c\"><sup>+{balance_increase:.2}</sup></span> {token_symbol}"
                                    )
                                },
                                set_use_markup: true,
                            },
                        }
                    },
                },

                gtk::ScrolledWindow {
                    set_margin_start: 10,
                    set_margin_end: 10,
                    set_vexpand: true,

                    gtk::Box {
                        set_orientation: gtk::Orientation::Vertical,
                        set_spacing: 10,

                        gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 10,
                            #[watch]
                            set_visible: model.farmer_state.piece_cache_sync_progress < 100.0,

                            gtk::Box {
                                set_spacing: 5,

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
                        },

                        #[local_ref]
                        farms_box -> gtk::Box {
                            set_orientation: gtk::Orientation::Vertical,
                            set_spacing: 10,
                        },
                    },
                },
            },
        }
    }

    fn init(
        init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let node_view = NodeView::builder().launch(()).detach();
        let farms = FactoryHashMap::builder()
            .launch(gtk::Box::default())
            .detach();

        let model = Self {
            node_view,
            node_synced: false,
            farmer_state: FarmerState::default(),
            farms,
            plotting_paused: init.plotting_paused,
            circular_progress_bar: create_circular_progress_bar(20.0),
        };

        let farms_box = model.farms.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        self.process_input(input, sender);
    }
}

impl RunningView {
    fn process_input(&mut self, input: RunningInput, sender: ComponentSender<Self>) {
        match input {
            RunningInput::Initialize {
                best_block_number,
                reward_address_balance,
                initial_farm_states,
                raw_config,
                chain_info,
            } => {
                for (farm_index, (initial_farm_state, farm)) in initial_farm_states
                    .iter()
                    .copied()
                    .zip(raw_config.farms().iter().cloned())
                    .enumerate()
                {
                    self.farms.insert(
                        u8::try_from(farm_index).expect(
                            "More than 256 plots are not supported, this is checked on \
                            backend; qed",
                        ),
                        FarmWidgetInit {
                            farm,
                            total_sectors: initial_farm_state.total_sectors_count,
                            plotted_total_sectors: initial_farm_state.plotted_sectors_count,
                            plotting_paused: self.plotting_paused,
                        },
                    );
                }

                self.farmer_state = FarmerState {
                    initial_reward_address_balance: reward_address_balance,
                    reward_address_balance,
                    piece_cache_sync_progress: 0.0,
                    // TODO: Would be great to have `gemini-3h` in chain spec, but it is
                    //  not available in there in clean form
                    reward_address_url: format!(
                        "https://explorer.subspace.network/#/{}/consensus/accounts/{}",
                        chain_info
                            .protocol_id
                            .strip_prefix("subspace-")
                            .unwrap_or(&chain_info.protocol_id),
                        raw_config.reward_address()
                    ),
                    token_symbol: chain_info.token_symbol.clone(),
                };
                self.node_view.emit(NodeInput::Initialize {
                    best_block_number,
                    chain_info,
                    node_path: raw_config.node_path().clone(),
                });
            }
            RunningInput::NodeNotification(node_notification) => {
                self.node_view
                    .emit(NodeInput::NodeNotification(node_notification.clone()));

                match node_notification {
                    NodeNotification::SyncStateUpdate(sync_state) => {
                        let new_synced = sync_state.is_synced();
                        if self.node_synced != new_synced {
                            self.farms
                                .broadcast(FarmWidgetInput::NodeSynced(new_synced));
                        }
                        self.node_synced = new_synced;
                    }
                    NodeNotification::BlockImported(imported_block) => {
                        if !self.node_synced {
                            // Do not count balance increase during sync as increase related to
                            // farming, but preserve accumulated diff
                            let previous_diff = self.farmer_state.reward_address_balance
                                - self.farmer_state.initial_reward_address_balance;
                            self.farmer_state.initial_reward_address_balance =
                                imported_block.reward_address_balance - previous_diff;
                        }
                        // In case balance decreased, subtract it from initial balance to ignore,
                        // this typically happens due to chain reorg when reward is "disappears"
                        if let Some(decreased_by) = self
                            .farmer_state
                            .reward_address_balance
                            .checked_sub(imported_block.reward_address_balance)
                        {
                            self.farmer_state.initial_reward_address_balance -= decreased_by;
                        }
                        self.farmer_state.reward_address_balance =
                            imported_block.reward_address_balance;
                    }
                }
            }
            RunningInput::FarmerNotification(farmer_notification) => match farmer_notification {
                FarmerNotification::SectorUpdate {
                    farm_index,
                    sector_index,
                    update,
                } => {
                    self.farms.send(
                        &farm_index,
                        FarmWidgetInput::SectorUpdate {
                            sector_index,
                            update,
                        },
                    );
                }
                FarmerNotification::FarmingNotification {
                    farm_index,
                    notification,
                } => {
                    self.farms.send(
                        &farm_index,
                        FarmWidgetInput::FarmingNotification(notification),
                    );
                }
                FarmerNotification::FarmerCacheSyncProgress { progress } => {
                    self.farmer_state.piece_cache_sync_progress = progress;
                }
                FarmerNotification::FarmError { farm_index, error } => {
                    self.farms
                        .send(&farm_index, FarmWidgetInput::Error { error });
                }
            },
            RunningInput::ToggleFarmDetails => {
                self.farms.broadcast(FarmWidgetInput::ToggleFarmDetails);
            }
            RunningInput::TogglePausePlotting => {
                self.plotting_paused = !self.plotting_paused;
                self.farms
                    .broadcast(FarmWidgetInput::PausePlotting(self.plotting_paused));
                if sender
                    .output(RunningOutput::PausePlotting(self.plotting_paused))
                    .is_err()
                {
                    debug!("Failed to send RunningOutput::TogglePausePlotting");
                }
            }
        }
    }
}
