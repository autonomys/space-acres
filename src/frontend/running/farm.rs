use crate::backend::farmer::DiskFarm;
use crate::frontend::translations::{AsDefaultStr, T};
use crate::frontend::NotificationExt;
use crate::icon_names;
use bytesize::ByteSize;
use gtk::prelude::*;
use notify_rust::Notification;
use relm4::prelude::*;
use simple_moving_average::{SingleSumSMA, SMA};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use subspace_core_primitives::SectorIndex;
use subspace_farmer::farm::{
    FarmingError, FarmingNotification, ProvingResult, SectorExpirationDetails,
    SectorPlottingDetails, SectorUpdate,
};
use tracing::{error, warn};

const INVALID_SCORE_VALUE: f64 = -1.0;
/// Experimentally found number that is good for default window size to not have horizontal scroll
const SECTORS_PER_ROW: usize = 108;
/// Number of samples over which to track auditing time, 1 minute in slots
const AUDITING_TIME_TRACKING_WINDOW: usize = 60;
/// 500ms auditing time is excellent, anything larger will result in auditing performance indicator decrease
const EXCELLENT_AUDITING_TIME: Duration = Duration::from_millis(500);
/// Number of samples over which to track proving time
const PROVING_TIME_TRACKING_WINDOW: usize = 10;
/// 1800ms proving time is excellent, anything larger will result in proving performance indicator decrease
const EXCELLENT_PROVING_TIME: Duration = Duration::from_millis(1800);
/// Number of samples over which to track sector plotting time
const SECTOR_PLOTTING_TIME_TRACKING_WINDOW: usize = 10;

