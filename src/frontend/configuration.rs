mod farm;

use crate::backend::config::RawConfig;
use crate::frontend::configuration::farm::{
    FarmWidget, FarmWidgetInit, FarmWidgetInput, FarmWidgetOutput,
};
use gtk::prelude::*;
use relm4::factory::FactoryVecDeque;
use relm4::prelude::*;
use relm4_components::open_dialog::{
    OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings,
};
use relm4_icons::icon_name;
use std::ops::Deref;
use std::path::PathBuf;
use subspace_farmer::utils::ss58::parse_ss58_reward_address;
use tracing::{debug, warn};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum DirectoryKind {
    NodePath,
    FarmPath(DynamicIndex),
}

#[derive(Debug)]
pub enum ConfigurationInput {
    AddFarm,
    RewardAddressChanged(String),
    OpenDirectory(DirectoryKind),
    DirectorySelected(PathBuf),
    Delete(DynamicIndex),
    Reconfigure(RawConfig),
    Start,
    Cancel,
    Save,
    Ignore,
}

#[derive(Debug)]
pub enum ConfigurationOutput {
    StartWithNewConfig(RawConfig),
    ConfigUpdate(RawConfig),
    Close,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum MaybeValid<T> {
    Unknown(T),
    Valid(T),
    Invalid(T),
}

impl<T> Default for MaybeValid<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::Unknown(T::default())
    }
}

impl<T> Deref for MaybeValid<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let (MaybeValid::Unknown(inner) | MaybeValid::Valid(inner) | MaybeValid::Invalid(inner)) =
            self;

        inner
    }
}

impl<T> MaybeValid<T> {
    fn unknown(&self) -> bool {
        matches!(self, MaybeValid::Unknown(_))
    }

    fn valid(&self) -> bool {
        matches!(self, MaybeValid::Valid(_))
    }

    fn icon(&self) -> Option<&'static str> {
        match self {
            MaybeValid::Unknown(_) => None,
            MaybeValid::Valid(_) => Some(icon_name::CHECKMARK),
            MaybeValid::Invalid(_) => Some(icon_name::CROSS),
        }
    }
}

#[derive(Debug)]
pub struct ConfigurationView {
    reward_address: MaybeValid<String>,
    node_path: MaybeValid<PathBuf>,
    farms: FactoryVecDeque<FarmWidget>,
    pending_directory_selection: Option<DirectoryKind>,
    open_dialog: Controller<OpenDialog>,
    reconfiguration: bool,
}

