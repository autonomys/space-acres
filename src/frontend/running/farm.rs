use crate::backend::config::Farm;
use gtk::prelude::*;
use relm4::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use subspace_core_primitives::SectorIndex;
use subspace_farmer::single_disk_farm::{
    SectorExpirationDetails, SectorPlottingDetails, SectorUpdate,
};

/// Experimentally found number that is good for default window size to not have horizontal scroll
/// and allows for sectors to not occupy too much vertical space
const MIN_SECTORS_PER_ROW: u32 = 108;
/// Effectively no limit
const MAX_SECTORS_PER_ROW: u32 = 100_000;

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
        /// Plotting/replotting speed in sectors/s
        speed: Option<f32>,
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
    PieceCacheSynced(bool),
    NodeSynced(bool),
}

#[derive(Debug)]
pub(super) struct FarmWidget {
    path: PathBuf,
    size: String,
    last_sector_plotted: Option<SectorIndex>,
    plotting_state: PlottingState,
    is_piece_cache_synced: bool,
    is_node_synced: bool,
    farm_during_initial_plotting: bool,
    sectors_grid: gtk::GridView,
    sectors: HashMap<SectorIndex, gtk::Box>,
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

            gtk::Label {
                set_halign: gtk::Align::Start,
                set_label: &format!("{} [{}]:", self.path.display(), self.size),
            },

            #[transition = "SlideUpDown"]
            match (self.plotting_state, self.is_piece_cache_synced, self.is_node_synced) {
                (_, false, _) => gtk::Label {
                    set_halign: gtk::Align::Start,
                    set_label: "Waiting for piece cache sync",
                },
                (PlottingState::Plotting { kind, progress, speed }, _, true) => gtk::Box {
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
                                let kind = match kind {
                                    PlottingKind::Initial => {
                                        if self.farm_during_initial_plotting {
                                            "Initial plotting, farming"
                                        } else {
                                            "Initial plotting, not farming"
                                        }
                                    },
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
                (PlottingState::Idle, _, true) => gtk::Box {
                    gtk::Label {
                        set_label: "Farming",
                    }
                },
                _ => gtk::Box {
                    gtk::Label {
                        set_label: "Waiting for node to sync first",
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
            last_sector_plotted: None,
            plotting_state: PlottingState::Idle,
            is_piece_cache_synced: false,
            is_node_synced: false,
            farm_during_initial_plotting: init.farm_during_initial_plotting,
            sectors_grid,
            sectors: HashMap::from_iter((SectorIndex::MIN..).zip(sectors)),
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
                            speed: None,
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
                    SectorPlottingDetails::Finished { .. } => {
                        if self.last_sector_plotted == Some(sector_index) {
                            self.last_sector_plotted.take();

                            self.plotting_state = PlottingState::Idle;
                        }

                        self.update_sector_state(sector_index, SectorState::Plotted);
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
            FarmWidgetInput::PieceCacheSynced(synced) => {
                self.is_piece_cache_synced = synced;
            }
            FarmWidgetInput::NodeSynced(synced) => {
                self.is_node_synced = synced;
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
