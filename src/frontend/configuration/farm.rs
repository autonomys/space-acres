use crate::backend::config::{Farm, MIN_FARM_SIZE};
use crate::frontend::configuration::MaybeValid;
use bytesize::ByteSize;
use gtk::prelude::*;
use std::fmt;
// TODO: Remove import once in prelude: https://github.com/Relm4/Relm4/issues/662
use relm4::factory::AsyncFactoryComponent;
use relm4::prelude::*;
// TODO: Remove import once in prelude: https://github.com/Relm4/Relm4/issues/662
use crate::frontend::configuration::utils::is_directory_writable;
use crate::frontend::translations::{AsDefaultStr, T};
use relm4::AsyncFactorySender;
use relm4_components::simple_combo_box::SimpleComboBox;
use relm4_icons::icon_name;
use std::path::PathBuf;
use std::str::FromStr;
use tracing::warn;

fn is_fixed_size_valid(size: &str) -> bool {
    ByteSize::from_str(size)
        .map(|size| size.as_u64() >= MIN_FARM_SIZE)
        .unwrap_or_default()
}

fn is_free_percentage_size_valid(size: &str) -> bool {
    size.ends_with("%")
        && f32::from_str(size.trim_end_matches('%'))
            .ok()
            .map(|size| size > 0.0 && size <= 100.0)
            .unwrap_or_default()
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SizeKind {
    Fixed,
    FreePercentage,
}

impl fmt::Display for SizeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&match self {
            Self::Fixed => T.configuration_farm_size_kind_fixed(),
            Self::FreePercentage => T.configuration_farm_size_kind_free_percentage(),
        })
    }
}

impl SizeKind {
    fn all() -> [SizeKind; std::mem::variant_count::<SizeKind>()] {
        [Self::Fixed, Self::FreePercentage]
    }
}

#[derive(Debug)]
pub(super) struct FarmWidgetInit {
    pub(super) path: PathBuf,
    pub(super) size: String,
}

impl Default for FarmWidgetInit {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            size: String::new(),
        }
    }
}

#[derive(Debug)]
pub(super) enum FarmWidgetInput {
    DirectorySelected(PathBuf),
    SizeKindChanged(usize),
    FarmFixedSizeChanged(String),
    FarmFreePercentageSizeChanged(String),
}

#[derive(Debug)]
pub(super) enum FarmWidgetOutput {
    OpenDirectory(DynamicIndex),
    ValidityUpdate,
    Delete(DynamicIndex),
}

#[tracker::track]
#[derive(Debug)]
pub(super) struct FarmWidget {
    // TODO: Track changes for dynamic index
    #[do_not_track]
    index: DynamicIndex,
    #[do_not_track]
    path: MaybeValid<PathBuf>,
    size_kind: SizeKind,
    #[do_not_track]
    size_kind_selector: Controller<SimpleComboBox<SizeKind>>,
    #[do_not_track]
    fixed_size: MaybeValid<String>,
    /// 0.0%..=100.0%
    #[do_not_track]
    free_percentage_size: MaybeValid<String>,
}