#[relm4::component(pub)]
impl Component for ConfigurationView {
    type Init = gtk::Window;
    type Input = ConfigurationInput;
    type Output = ConfigurationOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            gtk::ScrolledWindow {
                set_vexpand: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 20,

                    gtk::ListBox {
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
                                    set_label: "Node path",
                                },

                                gtk::Box {
                                    add_css_class: "linked",

                                    gtk::Entry {
                                        set_can_focus: false,
                                        set_editable: false,
                                        set_hexpand: true,
                                        set_placeholder_text: Some(
                                            if cfg!(windows) {
                                                "D:\\subspace-node"
                                            } else {
                                                "/media/subspace-node"
                                            },
                                        ),
                                        set_primary_icon_name: Some(icon_name::SSD),
                                        set_primary_icon_activatable: false,
                                        set_primary_icon_sensitive: false,
                                        #[watch]
                                        set_secondary_icon_name: model.node_path.icon(),
                                        set_secondary_icon_activatable: false,
                                        set_secondary_icon_sensitive: false,
                                        #[watch]
                                        set_text: model.node_path.display().to_string().as_str(),
                                        set_tooltip_markup: Some(
                                            "Absolute path where node files will be stored, prepare to \
                                            dedicate at least 100 GiB of space for it, good quality SSD \
                                            recommended"
                                        ),
                                    },

                                    gtk::Button {
                                        connect_clicked => ConfigurationInput::OpenDirectory(
                                            DirectoryKind::NodePath
                                        ),
                                        set_label: "Select",
                                    },
                                },
                            },
                        },
                        gtk::ListBoxRow {
                            set_activatable: false,
                            set_selectable: false,

                            gtk::Box {
                                set_orientation: gtk::Orientation::Vertical,
                                set_spacing: 10,

                                gtk::Label {
                                    add_css_class: "heading",
                                    set_halign: gtk::Align::Start,
                                    set_label: "Rewards address",
                                },

                                gtk::Entry {
                                    connect_activate[sender] => move |entry| {
                                        sender.input(ConfigurationInput::RewardAddressChanged(
                                            entry.text().into()
                                        ));
                                    },
                                    connect_changed[sender] => move |entry| {
                                        sender.input(ConfigurationInput::RewardAddressChanged(
                                            entry.text().into()
                                        ));
                                    },
                                    set_placeholder_text: Some(
                                        "stB4S14whneyomiEa22Fu2PzVoibMB7n5PvBFUwafbCbRkC1K",
                                    ),
                                    set_primary_icon_name: Some(icon_name::WALLET2),
                                    set_primary_icon_activatable: false,
                                    set_primary_icon_sensitive: false,
                                    #[watch]
                                    set_secondary_icon_name: model.reward_address.icon(),
                                    set_secondary_icon_activatable: false,
                                    set_secondary_icon_sensitive: false,
                                    #[track = "model.reward_address.unknown()"]
                                    set_text: &model.reward_address,
                                    set_tooltip_markup: Some(
                                        "Use Subwallet or polkadot{.js} extension or any other \
                                        Substrate wallet to create it first (address for any Substrate \
                                        chain in SS58 format works)"
                                    ),
                                },
                            },
                        },
                    },

                    // TODO: This should be the same list box as above, but then farms will unfortunately render before
                    //  other fields
                    #[local_ref]
                    configuration_list_box -> gtk::ListBox {
                    },

                    gtk::Box {
                        gtk::Box {
                            set_halign: gtk::Align::Start,
                            set_hexpand: true,
                            set_spacing: 10,

                            gtk::Button {
                                connect_clicked => ConfigurationInput::AddFarm,

                                gtk::Label {
                                    set_label: "Add farm",
                                    set_margin_all: 10,
                                },
                            },
                        },

                        if model.reconfiguration {
                            gtk::Box {
                                set_halign: gtk::Align::End,
                                set_spacing: 10,

                                gtk::Button {
                                    connect_clicked => ConfigurationInput::Cancel,

                                    gtk::Label {
                                        set_label: "Cancel",
                                        set_margin_all: 10,
                                    },
                                },

                                gtk::Button {
                                    add_css_class: "suggested-action",
                                    connect_clicked => ConfigurationInput::Save,
                                    #[watch]
                                    set_sensitive: model.reward_address.valid()
                                        && model.node_path.valid()
                                        && !model.farms.is_empty()
                                        && model.farms.iter().all(FarmWidget::valid),

                                    gtk::Label {
                                        set_label: "Save",
                                        set_margin_all: 10,
                                    },
                                },
                            }
                        } else {
                            gtk::Box {
                                set_halign: gtk::Align::End,

                                gtk::Button {
                                    add_css_class: "suggested-action",
                                    connect_clicked => ConfigurationInput::Start,
                                    #[watch]
                                    set_sensitive:
                                        model.reward_address.valid()
                                            && model.node_path.valid()
                                            && !model.farms.is_empty()
                                            && model.farms.iter().all(FarmWidget::valid),

                                    gtk::Label {
                                        set_label: "Start",
                                        set_margin_all: 10,
                                    },
                                },
                            }
                        },
                    },
                },
            },
        }
    }

    fn init(
        parent_root: Self::Init,
        root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let open_dialog = OpenDialog::builder()
            .transient_for_native(&parent_root)
            .launch(OpenDialogSettings {
                folder_mode: true,
                accept_label: "Select".to_string(),
                ..OpenDialogSettings::default()
            })
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => ConfigurationInput::DirectorySelected(path),
                OpenDialogResponse::Cancel => ConfigurationInput::Ignore,
            });

        let mut farms = FactoryVecDeque::builder()
            .launch(gtk::ListBox::new())
            .forward(sender.input_sender(), |output| match output {
                FarmWidgetOutput::OpenDirectory(index) => {
                    ConfigurationInput::OpenDirectory(DirectoryKind::FarmPath(index))
                }
                FarmWidgetOutput::ValidityUpdate => ConfigurationInput::Ignore,
                FarmWidgetOutput::Delete(index) => ConfigurationInput::Delete(index),
            });

        farms.guard().push_back(FarmWidgetInit::default());

        let model = Self {
            reward_address: Default::default(),
            node_path: Default::default(),
            farms,
            pending_directory_selection: Default::default(),
            open_dialog,
            reconfiguration: false,
        };

        let configuration_list_box = model.farms.widget();
        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        self.process_input(input, sender);
    }
}

