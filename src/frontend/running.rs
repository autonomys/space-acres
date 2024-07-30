mod farm;
mod node;

use crate::backend::config::{Config, RawConfig};
use crate::backend::farmer::{FarmerNotification, InitialFarmState};
use crate::backend::node::ChainInfo;
use crate::backend::{FarmIndex, NodeNotification};
use crate::frontend::running::farm::{FarmWidget, FarmWidgetInit, FarmWidgetInput};
use crate::frontend::running::node::{NodeInput, NodeView};
use crate::frontend::widgets::progress_circle::{
    ProgressCircle, ProgressCircleInit, ProgressCircleInput,
};
use gtk::prelude::*;
use relm4::factory::FactoryHashMap;
use relm4::prelude::*;
use relm4_icons::icon_name;
use sp_consensus_subspace::ChainConstants;
use std::time::{Duration, Instant};
use subspace_core_primitives::{solution_range_to_sectors, BlockNumber, Piece, SolutionRange};
use subspace_runtime_primitives::{Balance, SSC};
use tracing::{debug, error};

#[derive(Debug)]
pub struct RunningInit {
    pub plotting_paused: bool,
}

#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum RunningInput {
    Initialize {
        best_block_number: BlockNumber,
        reward_address_balance: Balance,
        initial_farm_states: Vec<InitialFarmState>,
        config: Config,
        raw_config: RawConfig,
        chain_info: ChainInfo,
        chain_constants: ChainConstants,
    },
    NodeNotification(NodeNotification),
    FarmerNotification(FarmerNotification<FarmIndex>),
    ToggleFarmDetails,
    TogglePausePlotting,
    // TODO: Use LinkButton once https://gitlab.gnome.org/GNOME/glib/-/issues/3403 is fixed
    //  for macOS
    OpenRewardAddressInExplorer,
}

#[derive(Debug)]
pub enum RunningOutput {
    PausePlotting(bool),
}

#[tracker::track]
#[derive(Debug)]
struct FarmerState {
    initial_reward_address_balance: Balance,
    reward_address_balance: Balance,
    piece_cache_sync_progress: f32,
    reward_address_url: String,
    token_symbol: String,
    local_space_pledged: u64,
    network_space_pledged: u128,
    slot_probability: (u64, u64),
    slot_duration: Duration,
    last_reward_received_time: Instant,
    #[do_not_track]
    reward_eta_progress_circle: Controller<ProgressCircle>,
}

