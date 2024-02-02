use crate::backend::config::Farm;
use crate::frontend::running::SmaWrapper;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4_icons::icon_name;
use simple_moving_average::{SingleSumSMA, SMA};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use subspace_core_primitives::SectorIndex;
use subspace_farmer::single_disk_farm::farming::FarmingNotification;
use subspace_farmer::single_disk_farm::{
    FarmingError, SectorExpirationDetails, SectorPlottingDetails, SectorUpdate,
};

/// Experimentally found number that is good for default window size to not have horizontal scroll
/// and allows for sectors to not occupy too much vertical space
const MIN_SECTORS_PER_ROW: u32 = 108;
/// Effectively no limit
const MAX_SECTORS_PER_ROW: u32 = 100_000;
/// Number of samples over which to track auditing time, 1 minute in slots
const AUDITING_TIME_TRACKING_WINDOW: usize = 60;
/// One second to audit
const MAX_AUDITING_TIME: Duration = Duration::from_secs(1);
/// 500ms auditing time is excellent, anything larger will result in auditing performance indicator decrease
const EXCELLENT_AUDITING_TIME: Duration = Duration::from_millis(500);
/// Number of samples over which to track proving time
const PROVING_TIME_TRACKING_WINDOW: usize = 10;
/// TODO: Ideally this would come from node's chain constants, but this will do for now
const BLOCK_AUTHORING_DELAY: Duration = Duration::from_secs(4);
/// 1800ms proving time is excellent, anything larger will result in proving performance indicator decrease
const EXCELLENT_PROVING_TIME: Duration = Duration::from_millis(1800);
/// Number of samples over which to track sector plotting time
const SECTOR_PLOTTING_TIME_TRACKING_WINDOW: usize = 10;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum PlottingKind {
    Initial,
    Replotting,
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum PlottingState {
    Plotting {
        kind: PlottingKind,
        /// Progress so far in % (not including this sector)
        progress: f32,
    },
    Idle,
}

#[derive(Debug)]
enum SectorState {
    Plotted,
    AboutToExpire,
    Expired,
    Downloading,
    Encoding,
    Writing,
}

impl SectorState {
    fn css_class(&self) -> &'static str {
        match self {
            Self::Plotted => "plotted",
            Self::AboutToExpire => "about-to-expire",
            Self::Expired => "expired",
            Self::Downloading => "downloading",
            Self::Encoding => "encoding",
            Self::Writing => "writing",
        }
    }
}

#[derive(Debug)]
pub(super) struct FarmWidgetInit {
    pub(super) farm: Farm,
    pub(super) total_sectors: SectorIndex,
    pub(super) plotted_total_sectors: SectorIndex,
    pub(super) farm_during_initial_plotting: bool,
}

#[derive(Debug, Clone)]
pub(super) enum FarmWidgetInput {
    SectorUpdate {
        sector_index: SectorIndex,
        update: SectorUpdate,
    },
    FarmingNotification(FarmingNotification),
    PieceCacheSynced(bool),
}

#[derive(Debug)]
pub(super) struct FarmWidget {
    path: PathBuf,
    size: String,
    auditing_time: SmaWrapper<Duration, u32, AUDITING_TIME_TRACKING_WINDOW>,
    proving_time: SmaWrapper<Duration, u32, PROVING_TIME_TRACKING_WINDOW>,
    sector_plotting_time: SmaWrapper<Duration, u32, SECTOR_PLOTTING_TIME_TRACKING_WINDOW>,
    last_sector_plotted: Option<SectorIndex>,
    plotting_state: PlottingState,
    is_piece_cache_synced: bool,
    farm_during_initial_plotting: bool,
    sectors_grid: gtk::GridView,
    sectors: HashMap<SectorIndex, gtk::Box>,
    non_fatal_farming_error: Option<Arc<FarmingError>>,
}

