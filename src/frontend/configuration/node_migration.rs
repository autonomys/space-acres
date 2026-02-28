use crate::frontend::NODE_FREE_SPACE_WARNING_THRESHOLD;
use crate::frontend::configuration::utils::{calculate_node_data_size, get_available_space};
use crate::frontend::translations::{AsDefaultStr, T};
use crate::icon_names::shipped as icon_names;
use bytesize::ByteSize;
use gtk::glib;
use gtk::prelude::*;
use relm4::prelude::*;
use relm4_components::open_dialog::{
    OpenDialog, OpenDialogMsg, OpenDialogResponse, OpenDialogSettings,
};
use std::path::PathBuf;
use tracing::warn;

/// Sync mode for fresh database synchronization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    /// Snap sync - downloads state snapshots for faster sync
    #[default]
    Snap,
    // Full, // Future: full sync validates all blocks from genesis
}

/// Migration mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MigrationMode {
    /// Copy the existing database to new location
    #[default]
    Migrate,
    /// Fresh sync from network at new location using specified sync mode
    FreshSync(SyncMode),
    /// Wipe and resync in the same location using specified sync mode
    ResetInPlace(SyncMode),
}

impl MigrationMode {
    /// Returns true if this mode uses snap sync
    fn uses_snap_sync(&self) -> bool {
        match self {
            MigrationMode::Migrate => false,
            MigrationMode::FreshSync(mode) | MigrationMode::ResetInPlace(mode) => {
                matches!(mode, SyncMode::Snap)
            }
        }
    }

    /// Returns true if this mode requires a destination path
    fn requires_destination(&self) -> bool {
        !matches!(self, MigrationMode::ResetInPlace(_))
    }

    /// Returns the label for this mode
    fn label(&self) -> String {
        match self {
            MigrationMode::Migrate => T.node_migration_mode_migrate().to_string(),
            MigrationMode::FreshSync(_) => T.node_migration_mode_fresh_sync().to_string(),
            MigrationMode::ResetInPlace(_) => T.node_migration_mode_reset().to_string(),
        }
    }

    /// Returns the explanation for this mode
    fn explanation(&self) -> String {
        match self {
            MigrationMode::Migrate => T.node_migration_mode_migrate_explanation().to_string(),
            MigrationMode::FreshSync(_) => {
                T.node_migration_mode_fresh_sync_explanation().to_string()
            }
            MigrationMode::ResetInPlace(_) => T.node_migration_mode_reset_explanation().to_string(),
        }
    }

    /// Returns the button label for starting this mode
    fn start_button_label(&self) -> String {
        match self {
            MigrationMode::Migrate | MigrationMode::FreshSync(_) => {
                T.node_migration_button_start().to_string()
            }
            MigrationMode::ResetInPlace(_) => T.node_migration_button_reset().to_string(),
        }
    }
}

#[derive(Debug)]
pub enum NodeMigrationInput {
    OpenDestinationDialog,
    DestinationSelected(PathBuf),
    MigrationModeChanged(MigrationMode),
    StartMigration,
    Cancel,
    Ignore,
    /// Update source size after async calculation
    SourceSizeCalculated(u64),
    /// Update destination free space after async calculation
    DestinationSpaceCalculated(u64),
    /// Update whether non-node data was detected in source
    NonNodeDataDetected(bool),
}

#[derive(Debug, Clone)]
pub enum NodeMigrationOutput {
    StartMigration {
        source: PathBuf,
        destination: PathBuf,
        snap_sync: bool,
    },
    Cancel,
}

pub struct NodeMigrationInit {
    pub source_path: PathBuf,
    pub parent_window: gtk::Window,
    /// Initial migration mode to pre-select
    pub initial_mode: MigrationMode,
}

#[tracker::track]
#[derive(Debug)]
pub struct NodeMigrationDialog {
    source_path: PathBuf,
    source_size: Option<u64>,
    destination_path: Option<PathBuf>,
    destination_free_space: Option<u64>,
    destination_valid: bool,
    migration_mode: MigrationMode,
    /// True if non-node data was detected in the source directory
    has_non_node_data: bool,
    #[do_not_track]
    open_dialog: Controller<OpenDialog>,
}

impl NodeMigrationDialog {
    /// Returns the minimum required space at destination for migration
    fn required_destination_space(&self) -> Option<u64> {
        self.source_size
    }

