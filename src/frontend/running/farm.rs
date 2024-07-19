use crate::backend::config::Farm;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4_icons::icon_name;
use simple_moving_average::{SingleSumSMA, SMA};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use subspace_core_primitives::SectorIndex;
use subspace_farmer::farm::{
    FarmingError, FarmingNotification, SectorExpirationDetails, SectorPlottingDetails, SectorUpdate,
};
use tracing::error;

/// Experimentally found number that is good for default window size to not have horizontal scroll
const SECTORS_PER_ROW: usize = 108;
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
    pub(super) plotting_paused: bool,
}

#[derive(Debug, Clone)]
pub(super) enum FarmWidgetInput {
    SectorUpdate {
        sector_index: SectorIndex,
        update: SectorUpdate,
    },
    FarmingNotification(FarmingNotification),
    PausePlotting(bool),
    OpenFarmFolder,
    NodeSynced(bool),
    ToggleFarmDetails,
    Error {
        error: Arc<anyhow::Error>,
    },
}

#[tracker::track]
#[derive(Debug)]
pub(super) struct FarmWidget {
    path: PathBuf,
    size: String,
    #[no_eq]
    auditing_time: SingleSumSMA<Duration, u32, AUDITING_TIME_TRACKING_WINDOW>,
    #[no_eq]
    proving_time: SingleSumSMA<Duration, u32, PROVING_TIME_TRACKING_WINDOW>,
    #[no_eq]
    sector_plotting_time: SingleSumSMA<Duration, u32, SECTOR_PLOTTING_TIME_TRACKING_WINDOW>,
    last_sector_plotted: Option<SectorIndex>,
    plotting_state: PlottingState,
    is_node_synced: bool,
    sector_rows: gtk::Box,
    sectors: HashMap<SectorIndex, gtk::Box>,
    #[no_eq]
    non_fatal_farming_error: Option<Arc<FarmingError>>,
    farm_details: bool,
    encoding_sectors: usize,
    plotting_paused: bool,
    #[no_eq]
    error: Option<Arc<anyhow::Error>>,
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

