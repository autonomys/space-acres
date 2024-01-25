use crate::backend::node::{ChainInfo, SyncKind, SyncState};
use crate::backend::NodeNotification;
use gtk::prelude::*;
use relm4::prelude::*;
use subspace_core_primitives::BlockNumber;

/// Maximum blocks to store in the import queue.
// HACK: This constant comes from Substrate's sync, but it is not public in there
const MAX_IMPORTING_BLOCKS: BlockNumber = 2048;

#[derive(Debug)]
pub enum NodeInput {
    Initialize {
        best_block_number: BlockNumber,
        chain_info: ChainInfo,
    },
    NodeNotification(NodeNotification),
}

#[derive(Debug, Default)]
struct NodeState {
    best_block_number: BlockNumber,
    sync_state: SyncState,
}

#[derive(Debug)]
pub struct NodeView {
    node_state: NodeState,
    chain_name: String,
}

#[relm4::component(pub)]
impl Component for NodeView {
    type Init = ();
    type Input = NodeInput;
    type Output = ();
    type CommandOutput = ();

    view! {
        #[root]
        gtk::Box {
            set_height_request: 100,
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,

            gtk::Label {
                add_css_class: "heading",
                set_halign: gtk::Align::Start,
                #[watch]
                set_label: &model.chain_name,
            },

            #[transition = "SlideUpDown"]
            match model.node_state.sync_state {
                SyncState::Unknown => gtk::Box {
                    gtk::Label {
                        #[watch]
                        set_label: &format!(
                            "Connecting to the network, best block #{}",
                            model.node_state.best_block_number
                        ),
                    }
                },
                SyncState::Syncing { kind, target, speed } => gtk::Box {
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

                                format!(
                                    "{} #{}/{}{}",
                                    kind,
                                    model.node_state.best_block_number,
                                    target,
                                    speed
                                        .map(|speed| format!(", {:.2} blocks/s", speed))
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
                        set_fraction: model.node_state.best_block_number as f64 / target as f64,
                    },
                },
                SyncState::Idle => gtk::Box {
                    gtk::Label {
                        #[watch]
                        set_label: &format!("Synced, best block #{}", model.node_state.best_block_number),
                    }
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        _root: Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = Self {
            node_state: NodeState::default(),
            chain_name: String::new(),
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>, _root: &Self::Root) {
        self.process_input(input);
    }
}

impl NodeView {
    fn process_input(&mut self, input: NodeInput) {
        match input {
            NodeInput::Initialize {
                best_block_number,
                chain_info,
            } => {
                self.node_state = NodeState {
                    best_block_number,
                    sync_state: SyncState::default(),
                };
                self.chain_name = format!(
                    "{} consensus node",
                    chain_info
                        .chain_name
                        .strip_prefix("Subspace ")
                        .unwrap_or(&chain_info.chain_name)
                );
            }
            NodeInput::NodeNotification(node_notification) => match node_notification {
                NodeNotification::SyncStateUpdate(mut sync_state) => {
                    if let SyncState::Syncing {
                        target: new_target, ..
                    } = &mut sync_state
                    {
                        *new_target = (*new_target).max(self.node_state.best_block_number);

                        // Ensure target is never below current block
                        if let SyncState::Syncing {
                            target: old_target, ..
                        } = &self.node_state.sync_state
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
                    self.node_state.sync_state = sync_state;
                }
                NodeNotification::BlockImported(imported_block) => {
                    self.node_state.best_block_number = imported_block.number;
                    // Ensure target is never below current block
                    if let SyncState::Syncing { target, .. } = &mut self.node_state.sync_state {
                        *target = (*target).max(self.node_state.best_block_number);
                    }
                }
            },
        }
    }
}
