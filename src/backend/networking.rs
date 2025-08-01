use async_lock::RwLock as AsyncRwLock;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::fmt;
use std::hash::Hash;
use std::path::Path;
use std::sync::{Arc, Weak};
use subspace_farmer::KNOWN_PEERS_CACHE_SIZE;
use subspace_farmer::farm::plotted_pieces::PlottedPieces;
use subspace_farmer::farmer_cache::FarmerCache;
use subspace_farmer::node_client::NodeClientExt;
use subspace_networking::libp2p::Multiaddr;
use subspace_networking::libp2p::identity::ed25519::Keypair;
use subspace_networking::libp2p::multiaddr::Protocol;
use subspace_networking::protocols::request_response::handlers::cached_piece_by_index::{
    CachedPieceByIndexRequest, CachedPieceByIndexRequestHandler, CachedPieceByIndexResponse,
    PieceResult,
};
use subspace_networking::protocols::request_response::handlers::piece_by_index::{
    PieceByIndexRequest, PieceByIndexRequestHandler, PieceByIndexResponse,
};
use subspace_networking::protocols::request_response::handlers::segment_header::{
    SegmentHeaderBySegmentIndexesRequestHandler, SegmentHeaderRequest, SegmentHeaderResponse,
};
use subspace_networking::utils::multihash::ToMultihash;
use subspace_networking::utils::strip_peer_id;
use subspace_networking::{
    Config, KademliaMode, KnownPeersManager, KnownPeersManagerConfig, Node, NodeRunner, WeakNode,
    construct,
};
use subspace_rpc_primitives::MAX_SEGMENT_HEADERS_PER_REQUEST;
use tracing::{Instrument, debug, error, info, info_span, warn};

/// How many segment headers can be requested at a time.
///
/// Must be the same as RPC limit since all requests go to the node anyway.
const SEGMENT_HEADERS_LIMIT: u32 = MAX_SEGMENT_HEADERS_PER_REQUEST as u32;

/// Network options, mainly used for the DSN.
/// Most substrate network options come from the chainspec, or node defaults constants.
#[derive(Debug)]
pub struct NetworkOptions {
    /// Keypair to use for substrate and DSN network identity
    pub keypair: Keypair,
    /// Multiaddrs of DSN bootstrap nodes to connect to on startup, multiple are supported
    pub bootstrap_nodes: Vec<Multiaddr>,
    /// Multiaddr to listen on for DSN, for instance `/ip4/0.0.0.0/tcp/0`, multiple are supported
    pub listen_on: Vec<Multiaddr>,
    /// Determines whether we allow keeping non-global (private, shared, loopback..) addresses in
    /// DSN Kademlia DHT
    pub enable_private_ips: bool,
    /// Multiaddrs of reserved DSN nodes to maintain a connection to, multiple are supported
    pub reserved_peers: Vec<Multiaddr>,
    /// Defines max established incoming DSN connection limit
    pub in_connections: u32,
    /// Defines max established outgoing DSN swarm connection limit
    pub out_connections: u32,
    /// Defines max pending incoming DSN connection limit
    pub pending_in_connections: u32,
    /// Defines max pending outgoing DSN swarm connection limit
    pub pending_out_connections: u32,
    /// Known external DSN addresses
    pub external_addresses: Vec<Multiaddr>,
}