            gtk::Box {
                gtk::Button {
                    add_css_class: "folder-button",
                    connect_clicked => FarmWidgetInput::OpenFarmFolder,
                    set_halign: gtk::Align::Start,
                    set_has_frame: false,
                    set_tooltip: "Click to open in file manager",

                    gtk::Label {
                        #[track = "self.changed_error()"]
                        set_css_classes: if self.error.is_some() {
                            &["farm-error"]
                        } else {
                            &[]
                        },
                        set_label: &format!("{} [{}]:", self.path.display(), self.size),
                    },
                },

                match &self.error {
                    Some(_error) => gtk::Box {
                        add_css_class: "farm-error",
                        set_halign: gtk::Align::End,
                        set_hexpand: true,

                        gtk::Image {
                            set_icon_name: Some(icon_name::WARNING),
                        }
                    },
                    None => {
                        gtk::Box {
                            set_halign: gtk::Align::End,
                            set_hexpand: true,
                            set_margin_top: 5,
                            set_spacing: 10,

                            // TODO: Optimize rendering here, it will update every second
                            gtk::Box {
                                set_spacing: 5,
                                #[track = "self.changed_auditing_time()"]
                                set_tooltip: &format!(
                                    "Auditing performance: average time {:.2}s, time limit {:.2}s",
                                    self.auditing_time.get_average().as_secs_f32(),
                                    MAX_AUDITING_TIME.as_secs_f32()
                                ),
                                #[track = "self.changed_auditing_time()"]
                                set_visible: self.auditing_time.get_num_samples() > 0,

                                gtk::Image {
                                    set_icon_name: Some(icon_name::PUZZLE_PIECE),
                                },

                                gtk::LevelBar {
                                    add_css_class: "auditing-performance",
                                    #[track = "self.changed_auditing_time()"]
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
                                set_spacing: 5,
                                #[track = "self.changed_proving_time()"]
                                set_tooltip: &format!(
                                    "Proving performance: average time {:.2}s, time limit {:.2}s",
                                    self.proving_time.get_average().as_secs_f32(),
                                    BLOCK_AUTHORING_DELAY.as_secs_f32()
                                ),
                                #[track = "self.changed_proving_time()"]
                                set_visible: self.proving_time.get_num_samples() > 0,

                                gtk::Image {
                                    set_icon_name: Some(icon_name::PROCESSOR),
                                },

                                gtk::LevelBar {
                                    add_css_class: "proving-performance",
                                    #[track = "self.changed_proving_time()"]
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
                                #[track = "self.changed_non_fatal_farming_error()"]
                                set_tooltip: &{
                                    let last_error = self.non_fatal_farming_error
                                        .as_ref()
                                        .map(|error| error.to_string())
                                        .unwrap_or_default();

                                    format!("Non-fatal farming error happened and was recovered, see logs for more details: {last_error}")
                                },
                                #[track = "self.changed_non_fatal_farming_error()"]
                                set_visible: self.non_fatal_farming_error.is_some(),
                            },
                        }
                    },
                },
            },

            #[transition = "SlideUpDown"]
            match (&self.error, self.plotting_state) {
                (Some(error), _) => gtk::Box {
                    gtk::Label {
                        add_css_class: "farm-error",
                        set_halign: gtk::Align::Start,
                        #[track = "self.changed_error()"]
                        set_label: &format!("Farm crashed: {error}"),
                    }
                },
                (_, PlottingState::Plotting { kind, progress }) => gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Box {
                        set_spacing: 5,

                        gtk::Label {
                            set_halign: gtk::Align::Start,

                            #[track = "self.changed_plotting_state() || self.changed_encoding_sectors() || self.changed_plotting_paused() || self.changed_is_node_synced()"]
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
                                        let initial_plotting = if self.plotting_paused {
                                            if self.encoding_sectors > 0 {
                                                "Pausing initial plotting"
                                            } else {
                                                "Paused initial plotting"
                                            }
                                        } else {
                                            "Initial plotting"
                                        };
                                        let farming = if self.is_node_synced {
                                            "farming"
                                        } else {
                                            "not farming"
                                        };
                                        format!(
                                            "{} {:.2}%{}, {}",
                                            initial_plotting,
                                            progress,
                                            plotting_speed,
                                            farming,
                                        )
                                    },
                                    PlottingKind::Replotting => {
                                        let replotting = if self.plotting_paused {
                                            if self.encoding_sectors > 0 {
                                                "Pausing replotting"
                                            } else {
                                                "Paused replotting"
                                            }
                                        } else {
                                            "Replotting"
                                        };
                                        let farming = if self.is_node_synced {
                                            "farming"
                                        } else {
                                            "not farming"
                                        };
                                        format!(
                                            "{} {:.2}%{}, {}",
                                            replotting,
                                            progress,
                                            plotting_speed,
                                            farming,
                                        )
                                    },
                                }
                            },
                        },

                        gtk::Spinner {
                            start: (),
                        },
                    },