#[relm4::factory(pub(super))]
impl FactoryComponent for FarmWidget {
    type Init = FarmWidgetInit;
    type Input = FarmWidgetInput;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type Index = u8;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            gtk::Box {
                gtk::Label {
                    set_halign: gtk::Align::Start,
                    set_label: &format!("{} [{}]:", self.path.display(), self.size),
                },

                gtk::Box {
                    set_halign: gtk::Align::End,
                    set_hexpand: true,
                    set_spacing: 10,

                    gtk::Box {
                        set_spacing: 10,
                        #[watch]
                        set_tooltip: &format!(
                            "Auditing performance: average time {:.2}s, time limit {:.2}s",
                            self.auditing_time.get_average().as_secs_f32(),
                            MAX_AUDITING_TIME.as_secs_f32()
                        ),
                        #[watch]
                        set_visible: self.auditing_time.get_num_samples() > 0,

                        gtk::Image {
                            set_icon_name: Some(icon_name::PUZZLE_PIECE),
                        },

                        gtk::LevelBar {
                            add_css_class: "auditing-performance",
                            #[watch]
                            set_value: {
                                let average_time = self.auditing_time.get_average();
                                let slot_time_fraction_remaining = 1.0 - average_time.as_secs_f64() / MAX_AUDITING_TIME.as_secs_f64();
                                let excellent_time_fraction_remaining = 1.0 - EXCELLENT_AUDITING_TIME.as_secs_f64() / MAX_AUDITING_TIME.as_secs_f64();
                                (slot_time_fraction_remaining / excellent_time_fraction_remaining).clamp(0.0, 1.0)
                            },
                            set_width_request: 70,
                        },
                    },

                    gtk::Box {
                        set_spacing: 10,
                        #[watch]
                        set_tooltip: &format!(
                            "Proving performance: average time {:.2}s, time limit {:.2}s",
                            self.proving_time.get_average().as_secs_f32(),
                            BLOCK_AUTHORING_DELAY.as_secs_f32()
                        ),
                        #[watch]
                        set_visible: self.proving_time.get_num_samples() > 0,

                        gtk::Image {
                            set_icon_name: Some(icon_name::PROCESSOR),
                        },

                        gtk::LevelBar {
                            add_css_class: "proving-performance",
                            #[watch]
                            set_value: {
                                let average_time = self.proving_time.get_average();
                                let slot_time_fraction_remaining = 1.0 - average_time.as_secs_f64() / BLOCK_AUTHORING_DELAY.as_secs_f64();
                                let excellent_time_fraction_remaining = 1.0 - EXCELLENT_PROVING_TIME.as_secs_f64() / BLOCK_AUTHORING_DELAY.as_secs_f64();
                                (slot_time_fraction_remaining / excellent_time_fraction_remaining).clamp(0.0, 1.0)
                            },
                            set_width_request: 70,
                        },
                    },

                    gtk::Image {
                        set_icon_name: Some(icon_name::WARNING),
                        set_tooltip: &{
                            let last_error = self.non_fatal_farming_error
                                .as_ref()
                                .map(|error| error.to_string())
                                .unwrap_or_default();

                            format!("Non-fatal farming error happened and was recovered, see logs for more details: {last_error}")
                        },
                        set_visible: self.non_fatal_farming_error.is_some(),
                    },
                },
            },

            #[transition = "SlideUpDown"]
            match (self.plotting_state, self.is_piece_cache_synced) {
                (_, false) => gtk::Label {
                    set_halign: gtk::Align::Start,
                    set_label: "Waiting for piece cache sync",
                },
                (PlottingState::Plotting { kind, progress }, _) => gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Box {
                        set_spacing: 5,
                        set_tooltip: if self.farm_during_initial_plotting {
                            "Farming runs in parallel to plotting on CPUs with more than 8 logical cores"
                        } else {
                            "Farming starts after initial plotting is complete on CPUs with 8 or less logical cores"
                        },

                        gtk::Label {
                            set_halign: gtk::Align::Start,

                            #[watch]
                            set_label: &{
                                let plotting_speed = if self.sector_plotting_time.get_num_samples() > 0 {
                                     format!(
                                        " ({:.2} m/sector, {:.2} sectors/h)",
                                        self.sector_plotting_time.get_average().as_secs_f32() / 60.0,
                                        3600.0 / self.sector_plotting_time.get_average().as_secs_f32()
                                    )
                                } else {
                                    String::new()
                                };

                                match kind {
                                    PlottingKind::Initial => {
                                        if self.farm_during_initial_plotting {
                                            format!(
                                                "Initial plotting {:.2}%{}, farming",
                                                progress,
                                                plotting_speed,
                                            )
                                        } else {
                                            format!(
                                                "Initial plotting {:.2}%{}, not farming",
                                                progress,
                                                plotting_speed,
                                            )
                                        }
                                    },
                                    PlottingKind::Replotting => format!(
                                        "Replotting {:.2}%{}, farming",
                                        progress,
                                        plotting_speed,
                                    ),
                                }
                            },
                        },

                        gtk::Spinner {
                            start: (),
                        },
                    },

                    gtk::ProgressBar {
                        #[watch]
                        set_fraction: progress as f64 / 100.0,
                    },
                },
                (PlottingState::Idle, _) => gtk::Box {
                    gtk::Label {
                        set_label: "Farming",
                    }
                },
            },

            self.sectors_grid.clone() -> gtk::GridView {
                remove_css_class: "view",
                set_max_columns: MAX_SECTORS_PER_ROW,
                set_min_columns: MIN_SECTORS_PER_ROW.min(self.sectors.len() as u32 - 1),
                set_sensitive: false,
            },
        },
    }

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        let mut sectors = Vec::with_capacity(usize::from(init.total_sectors));
        for sector_index in 0..init.total_sectors {
            let sector = gtk::Box::builder()
                .css_name("farm-sector")
                .tooltip_text(format!("Sector {sector_index}"))
                .build();
            if sector_index < init.plotted_total_sectors {
                sector.add_css_class("plotted")
            }
            sectors.push(sector);
        }

        let factory = gtk::SignalListItemFactory::new();
        factory.connect_bind(|_, list_item| {
            if let Some(item) = list_item.item() {
                list_item.set_child(Some(
                    &item
                        .downcast::<gtk::Box>()
                        .expect("Box was created above; qed"),
                ));
            }
        });

        let selection = gtk::NoSelection::new(Some({
            let store = gtk::gio::ListStore::new::<gtk::Box>();
            store.extend_from_slice(&sectors);
            store
        }));
        let sectors_grid = gtk::GridView::builder()
            .single_click_activate(true)
            .model(&selection)
            .factory(&factory)
            .css_name("farm-sectors")
            .build();

        Self {
            path: init.farm.path,
            size: init.farm.size,
            auditing_time: SmaWrapper(SingleSumSMA::from_zero(Duration::ZERO)),
            proving_time: SmaWrapper(SingleSumSMA::from_zero(Duration::ZERO)),
            sector_plotting_time: SmaWrapper(SingleSumSMA::from_zero(Duration::ZERO)),
            last_sector_plotted: None,
            plotting_state: PlottingState::Idle,
            is_piece_cache_synced: false,
            farm_during_initial_plotting: init.farm_during_initial_plotting,
            sectors_grid,
            sectors: HashMap::from_iter((SectorIndex::MIN..).zip(sectors)),
            non_fatal_farming_error: None,
        }
    }

    fn update(&mut self, input: Self::Input, _sender: FactorySender<Self>) {
        self.process_input(input);
    }
}

