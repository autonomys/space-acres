use crate::frontend::translations::{AsDefaultStr, T};
use bytesize::ByteSize;
use gtk::prelude::*;
use relm4::prelude::*;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, PartialEq)]
pub enum MigrationStatus {
    /// Waiting for migration to be started
    Idle,
    Copying {
        percentage: f64,
    },
    DeletingSource,
    UpdatingConfig,
    Verifying,
    Completed,
    Failed {
        error: String,
    },
    Restarting,
}

impl MigrationStatus {
    fn message(&self) -> String {
        match self {
            MigrationStatus::Idle => String::new(),
            MigrationStatus::Copying { percentage } => T
                .node_migration_status_copying(percentage.round() as i64)
                .to_string(),
            MigrationStatus::DeletingSource => {
                T.node_migration_status_deleting_source().to_string()
            }
            MigrationStatus::UpdatingConfig => {
                T.node_migration_status_updating_config().to_string()
            }
            MigrationStatus::Verifying => T.node_migration_status_verifying().to_string(),
            MigrationStatus::Completed => T.node_migration_status_completed().to_string(),
            MigrationStatus::Failed { error } => {
                T.node_migration_status_failed(error.clone()).to_string()
            }
            MigrationStatus::Restarting => T.node_migration_status_restarting().to_string(),
        }
    }

    fn progress_fraction(&self) -> f64 {
        match self {
            MigrationStatus::Idle => 0.0,
            MigrationStatus::Copying { percentage } => 0.05 + (percentage / 100.0) * 0.80,
            MigrationStatus::DeletingSource => 0.87,
            MigrationStatus::UpdatingConfig => 0.90,
            MigrationStatus::Verifying => 0.95,
            MigrationStatus::Completed => 1.0,
            MigrationStatus::Failed { .. } => 0.0,
            MigrationStatus::Restarting => 0.98,
        }
    }

    fn is_error(&self) -> bool {
        matches!(self, MigrationStatus::Failed { .. })
    }

    fn is_idle(&self) -> bool {
        matches!(self, MigrationStatus::Idle)
    }
}

#[derive(Debug)]
pub enum MigrationInput {
    Start {
        source: PathBuf,
        destination: PathBuf,
        snap_sync: bool,
    },
}

#[derive(Debug, Clone)]
pub enum MigrationOutput {
    Completed { new_node_path: PathBuf },
    Failed { error: String },
    RestartRequested,
}

#[tracker::track]
#[derive(Debug)]
pub struct MigrationView {
    source: PathBuf,
    destination: PathBuf,
    status: MigrationStatus,
}

#[relm4::component(pub)]
impl Component for MigrationView {
    type Init = ();
    type Input = MigrationInput;
    type Output = MigrationOutput;
    type CommandOutput = MigrationStatus;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 20,
            set_margin_all: 40,
            set_halign: gtk::Align::Center,
            set_valign: gtk::Align::Center,
            set_vexpand: true,