#[tracker::track]
#[derive(Debug)]
pub struct RunningView {
    #[do_not_track]
    node_view: Controller<NodeView>,
    node_synced: bool,
    #[do_not_track]
    farmer_state: FarmerState,
    #[do_not_track]
    farms: FactoryHashMap<u8, FarmWidget>,
    plotting_paused: bool,
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
                            set_cursor_from_name: Some("pointer"),
                            set_has_frame: false,
                            set_icon_name: icon_name::GRID_FILLED,
                            set_tooltip: "Expand details about each farm",
                        },
                        gtk::ToggleButton {
                            connect_clicked => RunningInput::TogglePausePlotting,
                            set_active: model.plotting_paused,
                            set_cursor_from_name: Some("pointer"),
                            set_has_frame: false,
                            set_icon_name: icon_name::PAUSE,
                            set_tooltip: "Pause plotting/replotting, note that currently encoding sectors will not be interrupted",
                        },
                    },
                    gtk::Box {
                        set_halign: gtk::Align::End,
                        set_hexpand: true,

                        gtk::Box {
                            #[track = "model.changed_node_synced()"]
                            set_visible: model.node_synced,

                            model.farmer_state.reward_eta_progress_circle.widget().clone(),
                        },

                        // TODO: Use LinkButton once https://gitlab.gnome.org/GNOME/glib/-/issues/3403 is fixed
                        //  for macOS
                        gtk::Button {
                            // TODO: Use LinkButton once https://gitlab.gnome.org/GNOME/glib/-/issues/3403 is fixed
                            //  for macOS
                            connect_clicked => RunningInput::OpenRewardAddressInExplorer,
                            remove_css_class: "link",
                            set_cursor_from_name: Some("pointer"),
                            set_has_frame: false,
                            set_tooltip: "Total account balance and coins farmed since application started, click to see details in Astral",
                            // TODO: Use LinkButton once https://gitlab.gnome.org/GNOME/glib/-/issues/3403 is fixed
                            //  for macOS
                            // #[watch]
                            // set_uri: &model.farmer_state.reward_address_url,
                            set_use_underline: false,

                            gtk::Label {
                                #[track = "model.farmer_state.changed_reward_address_balance() || model.farmer_state.changed_initial_reward_address_balance() || model.farmer_state.changed_token_symbol()"]
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
                            #[track = "model.farmer_state.changed_piece_cache_sync_progress()"]
                            set_visible: model.farmer_state.piece_cache_sync_progress < 100.0,

                            gtk::Box {
                                set_spacing: 5,

                                gtk::Label {
                                    set_halign: gtk::Align::Start,

                                    #[track = "model.farmer_state.changed_piece_cache_sync_progress()"]
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
                                #[track = "model.farmer_state.changed_piece_cache_sync_progress()"]
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

        let reward_eta_progress_circle = ProgressCircle::builder()
            .launch(ProgressCircleInit {
                tooltip: String::new(),
                size: 16,
            })
            .detach();

        let model = Self {
            node_view,
            node_synced: false,
            farmer_state: FarmerState {
                initial_reward_address_balance: 0,
                reward_address_balance: 0,
                piece_cache_sync_progress: 0.0,
                reward_address_url: String::new(),
                token_symbol: String::new(),
                local_space_pledged: 0,
                network_space_pledged: 1,
                slot_probability: (1, 1),
                slot_duration: Duration::from_secs(1),
                last_reward_received_time: Instant::now(),
                reward_eta_progress_circle,
                tracker: u16::MAX,
            },
            farms,
            plotting_paused: init.plotting_paused,
            tracker: u8::MAX,
        };

        let farms_box = model.farms.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        // Reset changes
        self.reset();
        self.farmer_state.reset();

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
                config,
                raw_config,
                chain_info,
                chain_constants,
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

                self.farmer_state
                    .set_initial_reward_address_balance(reward_address_balance);
                self.farmer_state
                    .set_reward_address_balance(reward_address_balance);
                // TODO: Would be great to have `gemini-3h` in chain spec, but it is
                //  not available in there in clean form
                self.farmer_state.set_reward_address_url(format!(
                    "https://explorer.subspace.network/{}/consensus/accounts/{}",
                    chain_info
                        .protocol_id
                        .strip_prefix("subspace-")
                        .unwrap_or(&chain_info.protocol_id),
                    raw_config.reward_address()
                ));
                self.farmer_state
                    .get_mut_token_symbol()
                    .clone_from(&chain_info.token_symbol);
                self.farmer_state.local_space_pledged = config
                    .farms
                    .iter()
                    .map(|farm| farm.allocated_plotting_space)
                    .sum();
                self.farmer_state.slot_probability = chain_constants.slot_probability();
                self.farmer_state.slot_duration = chain_constants.slot_duration().as_duration();
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
                        self.set_node_synced(new_synced);
                    }
                    NodeNotification::BlockImported(imported_block) => {
                        if !self.node_synced {
                            // Do not count balance increase during sync as increase related to
                            // farming, but preserve accumulated diff
                            let previous_diff = self.farmer_state.reward_address_balance
                                - self.farmer_state.initial_reward_address_balance;
                            self.farmer_state.set_initial_reward_address_balance(
                                imported_block.reward_address_balance - previous_diff,
                            );
                        }
                        // In case balance decreased, subtract it from initial balance to ignore,
                        // this typically happens due to chain reorg when reward is "disappears"
                        if let Some(decreased_by) = self
                            .farmer_state
                            .reward_address_balance
                            .checked_sub(imported_block.reward_address_balance)
                        {
                            *self.farmer_state.get_mut_initial_reward_address_balance() -=
                                decreased_by;
                        }
                        if self.farmer_state.reward_address_balance
                            != imported_block.reward_address_balance
                        {
                            self.farmer_state
                                .set_reward_address_balance(imported_block.reward_address_balance);
                            self.farmer_state.last_reward_received_time = Instant::now();
                        }

                        let network_space_pledged_sectors = solution_range_to_sectors(
                            imported_block.solution_range,
                            self.farmer_state.slot_probability,
                            // It doesn't really matter what we use as pieces in sectors, just need
                            // to be the same in the next expression
                            2,
                        );
                        let network_space_pledged =
                            u128::from(network_space_pledged_sectors) * 2 * Piece::SIZE as u128;
                        self.farmer_state.network_space_pledged = network_space_pledged;

                        self.update_reward_eta_progress(imported_block.voting_solution_range);
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
                    self.farmer_state.set_piece_cache_sync_progress(progress);
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
            RunningInput::OpenRewardAddressInExplorer => {
                if let Err(error) = open::that_detached(&self.farmer_state.reward_address_url) {
                    error!(%error, "Failed to open explorer in default browser");
                }
            }
        }
    }

    fn update_reward_eta_progress(&self, voting_solution_range: SolutionRange) {
        // Space pledged derived from voting solution range is not real, but it is useful to
        // identify reward ETA because it is wider than regular solution range and will result
        // higher perceived fraction of the space pledged that will increase reward frequency from
        // block reward to vote reward, which is exactly what we want without having access to
        // expected number of votes per block
        let network_voting_space_pledged_sectors = solution_range_to_sectors(
            voting_solution_range,
            self.farmer_state.slot_probability,
            // It doesn't really matter what we use as pieces in sectors, just need
            // to be the same in the next expression
            2,
        );
        let network_voting_space_pledged =
            u128::from(network_voting_space_pledged_sectors) * 2 * Piece::SIZE as u128;

        // network_voting_space_pledged/local_space_pledged is a time multiplier based on how much
        // smaller space pledged is comparing to network space pledged, then we also account for
        // slot probability. The fact that we have votes and blocks is accounted for by using
        // solution range to calculate space pledged as explained above.
        let expected_reward_interval = self.farmer_state.slot_duration.as_millis()
            * network_voting_space_pledged
            / u128::from(self.farmer_state.local_space_pledged)
            / u128::from(self.farmer_state.slot_probability.0)
            * u128::from(self.farmer_state.slot_probability.1);
        let expected_reward_interval = Duration::from_millis(expected_reward_interval as u64);

        let eta = expected_reward_interval
            .saturating_sub(self.farmer_state.last_reward_received_time.elapsed());
        let eta_seconds = eta.as_secs();
        let progress = 1.0 - eta.as_secs_f64() / expected_reward_interval.as_secs_f64();
        let eta_string = if eta_seconds < 10 * 60 {
            "any time now"
        } else if eta_seconds <= 3600 {
            "less than an hour"
        } else if eta_seconds <= 24 * 3600 {
            "today"
        } else if eta_seconds <= 7 * 24 * 3600 {
            "this week"
        } else {
            "more than a week"
        };

        self.farmer_state
            .reward_eta_progress_circle
            .emit(ProgressCircleInput::Update {
                // Clamp upper bound to make sure something is being shown all the time
                progress: progress.clamp(0.05, 0.95),
                tooltip: format!("Next reward estimate: {eta_string}"),
            });
    }
}
