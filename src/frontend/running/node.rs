use crate::backend::node::{ChainInfo, SyncKind, SyncState};
use crate::backend::NodeNotification;
use bytesize::ByteSize;
use gtk::prelude::*;
use parking_lot::Mutex;
use relm4::prelude::*;
use relm4::{Sender, ShutdownReceiver};
use relm4_icons::icon_name;
use simple_moving_average::{SingleSumSMA, SMA};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};
use subspace_core_primitives::BlockNumber;
use tracing::error;

/// Maximum blocks to store in the import queue.
// HACK: This constant comes from Substrate's sync, but it is not public in there
const MAX_IMPORTING_BLOCKS: BlockNumber = 2048;
/// How frequently to check for free disk space
const FREE_DISK_SPACE_CHECK_INTERVAL: Duration = Duration::from_secs(5);
/// Free disk space below which warning must be shown
const FREE_DISK_SPACE_CHECK_WARNING_THRESHOLD: u64 = 10 * 1024 * 1024 * 1024;
/// Number of samples over which to track block import time, 1 minute in slots
const BLOCK_IMPORT_TIME_TRACKING_WINDOW: usize = 1000;

#[derive(Debug)]
pub enum NodeInput {
    Initialize {
        best_block_number: BlockNumber,
        chain_info: ChainInfo,
        node_path: PathBuf,
    },
    NodeNotification(NodeNotification),
    OpenNodeFolder(),
}

#[derive(Debug)]
pub enum NodeCommandOutput {
    FreeDiskSpace(ByteSize),
}

#[derive(Debug)]
pub struct NodeView {
    best_block_number: BlockNumber,
    sync_state: SyncState,
    free_disk_space: Option<ByteSize>,
    chain_name: String,
    node_path: Arc<Mutex<PathBuf>>,
    block_import_time: SingleSumSMA<Duration, u32, BLOCK_IMPORT_TIME_TRACKING_WINDOW>,
    last_block_import_time: Option<Instant>,
}

#[relm4::component(pub)]
impl Component for NodeView {
    type Init = ();
    type Input = NodeInput;
    type Output = ();
    type CommandOutput = NodeCommandOutput;