            gtk::Label {
                add_css_class: "title-1",
                set_label: &T.node_migration_dialog_title(),
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                set_spacing: 10,
                set_width_request: 400,
                #[track = "model.changed_status()"]
                set_visible: !model.status.is_idle(),

                gtk::Label {
                    set_halign: gtk::Align::Start,
                    #[track = "model.changed_status()"]
                    set_label: &model.status.message(),
                    #[track = "model.changed_status()"]
                    set_css_classes: if model.status.is_error() { &["error-label"] } else { &[] },
                },

                gtk::ProgressBar {
                    #[track = "model.changed_status()"]
                    set_fraction: model.status.progress_fraction(),
                    set_show_text: false,
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_halign: gtk::Align::Start,

                    gtk::Label {
                        set_label: &T.node_migration_source_label(),
                        add_css_class: "dim-label",
                    },

                    gtk::Label {
                        #[track = "model.changed_source()"]
                        set_label: &model.source.display().to_string(),
                        set_ellipsize: gtk::pango::EllipsizeMode::Middle,
                        set_max_width_chars: 40,
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Horizontal,
                    set_spacing: 10,
                    set_halign: gtk::Align::Start,

                    gtk::Label {
                        set_label: &T.node_migration_destination_label(),
                        add_css_class: "dim-label",
                    },

                    gtk::Label {
                        #[track = "model.changed_destination()"]
                        set_label: &model.destination.display().to_string(),
                        set_ellipsize: gtk::pango::EllipsizeMode::Middle,
                        set_max_width_chars: 40,
                    },
                },
            },

            // Show a spinner while waiting for migration to start (idle state)
            gtk::Spinner {
                start: (),
                set_size_request: (50, 50),
                #[track = "model.changed_status()"]
                set_visible: model.status.is_idle(),
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            source: PathBuf::new(),
            destination: PathBuf::new(),
            status: MigrationStatus::Idle,
            tracker: u8::MAX,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        self.reset();

        match input {
            MigrationInput::Start {
                source,
                destination,
                snap_sync,
            } => {
                debug!(?source, ?destination, snap_sync, "Starting migration");

                self.set_source(source.clone());
                self.set_destination(destination.clone());
                self.set_status(MigrationStatus::Copying { percentage: 0.0 });

                // Start the migration process
                sender.command(move |out_sender, shutdown_receiver| async move {
                    Self::perform_migration(
                        source,
                        destination,
                        snap_sync,
                        out_sender,
                        shutdown_receiver,
                    )
                    .await;
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        status: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.reset();

        match &status {
            MigrationStatus::Completed => {
                let _ = sender.output(MigrationOutput::Completed {
                    new_node_path: self.destination.clone(),
                });
            }
            MigrationStatus::Failed { error } => {
                let _ = sender.output(MigrationOutput::Failed {
                    error: error.clone(),
                });
            }
            MigrationStatus::Restarting => {
                let _ = sender.output(MigrationOutput::RestartRequested);
            }
            _ => {}
        }

        self.set_status(status);
    }
}

use super::NODE_DATA_DIRS;

impl MigrationView {
    /// Delete only known node data from the given path, leaving other files untouched.
    async fn delete_node_data(path: &std::path::Path) -> Result<(), std::io::Error> {
        for dir_name in NODE_DATA_DIRS {
            let dir_path = path.join(dir_name);
            if tokio::fs::try_exists(&dir_path).await.unwrap_or(false) {
                tokio::fs::remove_dir_all(&dir_path).await?;
            }
        }
        Ok(())
    }

    /// Try to remove the directory if it's empty after node data deletion.
    async fn try_cleanup_empty_dir(path: &std::path::Path) {
        // Check if directory is empty
        let is_empty = match tokio::fs::read_dir(path).await {
            Ok(mut entries) => entries.next_entry().await.ok().flatten().is_none(),
            Err(_) => false,
        };

        if is_empty {
            if let Err(error) = tokio::fs::remove_dir(path).await {
                warn!(%error, ?path, "Failed to remove empty source directory");
            } else {
                info!(?path, "Removed empty source directory after migration");
            }
        }
    }

    /// Calculate total size of known node data directories.
    async fn calculate_node_data_size(path: &std::path::Path) -> Result<u64, std::io::Error> {
        let mut total_size = 0u64;
        for dir_name in NODE_DATA_DIRS {
            let dir_path = path.join(dir_name);
            if tokio::fs::try_exists(&dir_path).await.unwrap_or(false) {
                total_size += Self::calculate_dir_size(&dir_path).await?;
            }
        }
        Ok(total_size)
    }

    async fn perform_migration(
        source: PathBuf,
        destination: PathBuf,
        snap_sync: bool,
        sender: relm4::Sender<MigrationStatus>,
        shutdown_receiver: relm4::ShutdownReceiver,
    ) {
        shutdown_receiver
            .register(async move {
                info!(
                    ?source,
                    ?destination,
                    snap_sync,
                    "Starting node database migration"
                );

                if snap_sync {
                    // For snap sync, delete only known node data and let the node re-sync
                    let _ = sender.send(MigrationStatus::DeletingSource);

                    if let Err(error) = Self::delete_node_data(&source).await {
                        error!(%error, "Failed to delete node data for snap sync");
                        let _ = sender.send(MigrationStatus::Failed {
                            error: error.to_string(),
                        });
                        return;
                    }

                    // Create the destination directory if different from source
                    if source != destination {
                        // Try to cleanup empty source directory
                        Self::try_cleanup_empty_dir(&source).await;

                        if let Err(error) = tokio::fs::create_dir_all(&destination).await {
                            error!(%error, "Failed to create destination directory");
                            let _ = sender.send(MigrationStatus::Failed {
                                error: error.to_string(),
                            });
                            return;
                        }
                    }
                } else {
                    // Full copy migration - only copy known node data
                    let _ = sender.send(MigrationStatus::Copying { percentage: 0.0 });

                    // Calculate total size of node data for progress tracking
                    let total_size = match Self::calculate_node_data_size(&source).await {
                        Ok(size) => size,
                        Err(error) => {
                            error!(%error, "Failed to calculate source size");
                            let _ = sender.send(MigrationStatus::Failed {
                                error: error.to_string(),
                            });
                            return;
                        }
                    };

                    debug!(total_size, "Source directory size calculated");

                    // Create destination directory
                    if let Err(error) = tokio::fs::create_dir_all(&destination).await {
                        error!(%error, "Failed to create destination directory");
                        let _ = sender.send(MigrationStatus::Failed {
                            error: error.to_string(),
                        });
                        return;
                    }

                    // Copy only known node data with progress tracking
                    let sender_clone = sender.clone();
                    let source_clone = source.clone();
                    let destination_clone = destination.clone();

                    let copy_result = tokio::task::spawn_blocking(move || {
                        Self::copy_node_data_with_progress(
                            &source_clone,
                            &destination_clone,
                            total_size,
                            sender_clone,
                        )
                    })
                    .await;

                    match copy_result {
                        Ok(Ok(())) => {
                            debug!("Directory copy completed successfully");
                        }
                        Ok(Err(error)) => {
                            error!(%error, "Failed to copy directory");
                            let _ = sender.send(MigrationStatus::Failed {
                                error: error.to_string(),
                            });
                            return;
                        }
                        Err(error) => {
                            error!(%error, "Copy task panicked");
                            let _ = sender.send(MigrationStatus::Failed {
                                error: error.to_string(),
                            });
                            return;
                        }
                    }

                    // Verify the copy
                    let _ = sender.send(MigrationStatus::Verifying);

                    let dest_size = match Self::calculate_node_data_size(&destination).await {
                        Ok(size) => size,
                        Err(error) => {
                            warn!(%error, "Failed to verify destination size, continuing anyway");
                            total_size // Assume success if we can't verify
                        }
                    };

                    // Allow some tolerance for filesystem differences
                    if dest_size < total_size * 95 / 100 {
                        let _ = sender.send(MigrationStatus::Failed {
                            error: format!(
                                "Copy verification failed: destination size {} < source size {}",
                                ByteSize::b(dest_size).to_string_as(true),
                                ByteSize::b(total_size).to_string_as(true)
                            ),
                        });
                        return;
                    }

                    // Delete only known node data from source, leaving other files untouched
                    let _ = sender.send(MigrationStatus::DeletingSource);

                    if let Err(error) = Self::delete_node_data(&source).await {
                        warn!(%error, "Failed to delete source node data, but migration was successful");
                        // Don't fail the migration, the copy was successful
                    }

                    // Try to cleanup empty source directory
                    Self::try_cleanup_empty_dir(&source).await;
                }

                // Update config
                let _ = sender.send(MigrationStatus::UpdatingConfig);
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;

                // Mark as completed and request restart
                let _ = sender.send(MigrationStatus::Completed);
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;

                let _ = sender.send(MigrationStatus::Restarting);
            })
            .drop_on_shutdown()
            .await;
    }

    async fn calculate_dir_size(path: &std::path::Path) -> Result<u64, std::io::Error> {
        let path = path.to_path_buf();
        tokio::task::spawn_blocking(move || {
            fs_extra::dir::get_size(&path).map_err(|e| std::io::Error::other(e.to_string()))
        })
        .await
        .map_err(|e| std::io::Error::other(e.to_string()))?
    }

    /// Copy only known node data (db/, network/) with progress tracking.
    fn copy_node_data_with_progress(
        source: &std::path::Path,
        destination: &std::path::Path,
        total_size: u64,
        sender: relm4::Sender<MigrationStatus>,
    ) -> Result<(), std::io::Error> {
        use std::sync::atomic::{AtomicU64, Ordering};

        let copied_bytes = Arc::new(AtomicU64::new(0));
        let copied_bytes_clone = copied_bytes.clone();
        let sender_clone = sender.clone();

        // Spawn a thread to periodically send progress updates
        let progress_handle = std::thread::spawn(move || {
            loop {
                let copied = copied_bytes_clone.load(Ordering::Relaxed);
                let percentage = if total_size > 0 {
                    (copied as f64 / total_size as f64) * 100.0
                } else {
                    0.0
                };

                let _ = sender_clone.send(MigrationStatus::Copying { percentage });

                if copied >= total_size {
                    break;
                }

                std::thread::sleep(std::time::Duration::from_millis(500));
            }
        });

        let dir_options = fs_extra::dir::CopyOptions {
            overwrite: true,
            skip_exist: false,
            copy_inside: false,
            content_only: false,
            ..Default::default()
        };

        let copied_bytes_for_closure = copied_bytes.clone();
        let mut accumulated_bytes = 0u64;

        // Copy db and network directories
        for dir_name in NODE_DATA_DIRS {
            let src_dir = source.join(dir_name);
            if src_dir.exists() {
                let result = fs_extra::dir::copy_with_progress(
                    &src_dir,
                    destination,
                    &dir_options,
                    |progress| {
                        copied_bytes_for_closure
                            .store(accumulated_bytes + progress.copied_bytes, Ordering::Relaxed);
                        fs_extra::dir::TransitProcessResult::ContinueOrAbort
                    },
                );

                if let Err(e) = result {
                    copied_bytes.store(total_size, Ordering::Relaxed);
                    let _ = progress_handle.join();
                    return Err(std::io::Error::other(e.to_string()));
                }

                // Update accumulated bytes after successful copy
                if let Ok(size) = fs_extra::dir::get_size(&src_dir) {
                    accumulated_bytes += size;
                    copied_bytes.store(accumulated_bytes, Ordering::Relaxed);
                }
            }
        }

        // Signal completion to the progress thread
        copied_bytes.store(total_size, Ordering::Relaxed);
        let _ = progress_handle.join();

        Ok(())
    }
}