fn format_size(bytes: u64) -> String {
    ByteSize::b(bytes).to_string_as(bytes % ByteSize::mb(1).as_u64() != 0)
}

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
    pub(super) farm: DiskFarm,
    pub(super) total_sectors: SectorIndex,
    pub(super) plotted_total_sectors: SectorIndex,
    pub(super) plotting_paused: bool,
    pub(super) slot_duration: Duration,
    pub(super) block_authoring_delay: Duration,
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
    #[do_not_track]
    auditing_time: SingleSumSMA<Duration, u32, AUDITING_TIME_TRACKING_WINDOW>,
    auditing_time_average: Duration,
    /// 0.0..=1.0
    auditing_time_score: f64,
    #[do_not_track]
    proving_time: SingleSumSMA<Duration, u32, PROVING_TIME_TRACKING_WINDOW>,
    proving_time_average: Duration,
    /// 0.0..=1.0
    proving_time_score: f64,
    /// (success, total)
    proving_result: (u64, u64),
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
    slot_duration: Duration,
    block_authoring_delay: Duration,
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
                    set_cursor_from_name: Some("pointer"),
                    set_halign: gtk::Align::Start,
                    set_has_frame: false,
                    set_tooltip: &T.running_farmer_farm_tooltip(),

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
                            set_icon_name: Some(icon_names::WARNING),
                        }
                    },
                    None => {
                        gtk::Box {
                            set_halign: gtk::Align::End,
                            set_hexpand: true,
                            set_margin_top: 5,

                            gtk::Box {
                                set_spacing: 10,
                                #[track = "self.changed_is_node_synced()"]
                                set_visible: self.is_node_synced,

                                gtk::Box {
                                    #[track = "self.changed_proving_result() || self.changed_auditing_time_score() || self.changed_proving_time_score()"]
                                    set_css_classes: &[match self.farm_score() {
                                        ..=0.4 => "error-label",
                                        ..=0.8 => "warning-label",
                                        _ => "success-label",
                                    }],
                                    set_spacing: 5,
                                    #[track = "self.changed_proving_result()"]
                                    set_tooltip: T
                                        .running_farmer_farm_reward_signatures_tooltip(
                                            self.proving_result.0,
                                            self.proving_result.1
                                        )
                                        .as_str(),

                                    gtk::Label {
                                        #[track = "self.changed_proving_result()"]
                                        set_label: &format!("{}/{}", self.proving_result.0, self.proving_result.1),
                                    },

                                    gtk::Image {
                                        #[track = "self.changed_proving_result() || self.changed_auditing_time_score() || self.changed_proving_time_score()"]
                                        set_icon_name: Some(match self.farm_score() {
                                            ..=0.4 => icon_names::SPEEDOMETER4,
                                            ..=0.8 => icon_names::SPEEDOMETER3,
                                            _ => icon_names::SPEEDOMETER2,
                                        }),
                                    },
                                },

                                gtk::Box {
                                    set_spacing: 5,
                                    #[track = "self.changed_auditing_time_average()"]
                                    set_tooltip: T
                                        .running_farmer_farm_auditing_performance_tooltip(
                                            self.auditing_time_average.as_secs_f32(),
                                            self.slot_duration.as_secs_f32()
                                        )
                                        .as_str(),
                                    #[track = "self.changed_farm_details() || self.changed_auditing_time_score()"]
                                    set_visible: self.farm_details && self.auditing_time.get_num_samples() > 0,

                                    gtk::Image {
                                        set_icon_name: Some(icon_names::PUZZLE_PIECE),
                                    },

                                    gtk::LevelBar {
                                        add_css_class: "auditing-performance",
                                        #[track = "self.changed_auditing_time_score()"]
                                        set_value: self.auditing_time_score,
                                        set_width_request: 70,
                                    },
                                },

                                gtk::Box {
                                    set_spacing: 5,
                                    #[track = "self.changed_proving_time_average()"]
                                    set_tooltip: T
                                        .running_farmer_farm_proving_performance_tooltip(
                                            self.proving_time_average.as_secs_f32(),
                                            self.block_authoring_delay.as_secs_f32()
                                        )
                                        .as_str(),
                                    #[track = "self.changed_farm_details() || self.changed_proving_time_score()"]
                                    set_visible: self.farm_details && self.proving_time.get_num_samples() > 0,

                                    gtk::Image {
                                        set_icon_name: Some(icon_names::PROCESSOR),
                                    },

                                    gtk::LevelBar {
                                        add_css_class: "proving-performance",
                                        #[track = "self.changed_proving_time_score()"]
                                        set_value: self.proving_time_score,
                                        set_width_request: 70,
                                    },
                                },

                                gtk::Image {
                                    set_icon_name: Some(icon_names::WARNING),
                                    #[track = "self.changed_non_fatal_farming_error()"]
                                    set_tooltip: {
                                        let last_error = self.non_fatal_farming_error
                                            .as_ref()
                                            .map(|error| error.to_string())
                                            .unwrap_or_default();

                                        T
                                            .running_farmer_farm_non_fatal_error_tooltip(last_error)
                                            .as_str()
                                    },
                                    #[track = "self.changed_non_fatal_farming_error()"]
                                    set_visible: self.non_fatal_farming_error.is_some(),
                                },
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
                        set_label: T.running_farmer_farm_crashed(error.to_string()).as_str(),
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
                            set_label: {
                                let pausing_state = if self.plotting_paused {
                                    if self.encoding_sectors > 0 {
                                        "pausing"
                                    } else {
                                        "paused"
                                    }
                                } else {
                                    "no"
                                };
                                let plotting_speed = if self.sector_plotting_time.get_num_samples() > 0 {
                                     T
                                        .running_farmer_farm_plotting_speed(
                                            self.sector_plotting_time.get_average().as_secs_f32() / 60.0,
                                            3600.0 / self.sector_plotting_time.get_average().as_secs_f32()
                                        )
                                        .as_str()
                                        .to_string()
                                } else {
                                    String::new()
                                };
                                let farming = if self.is_node_synced {
                                    "yes"
                                } else {
                                    "no"
                                };

                                match kind {
                                    PlottingKind::Initial => T.running_farmer_farm_plotting_initial(
                                        pausing_state,
                                        progress,
                                        plotting_speed,
                                        farming,
                                    ),
                                    PlottingKind::Replotting => T.running_farmer_farm_replotting(
                                        pausing_state,
                                        progress,
                                        plotting_speed,
                                        farming,
                                    ),
                                }
                                    .as_str()
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
                        set_label: &if self.is_node_synced {
                            T.running_farmer_farm_farming()
                        } else {
                            T.running_farmer_farm_waiting_for_node_to_sync()
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
                .tooltip_text(T.running_farmer_farm_sector(sector_index).as_str())
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
            path: init.farm.directory,
            size: format_size(init.farm.allocated_space),
            auditing_time: SingleSumSMA::from_zero(Duration::ZERO),
            auditing_time_average: Duration::ZERO,
            auditing_time_score: INVALID_SCORE_VALUE,
            proving_time: SingleSumSMA::from_zero(Duration::ZERO),
            proving_time_average: Duration::ZERO,
            proving_time_score: INVALID_SCORE_VALUE,
            proving_result: (0, 0),
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
            slot_duration: init.slot_duration,
            block_authoring_delay: init.block_authoring_delay,
            error: None,
            tracker: u32::MAX,
        }
    }

    fn update(&mut self, input: Self::Input, sender: FactorySender<Self>) {
        // Reset changes
        self.reset();

        self.process_input(input, sender);
    }
}

impl FarmWidget {
    fn process_input(&mut self, input: FarmWidgetInput, sender: FactorySender<Self>) {
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
                    self.auditing_time.add_sample(auditing_details.time);

                    let average_time = self.auditing_time.get_average();
                    let slot_time_fraction_remaining =
                        1.0 - average_time.as_secs_f64() / self.slot_duration.as_secs_f64();
                    let excellent_time_fraction_remaining = 1.0
                        - EXCELLENT_AUDITING_TIME.as_secs_f64() / self.slot_duration.as_secs_f64();
                    let score = (slot_time_fraction_remaining / excellent_time_fraction_remaining)
                        .clamp(0.0, 1.0);
                    // Round to 5% precision
                    let score_rounded = (score * 20.0).round() / 20.0;
                    // Only update when there is a meaningful change, not all the time (accounting
                    // for hysteresis would be nice, but will make code significantly more complex)
                    if self.auditing_time_score != score_rounded {
                        self.set_auditing_time_score(score_rounded);
                        self.set_auditing_time_average(average_time);
                    }
                }
                FarmingNotification::Proving(proving_details) => {
                    self.proving_time.add_sample(proving_details.time);

                    let average_time = self.proving_time.get_average();
                    let slot_time_fraction_remaining =
                        1.0 - average_time.as_secs_f64() / self.block_authoring_delay.as_secs_f64();
                    let excellent_time_fraction_remaining = 1.0
                        - EXCELLENT_PROVING_TIME.as_secs_f64()
                            / self.block_authoring_delay.as_secs_f64();
                    let score = (slot_time_fraction_remaining / excellent_time_fraction_remaining)
                        .clamp(0.0, 1.0);
                    // Round to 5% precision
                    let score_rounded = (score * 20.0).round() / 20.0;
                    // Only update when there is a meaningful change, not all the time (accounting
                    // for hysteresis would be nice, but will make code significantly more complex)
                    if self.proving_time_score != score_rounded {
                        self.set_proving_time_score(score_rounded);
                        self.set_proving_time_average(average_time);
                    }

                    let proving_result = self.get_mut_proving_result();
                    if matches!(proving_details.result, ProvingResult::Success) {
                        proving_result.0 += 1;
                    }
                    proving_result.1 += 1;
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
                sender.spawn_command(|_sender| {
                    let mut notification = Notification::new();
                    notification
                        .summary(&T.notification_farm_error())
                        .body(&T.notification_farm_error_body())
                        .with_typical_options();
                    #[cfg(all(unix, not(target_os = "macos")))]
                    notification.urgency(notify_rust::Urgency::Critical);
                    if let Err(error) = notification.show() {
                        warn!(%error, "Failed to show desktop notification");
                    }
                });

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
        let tooltip = if sector.has_css_class(SectorState::Downloading.css_class()) {
            T.running_farmer_farm_sector_downloading(sector_index)
        } else if sector.has_css_class(SectorState::Encoding.css_class()) {
            T.running_farmer_farm_sector_encoding(sector_index)
        } else if sector.has_css_class(SectorState::Writing.css_class()) {
            T.running_farmer_farm_sector_writing(sector_index)
        } else if sector.has_css_class(SectorState::Expired.css_class()) {
            T.running_farmer_farm_sector_expired(sector_index)
        } else if sector.has_css_class(SectorState::AboutToExpire.css_class()) {
            T.running_farmer_farm_sector_about_to_expire(sector_index)
        } else if sector.has_css_class(SectorState::Plotted.css_class()) {
            T.running_farmer_farm_sector_up_to_date(sector_index)
        } else {
            T.running_farmer_farm_sector_waiting_to_be_plotted(sector_index)
        };
        sector.set_tooltip_text(Some(tooltip.as_str()));
    }

    /// 0.0..=1.0
    fn farm_score(&self) -> f64 {
        // 95% success rate with signing is good
        let proving_result_score = if self.proving_result.1 == 0 {
            1.0
        } else {
            match self.proving_result.0 as f64 / self.proving_result.1 as f64 {
                0.95.. => 1.0,
                0.9.. => 0.5,
                _ => 0.0,
            }
        };
        let auditing_time_score = if self.auditing_time_score == INVALID_SCORE_VALUE {
            1.0
        } else {
            self.auditing_time_score
        };
        let proving_time_score = if self.proving_time_score == INVALID_SCORE_VALUE {
            1.0
        } else {
            self.proving_time_score
        };

        (proving_result_score + auditing_time_score + proving_time_score) / 3.0
    }
}