    /// Check if destination has enough space for migration
    fn has_sufficient_space(&self) -> bool {
        if self.migration_mode.uses_snap_sync() {
            // Snap sync only needs NODE_FREE_SPACE_WARNING_THRESHOLD
            if let Some(dest_space) = self.destination_free_space {
                return dest_space >= NODE_FREE_SPACE_WARNING_THRESHOLD;
            }
            return false;
        }

        if let (Some(required), Some(dest_space)) = (
            self.required_destination_space(),
            self.destination_free_space,
        ) {
            dest_space >= required
        } else {
            false
        }
    }

    /// Returns true if destination is fully valid (exists, writable, and has enough space)
    fn is_destination_fully_valid(&self) -> bool {
        if self.destination_path.is_none() {
            return false;
        }

        if !self.destination_valid {
            return false;
        }

        // Check space if we have the calculation
        if self.destination_free_space.is_some() {
            return self.has_sufficient_space();
        }

        // Still calculating, consider invalid until we know
        false
    }

    /// Returns the icon for the destination field
    fn destination_icon(&self) -> Option<&'static str> {
        self.destination_path.as_ref()?;

        if self.is_destination_fully_valid() {
            Some(icon_names::CHECKMARK)
        } else {
            Some(icon_names::CROSS_SMALL)
        }
    }

    fn can_start_migration(&self) -> bool {
        // Reset in place doesn't need a destination
        if !self.migration_mode.requires_destination() {
            return true;
        }

        self.is_destination_fully_valid()
    }

    fn has_insufficient_space(&self) -> bool {
        if self.migration_mode.uses_snap_sync() {
            return false;
        }

        if let (Some(required), Some(dest_space)) = (
            self.required_destination_space(),
            self.destination_free_space,
        ) {
            dest_space < required
        } else {
            false
        }
    }
}

#[relm4::component(pub async)]
impl AsyncComponent for NodeMigrationDialog {
    type Init = NodeMigrationInit;
    type Input = NodeMigrationInput;
    type Output = NodeMigrationOutput;
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 20,
            set_margin_all: 20,

            // Source section
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,

                gtk::Label {
                    add_css_class: "heading",
                    set_halign: gtk::Align::Start,
                    set_label: &T.node_migration_source_label(),
                },

                gtk::Entry {
                    set_editable: false,
                    set_can_focus: false,
                    set_hexpand: true,
                    set_text: model.source_path.display().to_string().as_str(),
                    set_primary_icon_name: Some(icon_names::SSD),
                    set_primary_icon_activatable: false,
                    set_primary_icon_sensitive: false,
                },

                gtk::Label {
                    add_css_class: "dim-label",
                    add_css_class: "caption",
                    set_halign: gtk::Align::Start,
                    #[track = "model.changed_source_size()"]
                    set_label: &match model.source_size {
                        Some(size) => T.configuration_node_size(ByteSize::b(size).to_string_as(true)).to_string(),
                        None => T.configuration_node_size("...".to_string()).to_string(),
                    },
                },