    view! {
        #[root]
        gtk::Box {
            set_height_request: 100,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            gtk::Box {
                gtk::Button {
                    add_css_class: "heading",
                    add_css_class : "folder-button",
                    set_tooltip: "Click to open in file manager",
                    set_has_frame: false,
                    set_use_underline: false,
                    set_halign: gtk::Align::Start,
                    #[watch]
                    set_label: &model.chain_name,
                    connect_clicked[sender] => move |_| {
                        sender.input(NodeInput::OpenNodeFolder())
                    }
                },


                gtk::Box {
                    set_halign: gtk::Align::End,
                    set_hexpand: true,
                    set_spacing: 10,

                    gtk::Box {
                        set_spacing: 10,
                        #[watch]
                        set_tooltip: &format!(
                            "Free disk space: {} remaining",
                            model.free_disk_space
                                .map(|bytes| bytes.to_string_as(true))
                                .unwrap_or_default()
                        ),
                        #[watch]
                        set_visible: model.free_disk_space
                            .map(|bytes| bytes.as_u64() <= FREE_DISK_SPACE_CHECK_WARNING_THRESHOLD)
                            .unwrap_or_default(),

                        gtk::Image {
                            set_icon_name: Some(icon_name::SSD),
                        },

                        gtk::LevelBar {
                            add_css_class: "free-disk-space",
                            set_min_value: 0.1,
                            #[watch]
                            set_value: {
                                let free_space = model.free_disk_space
                                    .map(|bytes| bytes.as_u64())
                                    .unwrap_or_default();
                                free_space as f64 / FREE_DISK_SPACE_CHECK_WARNING_THRESHOLD as f64
                            },
                            set_width_request: 100,
                        },
                    },
                },
            },

            #[transition = "SlideUpDown"]
            match model.sync_state {
                SyncState::Unknown => gtk::Box {
                    gtk::Label {
                        #[watch]
                        set_label: &format!(
                            "Connecting to the network, best block #{}",
                            model.best_block_number
                        ),
                    }
                },
                SyncState::Syncing { kind, target } => gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 10,

                    gtk::Box {
                        set_spacing: 5,

                        gtk::Label {
                            set_halign: gtk::Align::Start,

                            #[watch]
                            set_label: &{
                                let kind = match kind {
                                    SyncKind::Dsn => "Syncing from DSN",
                                    SyncKind::Regular => "Regular sync",
                                };
                                let sync_speed = if model.block_import_time.get_num_samples() > 0 {
                                     let mut sync_speed = format!(
                                        ", {:.2} blocks/s",
                                        1.0 / model.block_import_time.get_average().as_secs_f32(),
                                    );

                                    if target > model.best_block_number {
                                        let time_remaining = (target - model.best_block_number) * model.block_import_time.get_average();
                                        if time_remaining > Duration::from_secs(3600) {
                                            sync_speed += &format!(
                                                " (~{:.2} hours remaining)",
                                                time_remaining.as_secs_f32() / 3600.0
                                            );
                                        } else if time_remaining > Duration::from_secs(60) {
                                            sync_speed += &format!(
                                                " (~{:.2} minutes remaining)",
                                                time_remaining.as_secs_f32() / 60.0
                                            );
                                        } else {
                                            sync_speed += &format!(
                                                " (~{:.2} seconds remaining)",
                                                time_remaining.as_secs_f32()
                                            );
                                        }
                                    }

                                    sync_speed
                                } else {
                                    String::new()
                                };

                                format!(
                                    "{} #{}/{}{}",
                                    kind,
                                    model.best_block_number,
                                    target,
                                    sync_speed,
                                )
                            },
                        },

                        gtk::Spinner {
                            start: (),
                        },
                    },

                    gtk::ProgressBar {
                        #[watch]
                        set_fraction: model.best_block_number as f64 / target as f64,
                    },
                },
                SyncState::Idle => gtk::Box {
                    gtk::Label {
                        #[watch]
                        set_label: &format!("Synced, best block #{}", model.best_block_number),
                    }
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let node_path = Arc::<Mutex<PathBuf>>::default();
        let model = Self {
            best_block_number: 0,
            sync_state: SyncState::default(),
            free_disk_space: None,
            chain_name: String::new(),
            node_path: node_path.clone(),
            block_import_time: SingleSumSMA::from_zero(Duration::ZERO),
            last_block_import_time: None,
        };

        let widgets = view_output!();

        sender.command(move |sender, shutdown_receiver| async move {
            Self::check_free_disk_space(sender, shutdown_receiver, node_path).await;
        });

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        self.process_input(input);
    }

    fn update_cmd(
        &mut self,
        input: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        self.process_command(input);
    }
}

impl NodeView {
    fn process_input(&mut self, input: NodeInput) {
        match input {
            NodeInput::Initialize {
                best_block_number,
                chain_info,
                node_path,
            } => {
                self.best_block_number = best_block_number;
                self.chain_name = format!(
                    "{} consensus node",
                    chain_info
                        .chain_name
                        .strip_prefix("Subspace ")
                        .unwrap_or(&chain_info.chain_name)
                );
                *self.node_path.lock() = node_path;
            }
            NodeInput::NodeNotification(node_notification) => match node_notification {
                NodeNotification::SyncStateUpdate(mut new_sync_state) => {
                    if let SyncState::Syncing {
                        target: new_target, ..
                    } = &mut new_sync_state
                    {
                        *new_target = (*new_target).max(self.best_block_number);

                        // Ensure target is never below current block
                        if let SyncState::Syncing {
                            target: old_target, ..
                        } = &self.sync_state
                        {
                            // If old target was within `MAX_IMPORTING_BLOCKS` from new target, keep old target
                            if old_target
                                .checked_sub(*new_target)
                                .map(|diff| diff <= MAX_IMPORTING_BLOCKS)
                                .unwrap_or_default()
                            {
                                *new_target = *old_target;
                            }
                        }
                    }
                    // Reset block import time on transition to sync
                    if self.sync_state.is_synced() && !new_sync_state.is_synced() {
                        self.block_import_time = SingleSumSMA::from_zero(Duration::ZERO);
                        self.last_block_import_time.take();
                    }
                    self.sync_state = new_sync_state;
                }
                NodeNotification::BlockImported(imported_block) => {
                    self.best_block_number = imported_block.number;
                    // Ensure target is never below current block
                    if let SyncState::Syncing { target, .. } = &mut self.sync_state {
                        *target = (*target).max(self.best_block_number);
                    }

                    if let Some(last_block_import_time) =
                        self.last_block_import_time.replace(Instant::now())
                    {
                        self.block_import_time
                            .add_sample(last_block_import_time.elapsed());
                    }
                }
            },
            NodeInput::OpenNodeFolder() => {
                let node_path = self.node_path.lock().clone();
                open::that_detached(node_path.as_os_str()).unwrap();
            }
        }
    }

    fn process_command(&mut self, command_output: NodeCommandOutput) {
        match command_output {
            NodeCommandOutput::FreeDiskSpace(bytes) => {
                self.free_disk_space.replace(bytes);
            }
        }
    }

    async fn check_free_disk_space(
        sender: Sender<NodeCommandOutput>,
        shutdown_receiver: ShutdownReceiver,
        node_path: Arc<Mutex<PathBuf>>,
    ) {
        shutdown_receiver
            .register(async move {
                loop {
                    let node_path = node_path.lock().clone();

                    if node_path == PathBuf::default() {
                        tokio::time::sleep(FREE_DISK_SPACE_CHECK_INTERVAL).await;
                        continue;
                    }

                    match tokio::task::spawn_blocking(move || fs4::available_space(node_path)).await
                    {
                        Ok(Ok(free_disk_space)) => {
                            if sender
                                .send(NodeCommandOutput::FreeDiskSpace(ByteSize::b(
                                    free_disk_space,
                                )))
                                .is_err()
                            {
                                break;
                            }
                        }
                        Ok(Err(error)) => {
                            error!(%error, "Failed to check free disk space");
                            break;
                        }
                        Err(error) => {
                            error!(%error, "Free disk space task panicked");
                            break;
                        }
                    }

                    tokio::time::sleep(FREE_DISK_SPACE_CHECK_INTERVAL).await;
                }
            })
            .drop_on_shutdown()
            .await
    }
}