impl Default for NetworkOptions {
    fn default() -> Self {
        Self {
            keypair: Keypair::generate(),
            bootstrap_nodes: vec![],
            listen_on: vec![],
            enable_private_ips: false,
            reserved_peers: Vec::new(),
            in_connections: 300,
            out_connections: 100,
            pending_in_connections: 100,
            pending_out_connections: 100,
            external_addresses: Vec::new(),
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn create_network<FarmIndex, NC>(
    protocol_prefix: String,
    base_path: &Path,
    NetworkOptions {
        keypair,
        listen_on,
        bootstrap_nodes,
        enable_private_ips,
        reserved_peers,
        in_connections,
        out_connections,
        pending_in_connections,
        pending_out_connections,
        external_addresses,
    }: NetworkOptions,
    weak_plotted_pieces: Weak<AsyncRwLock<PlottedPieces<FarmIndex>>>,
    node_client: NC,
    farmer_cache: FarmerCache,
) -> Result<(Node, NodeRunner), anyhow::Error>
where
    FarmIndex: Hash + Eq + Copy + fmt::Debug + Send + Sync + 'static,
    usize: From<FarmIndex>,
    NC: NodeClientExt + Clone,
{
    let span = info_span!("Network");
    let _enter = span.enter();

    let known_peers_registry = KnownPeersManager::new(KnownPeersManagerConfig {
        path: Some(base_path.join("known_addresses.bin").into_boxed_path()),
        ignore_peer_list: strip_peer_id(bootstrap_nodes.clone())
            .into_iter()
            .map(|(peer_id, _)| peer_id)
            .collect::<HashSet<_>>(),
        cache_size: KNOWN_PEERS_CACHE_SIZE,
        ..Default::default()
    })
    .map(Box::new)?;

    let maybe_weak_node = Arc::new(Mutex::new(None::<WeakNode>));
    let default_config = Config::new(protocol_prefix, keypair.into(), None);
    let config = Config {
        reserved_peers,
        listen_on,
        allow_non_global_addresses_in_dht: enable_private_ips,
        known_peers_registry,
        request_response_protocols: vec![
            {
                let maybe_weak_node = Arc::clone(&maybe_weak_node);
                let farmer_cache = farmer_cache.clone();

                CachedPieceByIndexRequestHandler::create(move |peer_id, request| {
                    let CachedPieceByIndexRequest {
                        piece_index,
                        cached_pieces,
                    } = request;
                    debug!(?piece_index, "Cached piece request received");

                    let maybe_weak_node = Arc::clone(&maybe_weak_node);
                    let farmer_cache = farmer_cache.clone();
                    let mut cached_pieces = Arc::unwrap_or_clone(cached_pieces);

                    async move {
                        let piece_from_cache =
                            farmer_cache.get_piece(piece_index.to_multihash()).await;
                        cached_pieces.truncate(CachedPieceByIndexRequest::RECOMMENDED_LIMIT);
                        let cached_pieces = farmer_cache.has_pieces(cached_pieces).await;

                        Some(CachedPieceByIndexResponse {
                            result: match piece_from_cache {
                                Some(piece) => PieceResult::Piece(piece),
                                None => {
                                    let maybe_node = maybe_weak_node
                                        .lock()
                                        .as_ref()
                                        .expect("Always called after network instantiation; qed")
                                        .upgrade();

                                    let closest_peers = if let Some(node) = maybe_node {
                                        node.get_closest_local_peers(
                                            piece_index.to_multihash(),
                                            Some(peer_id),
                                        )
                                        .await
                                        .inspect_err(|error| {
                                            warn!(%error, "Failed to get closest local peers");
                                        })
                                        .unwrap_or_default()
                                    } else {
                                        Vec::new()
                                    };

                                    PieceResult::ClosestPeers(closest_peers.into())
                                }
                            },
                            cached_pieces,
                        })
                    }
                    .in_current_span()
                })
            },
            PieceByIndexRequestHandler::create(move |_, request| {
                let PieceByIndexRequest {
                    piece_index,
                    cached_pieces,
                } = request;
                debug!(?piece_index, "Piece request received. Trying cache...");

                let weak_plotted_pieces = weak_plotted_pieces.clone();
                let farmer_cache = farmer_cache.clone();
                let mut cached_pieces = Arc::unwrap_or_clone(cached_pieces);

                async move {
                    let piece_from_cache = farmer_cache.get_piece(piece_index.to_multihash()).await;
                    cached_pieces.truncate(PieceByIndexRequest::RECOMMENDED_LIMIT);
                    let cached_pieces = farmer_cache.has_pieces(cached_pieces).await;

                    if let Some(piece) = piece_from_cache {
                        Some(PieceByIndexResponse {
                            piece: Some(piece),
                            cached_pieces,
                        })
                    } else {
                        debug!(
                            ?piece_index,
                            "No piece in the cache. Trying archival storage..."
                        );

                        let read_piece_fut = match weak_plotted_pieces.upgrade() {
                            Some(plotted_pieces) => plotted_pieces
                                .try_read()?
                                .read_piece(piece_index)?
                                .in_current_span(),
                            None => {
                                debug!("A readers and pieces are already dropped");
                                return None;
                            }
                        };

                        let piece = read_piece_fut.await;

                        Some(PieceByIndexResponse {
                            piece,
                            cached_pieces,
                        })
                    }
                }
                .in_current_span()
            }),
            SegmentHeaderBySegmentIndexesRequestHandler::create(move |_, req| {
                debug!(?req, "Segment headers request received.");

                let node_client = node_client.clone();

                async move {
                    let internal_result = match req {
                        SegmentHeaderRequest::SegmentIndexes { segment_indexes } => {
                            let segment_indexes = Arc::unwrap_or_clone(segment_indexes);

                            if segment_indexes.len() > SEGMENT_HEADERS_LIMIT as usize {
                                debug!(
                                    "segment_indexes length exceed the limit: {} ",
                                    segment_indexes.len()
                                );

                                return None;
                            }

                            debug!(
                                segment_indexes_count = ?segment_indexes.len(),
                                "Segment headers request received."
                            );

                            node_client.segment_headers(segment_indexes).await
                        }
                        SegmentHeaderRequest::LastSegmentHeaders { mut limit } => {
                            if limit > SEGMENT_HEADERS_LIMIT {
                                debug!(
                                    %limit,
                                    "Segment header number exceeded the limit."
                                );

                                limit = SEGMENT_HEADERS_LIMIT;
                            }
                            node_client.last_segment_headers(limit).await
                        }
                    };

                    match internal_result {
                        Ok(segment_headers) => segment_headers
                            .into_iter()
                            .inspect(|maybe_segment_header| {
                                if maybe_segment_header.is_none() {
                                    error!("Received empty optional segment header!");
                                }
                            })
                            .collect::<Option<Vec<_>>>()
                            .map(|segment_headers| SegmentHeaderResponse { segment_headers }),
                        Err(error) => {
                            error!(%error, "Failed to get segment headers from cache");

                            None
                        }
                    }
                }
                .in_current_span()
            }),
        ],
        max_established_outgoing_connections: out_connections,
        max_pending_outgoing_connections: pending_out_connections,
        max_established_incoming_connections: in_connections,
        max_pending_incoming_connections: pending_in_connections,
        bootstrap_addresses: bootstrap_nodes,
        kademlia_mode: KademliaMode::Dynamic,
        external_addresses,
        metrics: None,
        ..default_config
    };

    let (node, node_runner) = construct(config)?;
    maybe_weak_node.lock().replace(node.downgrade());

    node.on_new_listener(Arc::new({
        let node = node.clone();

        move |address| {
            info!(
                "DSN listening on {}",
                address.clone().with(Protocol::P2p(node.id()))
            );
        }
    }))
    .detach();

    // Consider returning HandlerId instead of each `detach()` calls for other usages.
    Ok((node, node_runner))
}