impl ConfigurationView {
    fn process_input(&mut self, input: ConfigurationInput, sender: ComponentSender<Self>) {
        match input {
            ConfigurationInput::AddFarm => {
                self.farms.guard().push_back(FarmWidgetInit::default());
            }
            ConfigurationInput::OpenDirectory(directory_kind) => {
                self.pending_directory_selection.replace(directory_kind);
                self.open_dialog.emit(OpenDialogMsg::Open);
            }
            ConfigurationInput::DirectorySelected(path) => {
                match self.pending_directory_selection.take() {
                    Some(DirectoryKind::NodePath) => {
                        self.node_path = MaybeValid::Valid(path);
                    }
                    Some(DirectoryKind::FarmPath(index)) => {
                        self.farms.send(
                            index.current_index(),
                            FarmWidgetInput::DirectorySelected(path),
                        );
                    }
                    None => {
                        warn!(
                            directory = %path.display(),
                            "Directory selected, but no pending selection found",
                        );
                    }
                }
            }
            ConfigurationInput::Delete(index) => {
                let mut farms = self.farms.guard();
                farms.remove(index.current_index());
                // Force re-rendering of all farms
                farms.iter_mut().for_each(|_| {
                    // Nothing
                });
            }
            ConfigurationInput::RewardAddressChanged(new_reward_address) => {
                let new_reward_address = new_reward_address.trim();
                self.reward_address = if parse_ss58_reward_address(new_reward_address).is_ok() {
                    MaybeValid::Valid(new_reward_address.to_string())
                } else {
                    MaybeValid::Invalid(new_reward_address.to_string())
                };
            }
            ConfigurationInput::Reconfigure(raw_config) => {
                // `Unknown` is a hack to make it actually render the first time
                self.reward_address = MaybeValid::Unknown(raw_config.reward_address().to_string());
                self.node_path = MaybeValid::Valid(raw_config.node_path().clone());
                {
                    let mut farms = self.farms.guard();
                    farms.clear();
                    for farm in raw_config.farms() {
                        farms.push_back(FarmWidgetInit {
                            path: MaybeValid::Valid(farm.path.clone()),
                            // `Unknown` is a hack to make it actually render the first time
                            size: MaybeValid::Unknown(farm.size.clone()),
                        });
                    }
                }
                self.reconfiguration = true;
            }
            ConfigurationInput::Start => {
                if sender
                    .output(ConfigurationOutput::StartWithNewConfig(
                        self.create_raw_config(),
                    ))
                    .is_err()
                {
                    debug!("Failed to send ConfigurationOutput::StartWithNewConfig");
                }
            }
            ConfigurationInput::Cancel => {
                if sender.output(ConfigurationOutput::Close).is_err() {
                    debug!("Failed to send ConfigurationOutput::Close");
                }
            }
            ConfigurationInput::Save => {
                if sender
                    .output(ConfigurationOutput::ConfigUpdate(self.create_raw_config()))
                    .is_err()
                {
                    debug!("Failed to send ConfigurationOutput::ConfigUpdate");
                }
            }
            ConfigurationInput::Ignore => {
                // Ignore
            }
        }
    }

    /// Create raw config from own state
    fn create_raw_config(&self) -> RawConfig {
        RawConfig::V0 {
            reward_address: String::clone(&self.reward_address),
            node_path: PathBuf::clone(&self.node_path),
            farms: self.farms.iter().map(FarmWidget::farm).collect(),
        }
    }
}
