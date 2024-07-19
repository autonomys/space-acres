use crate::backend::config::Farm;
use crate::frontend::configuration::{IsValid, MaybeValid};
use bytesize::ByteSize;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4_icons::icon_name;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::warn;

// 2 GB
const MIN_FARM_SIZE: u64 = 1000 * 1000 * 1000 * 2;

#[derive(Debug)]
pub(super) struct FarmWidgetInit {
    pub(super) path: MaybeValid<PathBuf>,
    pub(super) size: MaybeValid<String>,
}

impl Default for FarmWidgetInit {
    fn default() -> Self {
        Self {
            path: MaybeValid::no(PathBuf::new()),
            size: MaybeValid::no(String::new()),
        }
    }
}

#[derive(Debug)]
pub(super) enum FarmWidgetInput {
    DirectorySelected(PathBuf),
    FarmSizeChanged(String),
}

#[derive(Debug)]
pub(super) enum FarmWidgetOutput {
    OpenDirectory(DynamicIndex),
    ValidityUpdate,
    Delete(DynamicIndex),
}

#[derive(Debug)]
pub(super) struct FarmWidget {
    // TODO: Track changes for dynamic index
    index: DynamicIndex,
    path: MaybeValid<PathBuf>,
    size: MaybeValid<String>,
    valid: bool,
}

#[relm4::factory(pub(super))]
impl FactoryComponent for FarmWidget {
    type Init = FarmWidgetInit;
    type Input = FarmWidgetInput;
    type Output = FarmWidgetOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::ListBox;

    view! {
        #[root]
        gtk::ListBoxRow {
            set_activatable: false,
            set_margin_bottom: 10,
            set_selectable: false,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,

                gtk::Label {
                    add_css_class: "heading",
                    set_halign: gtk::Align::Start,
                    #[watch]
                    set_label: &format!("Path to farm {} and its size", self.index.current_index()),
                },

                gtk::Box {
                    set_spacing: 10,

                    gtk::Box {
                        add_css_class: "linked",

                        gtk::Entry {
                            #[track = "self.path.changed_is_valid()"]
                            set_css_classes: if self.path.is_valid.yes() {
                                &["valid-input"]
                            } else {
                                &["invalid-input"]
                            },
                            set_can_focus: false,
                            set_editable: false,
                            set_hexpand: true,
                            set_placeholder_text: Some(
                                if cfg!(windows) {
                                    "Example: D:\\subspace-farm"
                                } else {
                                    "Example: /media/subspace-farm"
                                },
                            ),
                            set_primary_icon_name: Some(icon_name::SSD),
                            set_primary_icon_activatable: false,
                            set_primary_icon_sensitive: false,
                            #[track = "self.path.changed_is_valid()"]
                            set_secondary_icon_name: self.path.icon(),
                            set_secondary_icon_activatable: false,
                            set_secondary_icon_sensitive: false,
                            #[track = "self.path.changed_value()"]
                            set_text: self.path.display().to_string().as_str(),
                            set_tooltip_markup: Some(
                                "Absolute path where farm files will be stored, any \
                                SSD works, high endurance not necessary"
                            ),
                        },

                        gtk::Button {
                            connect_clicked[sender, index] => move |_| {
                                if sender.output(FarmWidgetOutput::OpenDirectory(index.clone())).is_err() {
                                    warn!("Can't send open directory output");
                                }
                            },
                            set_label: "Select",
                        },
                    },

                    gtk::Entry {
                        connect_activate[sender] => move |entry| {
                            sender.input(FarmWidgetInput::FarmSizeChanged(entry.text().into()));
                        },
                        connect_changed[sender] => move |entry| {
                            sender.input(FarmWidgetInput::FarmSizeChanged(entry.text().into()));
                        },
                        #[track = "self.size.changed_is_valid()"]
                        set_css_classes: if self.size.is_valid.yes() {
                            &["valid-input"]
                        } else {
                            &["invalid-input"]
                        },
                        set_placeholder_text: Some(
                            "Example: 4T, 2.5TB, 500GiB, etc.",
                        ),
                        set_primary_icon_name: Some(icon_name::SIZE_HORIZONTALLY),
                        set_primary_icon_activatable: false,
                        set_primary_icon_sensitive: false,
                        #[track = "self.size.changed_is_valid()"]
                        set_secondary_icon_name: self.size.icon(),
                        set_secondary_icon_activatable: false,
                        set_secondary_icon_sensitive: false,
                        #[track = "self.size.changed_value()"]
                        set_text: self.size.as_str(),
                        set_tooltip_markup: Some(
                            "Size of the farm in whichever units you prefer, any \
                            amount of space above 2 GB works"
                        ),
                    },

                    gtk::Button {
                        connect_clicked[sender, index] => move |_| {
                            if sender.output(FarmWidgetOutput::Delete(index.clone())).is_err() {
                                warn!("Can't send delete output");
                            }
                        },
                        set_icon_name: icon_name::CROSS,
                        set_tooltip: "Delete this farm",
                    },
                },
            },
        }
    }

    fn init_model(value: Self::Init, index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self {
            index: index.clone(),
            path: value.path,
            size: value.size,
            valid: false,
        }
    }

    fn update(&mut self, input: Self::Input, sender: FactorySender<Self>) {
        // Reset changes
        self.path.reset();
        self.size.reset();

        match input {
            FarmWidgetInput::DirectorySelected(path) => {
                self.path.set_is_valid(IsValid::Yes);
                self.path.value = path;
            }
            FarmWidgetInput::FarmSizeChanged(size) => {
                if ByteSize::from_str(&size)
                    .map(|size| size.as_u64() >= MIN_FARM_SIZE)
                    .unwrap_or_default()
                {
                    self.size.set_is_valid(IsValid::Yes);
                } else {
                    self.size.set_is_valid(IsValid::No);
                }
                self.size.value = size;
            }
        }

        let valid = self.valid();
        if self.valid != valid {
            self.valid = valid;

            // Send notification up that validity was updated, such that parent view can re-render
            // view if necessary
            if sender.output(FarmWidgetOutput::ValidityUpdate).is_err() {
                warn!("Can't send validity update output");
            }
        }
    }
}

impl FarmWidget {
    pub(super) fn valid(&self) -> bool {
        self.path.is_valid.yes() && self.size.is_valid.yes()
    }

    pub(super) fn farm(&self) -> Farm {
        Farm {
            path: PathBuf::clone(&self.path),
            size: String::clone(&self.size),
        }
    }
}