impl FarmWidget {
    fn process_input(&mut self, input: FarmWidgetInput) {
        match input {
            FarmWidgetInput::SectorUpdate {
                sector_index,
                update,
            } => match update {
                SectorUpdate::Plotting(plotting_update) => match plotting_update {
                    SectorPlottingDetails::Starting {
                        progress,
                        replotting,
                        last_queued,
                    } => {
                        self.plotting_state = PlottingState::Plotting {
                            kind: if replotting {
                                PlottingKind::Replotting
                            } else {
                                PlottingKind::Initial
                            },
                            progress,
                        };

                        if last_queued {
                            self.last_sector_plotted.replace(sector_index);
                        }
                    }
                    SectorPlottingDetails::Downloading => {
                        self.update_sector_state(sector_index, SectorState::Downloading);
                    }
                    SectorPlottingDetails::Downloaded(_) => {
                        self.remove_sector_state(sector_index, SectorState::Downloading);
                    }
                    SectorPlottingDetails::Encoding => {
                        self.update_sector_state(sector_index, SectorState::Encoding);
                    }
                    SectorPlottingDetails::Encoded(_) => {
                        self.remove_sector_state(sector_index, SectorState::Encoding);
                    }
                    SectorPlottingDetails::Writing => {
                        self.update_sector_state(sector_index, SectorState::Writing);
                    }
                    SectorPlottingDetails::Written(_) => {
                        self.remove_sector_state(sector_index, SectorState::Writing);
                    }
                    SectorPlottingDetails::Finished { time, .. } => {
                        if self.last_sector_plotted == Some(sector_index) {
                            self.last_sector_plotted.take();

                            self.plotting_state = PlottingState::Idle;
                        }

                        self.update_sector_state(sector_index, SectorState::Plotted);
                        self.sector_plotting_time.add_sample(time);
                    }
                },
                SectorUpdate::Expiration(expiration_update) => match expiration_update {
                    SectorExpirationDetails::Determined { .. } => {
                        // TODO: Track segments to mark sector as about to expire/expired even if
                        //  farmer is still busy plotting previously expired sectors
                    }
                    SectorExpirationDetails::AboutToExpire => {
                        self.update_sector_state(sector_index, SectorState::AboutToExpire);
                    }
                    SectorExpirationDetails::Expired => {
                        self.update_sector_state(sector_index, SectorState::Expired);
                    }
                },
            },
            FarmWidgetInput::FarmingNotification(notification) => match notification {
                FarmingNotification::Auditing(auditing_details) => {
                    self.auditing_time.add_sample(auditing_details.time);
                }
                FarmingNotification::Proving(proving_details) => {
                    self.proving_time.add_sample(proving_details.time);
                }
                FarmingNotification::NonFatalError(error) => {
                    self.non_fatal_farming_error.replace(error);
                }
            },
            FarmWidgetInput::PieceCacheSynced(synced) => {
                self.is_piece_cache_synced = synced;
            }
        }
    }

    fn update_sector_state(&self, sector_index: SectorIndex, sector_state: SectorState) {
        if let Some(sector) = self.sectors.get(&sector_index) {
            match sector_state {
                SectorState::Plotted | SectorState::AboutToExpire | SectorState::Expired => {
                    sector.set_css_classes(&[sector_state.css_class()]);
                }
                SectorState::Downloading | SectorState::Encoding | SectorState::Writing => {
                    sector.add_css_class(sector_state.css_class());
                }
            }
        }
    }

    fn remove_sector_state(&self, sector_index: SectorIndex, sector_state: SectorState) {
        if let Some(sector) = self.sectors.get(&sector_index) {
            sector.remove_css_class(sector_state.css_class());
        }
    }
}