                    gtk::ProgressBar {
                        #[track = "self.changed_plotting_state()"]
                        set_fraction: progress as f64 / 100.0,
                    },
                },
                (_, PlottingState::Idle) => gtk::Box {
                    gtk::Label {
                        #[track = "self.changed_is_node_synced()"]
                        set_label: if self.is_node_synced {
                            "Farming"
                        } else {
                            "Waiting for node to sync"
                        },
                    }
                },
            },

            gtk::Box {
                #[track = "self.changed_farm_details() || self.changed_error()"]
                set_visible: self.farm_details && self.error.is_none(),

                self.sector_rows.clone(),
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
            Self::update_sector_tooltip(&sector, sector_index);
            sectors.push(sector);
        }

        let sector_rows = gtk::Box::new(gtk::Orientation::Vertical, 0);
        sectors.chunks(SECTORS_PER_ROW).for_each(|sectors| {
            let sector_row = gtk::Box::new(gtk::Orientation::Horizontal, 0);
            sector_rows.append(&sector_row);
            for sector in sectors {
                sector_row.append(sector);
            }
        });

        Self {
            path: init.farm.path,
            size: init.farm.size,
            auditing_time: SingleSumSMA::from_zero(Duration::ZERO),
            proving_time: SingleSumSMA::from_zero(Duration::ZERO),
            sector_plotting_time: SingleSumSMA::from_zero(Duration::ZERO),
            last_sector_plotted: None,
            plotting_state: PlottingState::Idle,
            is_node_synced: false,
            sector_rows,
            sectors: HashMap::from_iter((SectorIndex::MIN..).zip(sectors)),
            non_fatal_farming_error: None,
            farm_details: false,
            encoding_sectors: 0,
            plotting_paused: init.plotting_paused,
            error: None,
            tracker: u16::MAX,
        }
    }

    fn update(&mut self, input: Self::Input, _sender: FactorySender<Self>) {
        // Reset changes
        self.reset();

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
                        self.set_plotting_state(PlottingState::Plotting {
                            kind: if replotting {
                                PlottingKind::Replotting
                            } else {
                                PlottingKind::Initial
                            },
                            progress,
                        });

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
                        *self.get_mut_encoding_sectors() += 1;
                        self.update_sector_state(sector_index, SectorState::Encoding);
                    }
                    SectorPlottingDetails::Encoded(_) => {
                        *self.get_mut_encoding_sectors() -= 1;
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

                            self.set_plotting_state(PlottingState::Idle);
                        }

                        self.update_sector_state(sector_index, SectorState::Plotted);
                        self.sector_plotting_time.add_sample(time);
                    }
                    SectorPlottingDetails::Error(_error) => {
                        // TODO: treat sector as expired for now, in future with plotting retries
                        //  this might need to change
                        self.update_sector_state(sector_index, SectorState::Expired);
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
                    self.get_mut_auditing_time()
                        .add_sample(auditing_details.time);
                }
                FarmingNotification::Proving(proving_details) => {
                    self.get_mut_proving_time().add_sample(proving_details.time);
                }
                FarmingNotification::NonFatalError(error) => {
                    self.get_mut_non_fatal_farming_error().replace(error);
                }
            },
            FarmWidgetInput::PausePlotting(plotting_paused) => {
                self.set_plotting_paused(plotting_paused);
            }
            FarmWidgetInput::OpenFarmFolder => {
                if let Err(error) = open::that_detached(&self.path) {
                    error!(%error, path = %self.path.display(), "Failed to open farm folder");
                }
            }
            FarmWidgetInput::NodeSynced(synced) => {
                self.set_is_node_synced(synced);
            }
            FarmWidgetInput::ToggleFarmDetails => {
                self.set_farm_details(!self.farm_details);
            }
            FarmWidgetInput::Error { error } => {
                self.get_mut_error().replace(error);
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

            Self::update_sector_tooltip(sector, sector_index);
        }
    }

    fn remove_sector_state(&self, sector_index: SectorIndex, sector_state: SectorState) {
        if let Some(sector) = self.sectors.get(&sector_index) {
            sector.remove_css_class(sector_state.css_class());

            Self::update_sector_tooltip(sector, sector_index);
        }
    }

    fn update_sector_tooltip(sector: &gtk::Box, sector_index: SectorIndex) {
        if sector.has_css_class(SectorState::Downloading.css_class()) {
            sector.set_tooltip_text(Some(&format!("Sector {sector_index}: downloading")));
        } else if sector.has_css_class(SectorState::Encoding.css_class()) {
            sector.set_tooltip_text(Some(&format!("Sector {sector_index}: encoding")));
        } else if sector.has_css_class(SectorState::Writing.css_class()) {
            sector.set_tooltip_text(Some(&format!("Sector {sector_index}: writing")));
        } else if sector.has_css_class(SectorState::Expired.css_class()) {
            sector.set_tooltip_text(Some(&format!(
                "Sector {sector_index}: expired, waiting to be replotted"
            )));
        } else if sector.has_css_class(SectorState::AboutToExpire.css_class()) {
            sector.set_tooltip_text(Some(&format!(
                "Sector {sector_index}: about to expire, waiting to be replotted"
            )));
        } else if sector.has_css_class(SectorState::Plotted.css_class()) {
            sector.set_tooltip_text(Some(&format!("Sector {sector_index}: up to date")));
        } else {
            sector.set_tooltip_text(Some(&format!(
                "Sector {sector_index}: waiting to be plotted"
            )));
        }
    }
}