#[relm4::factory(pub(super) async)]
impl AsyncFactoryComponent for FarmWidget {
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
                    set_label: T.configuration_farm(self.index.current_index()).as_str(),
                },

                gtk::Box {
                    set_spacing: 10,

                    gtk::Box {
                        add_css_class: "linked",

                        gtk::Entry {
                            #[track = "self.path.changed_is_valid()"]
                            set_css_classes: if self.path.is_valid {
                                &["valid-input"]
                            } else {
                                &["invalid-input"]
                            },
                            set_can_focus: false,
                            set_editable: false,
                            set_hexpand: true,
                            set_placeholder_text: Some(
                                T
                                    .configuration_farm_path_placeholder(
                                        if cfg!(windows) {
                                            "D:\\subspace-farm"
                                        } else if cfg!(target_os = "macos") {
                                            "/Volumes/Subspace/subspace-farm"
                                        } else {
                                            "/media/subspace-farm"
                                        },
                                    )
                                    .as_str(),
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
                                &T.configuration_farm_path_tooltip()
                            ),
                        },

                        gtk::Button {
                            connect_clicked[sender, index] => move |_| {
                                if sender.output(FarmWidgetOutput::OpenDirectory(index.clone())).is_err() {
                                    warn!("Can't send open directory output");
                                }
                            },
                            set_label: &T.configuration_farm_path_button_select(),
                        },
                    },

                    gtk::Box {
                        add_css_class: "linked",

                        self.size_kind_selector.widget().clone(),

                        gtk::Entry {
                            connect_activate[sender] => move |entry| {
                                sender.input(FarmWidgetInput::FarmFixedSizeChanged(entry.text().into()));
                            },
                            connect_changed[sender] => move |entry| {
                                sender.input(FarmWidgetInput::FarmFixedSizeChanged(entry.text().into()));
                            },
                            #[track = "self.fixed_size.changed_is_valid()"]
                            set_css_classes: if self.fixed_size.is_valid {
                                &["valid-input"]
                            } else {
                                &["invalid-input"]
                            },
                            set_placeholder_text: Some(
                                &T.configuration_farm_fixed_size_placeholder(),
                            ),
                            set_primary_icon_name: Some(icon_name::SIZE_HORIZONTALLY),
                            set_primary_icon_activatable: false,
                            set_primary_icon_sensitive: false,
                            #[track = "self.fixed_size.changed_is_valid()"]
                            set_secondary_icon_name: self.fixed_size.icon(),
                            set_secondary_icon_activatable: false,
                            set_secondary_icon_sensitive: false,
                            #[track = "self.fixed_size.changed_value()"]
                            set_text: self.fixed_size.as_str(),
                            set_tooltip_markup: Some(
                                &T.configuration_farm_fixed_size_tooltip()
                            ),
                            #[track = "self.changed_size_kind()"]
                            set_visible: self.size_kind == SizeKind::Fixed,
                        },

                        gtk::Entry {
                            connect_activate[sender] => move |entry| {
                                sender.input(FarmWidgetInput::FarmFreePercentageSizeChanged(entry.text().into()));
                            },
                            connect_changed[sender] => move |entry| {
                                sender.input(FarmWidgetInput::FarmFreePercentageSizeChanged(entry.text().into()));
                            },
                            #[track = "self.free_percentage_size.changed_is_valid()"]
                            set_css_classes: if self.free_percentage_size.is_valid {
                                &["valid-input"]
                            } else {
                                &["invalid-input"]
                            },
                            set_placeholder_text: Some(
                                &T.configuration_farm_free_percentage_size_placeholder(),
                            ),
                            set_primary_icon_name: Some(icon_name::SIZE_HORIZONTALLY),
                            set_primary_icon_activatable: false,
                            set_primary_icon_sensitive: false,
                            #[track = "self.free_percentage_size.changed_is_valid()"]
                            set_secondary_icon_name: self.free_percentage_size.icon(),
                            set_secondary_icon_activatable: false,
                            set_secondary_icon_sensitive: false,
                            #[track = "self.free_percentage_size.changed_value()"]
                            set_text: self.free_percentage_size.as_str(),
                            set_tooltip_markup: Some(
                                &T.configuration_farm_free_percentage_size_tooltip()
                            ),
                            #[track = "self.changed_size_kind()"]
                            set_visible: self.size_kind == SizeKind::FreePercentage,
                        },
                    },

                    gtk::Button {
                        connect_clicked[sender, index] => move |_| {
                            if sender.output(FarmWidgetOutput::Delete(index.clone())).is_err() {
                                warn!("Can't send delete output");
                            }
                        },
                        set_icon_name: icon_name::CROSS,
                        set_tooltip: &T.configuration_farm_delete(),
                    },
                },

                gtk::Label {
                    add_css_class: "error-label",
                    set_halign: gtk::Align::Start,
                    set_label: &T.configuration_farm_path_error_doesnt_exist_or_write_permissions(),
                    #[track = "self.path.changed_is_valid()"]
                    set_visible: !self.path.is_valid && self.path.value != PathBuf::new(),
                },
            },
        }
    }

    async fn init_model(
        value: Self::Init,
        index: &DynamicIndex,
        sender: AsyncFactorySender<Self>,
    ) -> Self {
        let (size_kind, fixed_size, free_percentage_size) = if value.size.ends_with('%') {
            (
                SizeKind::FreePercentage,
                MaybeValid::no(String::new()),
                if is_free_percentage_size_valid(&value.size) {
                    MaybeValid::yes(value.size)
                } else {
                    MaybeValid::no(value.size)
                },
            )
        } else {
            (
                SizeKind::Fixed,
                if is_fixed_size_valid(&value.size) {
                    MaybeValid::yes(value.size)
                } else {
                    MaybeValid::no(value.size)
                },
                MaybeValid::no(String::new()),
            )
        };

        let size_kind_selector = SimpleComboBox::builder()
            .launch({
                let variants = SizeKind::all().to_vec();
                let active_index = variants
                    .iter()
                    .position(|candidate| *candidate == size_kind);

                SimpleComboBox {
                    variants,
                    active_index,
                }
            })
            .forward(sender.input_sender(), FarmWidgetInput::SizeKindChanged);

        let instance = Self {
            index: index.clone(),
            path: if is_directory_writable(value.path.clone()).await {
                MaybeValid::yes(value.path)
            } else {
                MaybeValid::no(value.path)
            },
            size_kind,
            size_kind_selector,
            fixed_size,
            free_percentage_size,
            tracker: u8::MAX,
        };

        // Send notification up that validity was updated, such that parent view can re-render
        // view if necessary, this is necessary due to async initialization of the model
        if sender.output(FarmWidgetOutput::ValidityUpdate).is_err() {
            warn!("Can't send validity update output");
        }

        instance
    }

    async fn update(&mut self, input: Self::Input, sender: AsyncFactorySender<Self>) {
        // Reset changes
        self.reset();
        self.path.reset();
        self.fixed_size.reset();
        self.free_percentage_size.reset();

        let was_valid = self.valid();

        match input {
            FarmWidgetInput::DirectorySelected(path) => {
                self.path = if is_directory_writable(path.clone()).await {
                    MaybeValid::yes(path)
                } else {
                    MaybeValid::no(path)
                };
            }
            FarmWidgetInput::SizeKindChanged(index) => self.set_size_kind(SizeKind::all()[index]),
            FarmWidgetInput::FarmFixedSizeChanged(size) => {
                self.fixed_size.set_is_valid(is_fixed_size_valid(&size));
                self.fixed_size.value = size;
            }
            FarmWidgetInput::FarmFreePercentageSizeChanged(size) => {
                self.free_percentage_size
                    .set_is_valid(is_free_percentage_size_valid(&size));
                self.free_percentage_size.value = size;
            }
        }

        let is_valid = self.valid();
        if was_valid != is_valid {
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
        if !self.path.is_valid {
            return false;
        }

        match self.size_kind {
            SizeKind::Fixed => self.fixed_size.is_valid,
            SizeKind::FreePercentage => self.free_percentage_size.is_valid,
        }
    }

    pub(super) fn farm(&self) -> Farm {
        Farm {
            path: PathBuf::clone(&self.path),
            size: String::clone(match self.size_kind {
                SizeKind::Fixed => &self.fixed_size,
                SizeKind::FreePercentage => &self.free_percentage_size,
            }),
        }
    }
}
