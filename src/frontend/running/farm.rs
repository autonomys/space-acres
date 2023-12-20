use crate::backend::config::Farm;
use crate::backend::farmer::{PlottingKind, PlottingState};
use gtk::prelude::*;
use relm4::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
pub(super) struct FarmWidgetInit {
    pub(super) initial_plotting_state: PlottingState,
    pub(super) farm: Farm,
}

#[derive(Debug, Copy, Clone)]
pub(super) enum FarmWidgetInput {
    PlottingStateUpdate(PlottingState),
    PieceCacheSynced(bool),
    NodeSynced(bool),
}

#[derive(Debug, Default)]
pub(super) struct FarmWidget {
    path: PathBuf,
    size: String,
    plotting_state: PlottingState,
    is_piece_cache_synced: bool,
    is_node_synced: bool,
}

#[relm4::factory(pub(super))]
impl FactoryComponent for FarmWidget {
    type Init = FarmWidgetInit;
    type Input = FarmWidgetInput;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type Index = usize;

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
                        set_tooltip: "Farming starts after initial plotting is complete",

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

    fn init_model(init: Self::Init, _index: &Self::Index, _sender: FactorySender<Self>) -> Self {
        Self {
            path: init.farm.path,
            size: init.farm.size,
            plotting_state: init.initial_plotting_state,
            is_piece_cache_synced: false,
            is_node_synced: false,
        }
    }

    fn update(&mut self, input: Self::Input, _sender: FactorySender<Self>) {
        self.process_input(input);
    }
}

impl FarmWidget {
    fn process_input(&mut self, input: FarmWidgetInput) {
        match input {
            FarmWidgetInput::PlottingStateUpdate(state) => {
                self.plotting_state = state;
            }
            FarmWidgetInput::PieceCacheSynced(synced) => {
                self.is_piece_cache_synced = synced;
            }
            FarmWidgetInput::NodeSynced(synced) => {
                self.is_node_synced = synced;
            }
        }
    }
}