                // Warning for non-node data
                gtk::Label {
                    add_css_class: "warning-label",
                    add_css_class: "caption",
                    set_halign: gtk::Align::Start,
                    set_label: &T.node_migration_non_node_data_warning(),
                    #[track = "model.changed_has_non_node_data()"]
                    set_visible: model.has_non_node_data,
                },
            },

            // Migration mode selection (radio buttons)
            gtk::Frame {
                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 8,
                    set_margin_all: 10,

                    #[name = "radio_migrate"]
                    gtk::CheckButton {
                        set_label: Some(&MigrationMode::Migrate.label()),
                        #[track = "model.changed_migration_mode()"]
                        set_active: model.migration_mode == MigrationMode::Migrate,
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(NodeMigrationInput::MigrationModeChanged(MigrationMode::Migrate));
                            }
                        },
                    },

                    #[name = "radio_fresh_sync"]
                    gtk::CheckButton {
                        set_group: Some(&radio_migrate),
                        set_label: Some(&MigrationMode::FreshSync(SyncMode::default()).label()),
                        #[track = "model.changed_migration_mode()"]
                        set_active: matches!(model.migration_mode, MigrationMode::FreshSync(_)),
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(NodeMigrationInput::MigrationModeChanged(MigrationMode::FreshSync(SyncMode::default())));
                            }
                        },
                    },

                    #[name = "radio_reset"]
                    gtk::CheckButton {
                        set_group: Some(&radio_migrate),
                        set_label: Some(&MigrationMode::ResetInPlace(SyncMode::default()).label()),
                        #[track = "model.changed_migration_mode()"]
                        set_active: matches!(model.migration_mode, MigrationMode::ResetInPlace(_)),
                        connect_toggled[sender] => move |btn| {
                            if btn.is_active() {
                                sender.input(NodeMigrationInput::MigrationModeChanged(MigrationMode::ResetInPlace(SyncMode::default())));
                            }
                        },
                    },
                },
            },

            // Destination section (hidden when reset in place is selected)
            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 5,
                #[track = "model.changed_migration_mode()"]
                set_visible: model.migration_mode.requires_destination(),

                gtk::Label {
                    add_css_class: "heading",
                    set_halign: gtk::Align::Start,
                    set_label: &T.node_migration_destination_label(),
                },

                gtk::Box {
                    add_css_class: "linked",

                    gtk::Entry {
                        set_editable: false,
                        set_can_focus: false,
                        set_hexpand: true,
                        #[track = "model.changed_destination_valid() || model.changed_destination_free_space() || model.changed_source_size() || model.changed_migration_mode()"]
                        set_css_classes: if model.destination_path.is_some() {
                            if model.is_destination_fully_valid() {
                                &["valid-input"]
                            } else {
                                &["invalid-input"]
                            }
                        } else {
                            &[]
                        },
                        set_placeholder_text: Some(&T.node_migration_destination_placeholder()),
                        #[track = "model.changed_destination_path()"]
                        set_text: model.destination_path.as_ref()
                            .map(|p| p.display().to_string())
                            .unwrap_or_default()
                            .as_str(),
                        set_primary_icon_name: Some(icon_names::SSD),
                        set_primary_icon_activatable: false,
                        set_primary_icon_sensitive: false,
                        #[track = "model.changed_destination_valid() || model.changed_destination_free_space() || model.changed_source_size() || model.changed_migration_mode()"]
                        set_secondary_icon_name: model.destination_icon(),
                        set_secondary_icon_activatable: false,
                        set_secondary_icon_sensitive: false,
                    },

                    gtk::Button {
                        connect_clicked => NodeMigrationInput::OpenDestinationDialog,
                        set_label: &T.configuration_node_path_button_select(),
                    },
                },

                gtk::Label {
                    add_css_class: "dim-label",
                    add_css_class: "caption",
                    set_halign: gtk::Align::Start,
                    #[track = "model.changed_destination_free_space()"]
                    set_visible: model.destination_path.is_some(),
                    #[track = "model.changed_destination_free_space()"]
                    set_label: &match model.destination_free_space {
                        Some(space) => T.node_migration_destination_free_space(ByteSize::b(space).to_string_as(true)).to_string(),
                        None => T.node_migration_destination_free_space("...".to_string()).to_string(),
                    },
                },

                // Insufficient space warning
                gtk::Label {
                    add_css_class: "error-label",
                    set_halign: gtk::Align::Start,
                    set_label: &T.node_migration_insufficient_space_warning(),
                    #[track = "model.changed_destination_free_space() || model.changed_source_size() || model.changed_migration_mode()"]
                    set_visible: model.has_insufficient_space(),
                },
            },

            // Dynamic description based on selected mode
            gtk::Label {
                add_css_class: "caption",
                set_halign: gtk::Align::Start,
                set_hexpand: true,
                set_wrap: true,
                set_natural_wrap_mode: gtk::NaturalWrapMode::None,
                #[track = "model.changed_migration_mode()"]
                set_css_classes: if model.migration_mode == MigrationMode::Migrate {
                    &["caption", "dim-label"]
                } else {
                    &["caption", "warning-label"]
                },
                #[track = "model.changed_migration_mode()"]
                set_label: &model.migration_mode.explanation(),
            },

            // Buttons
            gtk::Box {
                set_halign: gtk::Align::End,
                set_spacing: 10,
                set_margin_top: 10,

                gtk::Button {
                    connect_clicked => NodeMigrationInput::Cancel,
                    set_label: &T.node_migration_button_cancel(),
                },

                gtk::Button {
                    add_css_class: "suggested-action",
                    connect_clicked => NodeMigrationInput::StartMigration,
                    #[track = "model.changed_destination_valid() || model.changed_destination_free_space() || model.changed_source_size() || model.changed_migration_mode()"]
                    set_sensitive: model.can_start_migration(),
                    #[track = "model.changed_migration_mode()"]
                    set_label: &model.migration_mode.start_button_label(),
                },
            },
        }
    }

    async fn init(
        init: Self::Init,
        root: Self::Root,
        sender: AsyncComponentSender<Self>,
    ) -> AsyncComponentParts<Self> {
        let open_dialog = OpenDialog::builder()
            .transient_for_native(&init.parent_window)
            .launch(OpenDialogSettings {
                folder_mode: true,
                accept_label: T.configuration_dialog_button_select().to_string(),
                cancel_label: T.configuration_dialog_button_cancel().to_string(),
                ..OpenDialogSettings::default()
            })
            .forward(sender.input_sender(), |response| match response {
                OpenDialogResponse::Accept(path) => NodeMigrationInput::DestinationSelected(path),
                OpenDialogResponse::Cancel => NodeMigrationInput::Ignore,
            });

        let source_path = init.source_path.clone();

        let model = Self {
            source_path: init.source_path,
            source_size: None,
            destination_path: None,
            destination_free_space: None,
            destination_valid: false,
            migration_mode: init.initial_mode,
            has_non_node_data: false,
            open_dialog,
            tracker: u8::MAX,
        };

        let widgets = view_output!();

        // Calculate source size asynchronously
        let sender_clone = sender.clone();
        let source_path_clone = source_path.clone();
        glib::spawn_future_local(async move {
            if let Ok(size) = calculate_node_data_size(source_path_clone).await {
                sender_clone.input(NodeMigrationInput::SourceSizeCalculated(size));
            }
        });

        // Check for non-node data asynchronously
        let sender_clone = sender.clone();
        let source_path_clone = source_path.clone();
        glib::spawn_future_local(async move {
            let has_non_node = check_for_non_node_data(&source_path_clone).await;
            sender_clone.input(NodeMigrationInput::NonNodeDataDetected(has_non_node));
        });

        AsyncComponentParts { model, widgets }
    }

    async fn update(
        &mut self,
        input: Self::Input,
        sender: AsyncComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        match input {
            NodeMigrationInput::OpenDestinationDialog => {
                self.open_dialog.emit(OpenDialogMsg::Open);
            }
            NodeMigrationInput::DestinationSelected(path) => {
                // Validate: must be different from source and writable
                let is_valid = path != self.source_path
                    && path.exists()
                    && super::utils::is_directory_writable(path.clone()).await;

                self.set_destination_path(Some(path.clone()));
                self.set_destination_valid(is_valid);
                self.set_destination_free_space(None);

                if is_valid {
                    // Calculate free space asynchronously
                    let sender_clone = sender.clone();
                    glib::spawn_future_local(async move {
                        if let Ok(space) = get_available_space(path).await {
                            sender_clone
                                .input(NodeMigrationInput::DestinationSpaceCalculated(space));
                        }
                    });
                }
            }
            NodeMigrationInput::MigrationModeChanged(mode) => {
                self.set_migration_mode(mode);
            }
            NodeMigrationInput::StartMigration => {
                // For reset in place, use source as destination
                let destination = if !self.migration_mode.requires_destination() {
                    self.source_path.clone()
                } else {
                    match self.destination_path.clone() {
                        Some(dest) => dest,
                        None => return,
                    }
                };

                if sender
                    .output(NodeMigrationOutput::StartMigration {
                        source: self.source_path.clone(),
                        destination,
                        snap_sync: self.migration_mode.uses_snap_sync(),
                    })
                    .is_err()
                {
                    warn!("Failed to send StartMigration output");
                }
            }
            NodeMigrationInput::Cancel => {
                if sender.output(NodeMigrationOutput::Cancel).is_err() {
                    warn!("Failed to send Cancel output");
                }
            }
            NodeMigrationInput::Ignore => {
                // Do nothing
            }
            NodeMigrationInput::SourceSizeCalculated(size) => {
                self.set_source_size(Some(size));
            }
            NodeMigrationInput::DestinationSpaceCalculated(space) => {
                self.set_destination_free_space(Some(space));
            }
            NodeMigrationInput::NonNodeDataDetected(has_non_node) => {
                self.set_has_non_node_data(has_non_node);
            }
        }
    }
}

use crate::frontend::NODE_DATA_DIRS;

/// Check if the source directory contains any files/directories other than known node data
async fn check_for_non_node_data(path: &std::path::Path) -> bool {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || {
        if let Ok(entries) = std::fs::read_dir(&path) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let name = file_name.to_string_lossy();
                if !NODE_DATA_DIRS.contains(&name.as_ref()) {
                    return true;
                }
            }
        }
        false
    })
    .await
    .unwrap_or(false)
}
