pub(super) mod maybe_node_client;

use crate::backend::farmer::maybe_node_client::MaybeNodeRpcClient;
use crate::backend::utils::{Handler, Handler2, Handler2Fn, HandlerFn};
use crate::PosTable;
use anyhow::anyhow;
use atomic::Atomic;
use event_listener_primitives::HandlerId;
use futures::channel::oneshot;
use futures::future::BoxFuture;
use futures::stream::{FuturesOrdered, FuturesUnordered};
use futures::{select, FutureExt, StreamExt};
use lru::LruCache;
use parking_lot::Mutex;
use std::num::{NonZeroU8, NonZeroUsize};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::{fmt, fs};
use subspace_core_primitives::crypto::kzg::{embedded_kzg_settings, Kzg};
use subspace_core_primitives::{PublicKey, Record, SectorIndex};
use subspace_erasure_coding::ErasureCoding;
use subspace_farmer::piece_cache::{CacheWorker, PieceCache};
use subspace_farmer::single_disk_farm::{
    SingleDiskFarm, SingleDiskFarmError, SingleDiskFarmOptions, SingleDiskFarmSummary,
};
use subspace_farmer::utils::farmer_piece_getter::FarmerPieceGetter;
use subspace_farmer::utils::piece_validator::SegmentCommitmentPieceValidator;
use subspace_farmer::utils::readers_and_pieces::ReadersAndPieces;
use subspace_farmer::utils::run_future_in_dedicated_thread;
use subspace_farmer::{NodeClient, NodeRpcClient};
use subspace_farmer_components::plotting::PlottedSector;
use subspace_networking::utils::piece_provider::PieceProvider;
use subspace_networking::Node;
use tokio::sync::Semaphore;
use tracing::{error, info, info_span, warn, Instrument};

/// Minimal cache percentage, there is no need in setting it higher
const CACHE_PERCENTAGE: NonZeroU8 = NonZeroU8::MIN;
const RECORDS_ROOTS_CACHE_SIZE: NonZeroUsize = NonZeroUsize::new(1_000_000).expect("Not zero; qed");

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PlottingKind {
    Initial,
    Replotting,
}

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum PlottingState {
    #[default]
    Unknown,
    Plotting {
        kind: PlottingKind,
        /// Progress so far in % (not including this sector)
        progress: f32,
        /// Plotting/replotting speed in sectors/s
        speed: Option<f32>,
    },
    Idle,
}

#[derive(Default, Debug)]
struct Handlers {
    plotting_state_change: Handler2<usize, PlottingState>,
    piece_cache_sync_progress: Handler<f32>,
}

pub(super) struct Farmer {
    farm_fut: BoxFuture<'static, anyhow::Result<()>>,
    piece_cache_worker_fut: BoxFuture<'static, ()>,
    initial_plotting_states: Vec<PlottingState>,
    handlers: Arc<Handlers>,
}

impl Farmer {
    pub(super) async fn run(self) -> anyhow::Result<()> {
        let piece_cache_worker_fut = match run_future_in_dedicated_thread(
            move || self.piece_cache_worker_fut,
            "piece-cache-worker".to_string(),
        ) {
            Ok(piece_cache_worker_fut) => piece_cache_worker_fut,
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Failed to spawn piece future in background thread: {error}"
                ));
            }
        };

        let farm_fut = match run_future_in_dedicated_thread(
            move || self.farm_fut,
            "farmer-farm".to_string(),
        ) {
            Ok(piece_cache_worker_fut) => piece_cache_worker_fut,
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Failed to spawn piece future in background thread: {error}"
                ));
            }
        };

        select! {
            _ = piece_cache_worker_fut.fuse() => {
                // Nothing to do, just exit
            }
            result = farm_fut.fuse() => {
                result??;
            }
        }

        Ok(())
    }

    pub(super) fn initial_plotting_states(&self) -> &[PlottingState] {
        &self.initial_plotting_states
    }

    pub(super) fn on_plotting_state_change(
        &self,
        callback: Handler2Fn<usize, PlottingState>,
    ) -> HandlerId {
        self.handlers.plotting_state_change.add(callback)
    }

    pub(super) fn on_piece_cache_sync_progress(&self, callback: HandlerFn<f32>) -> HandlerId {
        self.handlers.piece_cache_sync_progress.add(callback)
    }
}

impl fmt::Debug for Farmer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Farmer").finish_non_exhaustive()
    }
}

fn available_parallelism() -> usize {
    match std::thread::available_parallelism() {
        Ok(parallelism) => parallelism.get(),
        Err(error) => {
            warn!(
                %error,
                "Unable to identify available parallelism, you might want to configure thread pool sizes with CLI \
                options manually"
            );

            0
        }
    }
}

#[derive(Debug, Clone)]
pub struct DiskFarm {
    pub directory: PathBuf,
    pub allocated_plotting_space: u64,
}

/// Arguments for farmer
#[derive(Debug)]
pub(super) struct FarmerOptions {
    pub(super) reward_address: PublicKey,
    pub(super) disk_farms: Vec<DiskFarm>,
    pub(super) node_client: NodeRpcClient,
    pub(super) node: Node,
    pub(super) readers_and_pieces: Arc<Mutex<Option<ReadersAndPieces>>>,
    pub(super) piece_cache: PieceCache,
    pub(super) piece_cache_worker: CacheWorker<MaybeNodeRpcClient>,
}

pub(super) async fn create_farmer(farmer_options: FarmerOptions) -> anyhow::Result<Farmer> {
    let span = info_span!("Farmer");
    let _enter = span.enter();

    let FarmerOptions {
        reward_address,
        disk_farms,
        node_client,
        node,
        readers_and_pieces,
        piece_cache,
        piece_cache_worker,
    } = farmer_options;

    if disk_farms.is_empty() {
        return Err(anyhow!("There must be at least one disk farm provided"));
    }

    for farm in &disk_farms {
        if !farm.directory.exists() {
            if let Err(error) = fs::create_dir(&farm.directory) {
                return Err(anyhow!(
                    "Directory {} doesn't exist and can't be created: {}",
                    farm.directory.display(),
                    error
                ));
            }
        }
    }

    let sector_downloading_concurrency = NonZeroUsize::new(2).expect("Not zero; qed");
    let sector_encoding_concurrency = NonZeroUsize::new(1).expect("Not zero; qed");
    let farm_during_initial_plotting = false;
    let available_parallelism = available_parallelism();
    let farming_thread_pool_size = available_parallelism;
    let plotting_thread_pool_size = available_parallelism;
    let replotting_thread_pool_size = available_parallelism / 2;

    let farmer_app_info = node_client
        .farmer_app_info()
        .await
        .map_err(|error| anyhow::anyhow!(error))?;

    let kzg = Kzg::new(embedded_kzg_settings());
    let erasure_coding = ErasureCoding::new(
        NonZeroUsize::new(Record::NUM_S_BUCKETS.next_power_of_two().ilog2() as usize)
            .expect("Not zero; qed"),
    )
    .map_err(|error| anyhow::anyhow!(error))?;
    // TODO: Consider introducing and using global in-memory segment header cache (this comment is
    //  in multiple files)
    let segment_commitments_cache = Mutex::new(LruCache::new(RECORDS_ROOTS_CACHE_SIZE));
    let piece_provider = PieceProvider::new(
        node.clone(),
        Some(SegmentCommitmentPieceValidator::new(
            node.clone(),
            node_client.clone(),
            kzg.clone(),
            segment_commitments_cache,
        )),
    );

    let piece_getter = Arc::new(FarmerPieceGetter::new(
        node.clone(),
        piece_provider,
        piece_cache.clone(),
        node_client.clone(),
        Arc::clone(&readers_and_pieces),
    ));

    let piece_cache_worker_fut = Box::pin(
        piece_cache_worker
            .run(piece_getter.clone())
            .in_current_span(),
    );

    let mut single_disk_farms = Vec::with_capacity(disk_farms.len());

    let downloading_semaphore = Arc::new(Semaphore::new(sector_downloading_concurrency.get()));
    let encoding_semaphore = Arc::new(Semaphore::new(sector_encoding_concurrency.get()));

    let mut plotting_delay_senders = Vec::with_capacity(disk_farms.len());

    for (disk_farm_index, disk_farm) in disk_farms.into_iter().enumerate() {
        let (plotting_delay_sender, plotting_delay_receiver) = oneshot::channel();
        plotting_delay_senders.push(plotting_delay_sender);

        let single_disk_farm_fut = SingleDiskFarm::new::<_, _, PosTable>(
            SingleDiskFarmOptions {
                directory: disk_farm.directory.clone(),
                farmer_app_info: farmer_app_info.clone(),
                allocated_space: disk_farm.allocated_plotting_space,
                max_pieces_in_sector: farmer_app_info.protocol_info.max_pieces_in_sector,
                node_client: node_client.clone(),
                reward_address,
                kzg: kzg.clone(),
                erasure_coding: erasure_coding.clone(),
                piece_getter: piece_getter.clone(),
                cache_percentage: CACHE_PERCENTAGE,
                downloading_semaphore: Arc::clone(&downloading_semaphore),
                encoding_semaphore: Arc::clone(&encoding_semaphore),
                farm_during_initial_plotting,
                farming_thread_pool_size,
                plotting_thread_pool_size,
                replotting_thread_pool_size,
                plotting_delay: Some(plotting_delay_receiver),
            },
            disk_farm_index,
        );

        let single_disk_farm = match single_disk_farm_fut.await {
            Ok(single_disk_farm) => single_disk_farm,
            Err(SingleDiskFarmError::InsufficientAllocatedSpace {
                min_space,
                allocated_space,
            }) => {
                return Err(anyhow::anyhow!(
                    "Allocated space {} ({}) is not enough, minimum is ~{} (~{}, {} bytes to be \
                    exact)",
                    bytesize::to_string(allocated_space, true),
                    bytesize::to_string(allocated_space, false),
                    bytesize::to_string(min_space, true),
                    bytesize::to_string(min_space, false),
                    min_space
                ));
            }
            Err(error) => {
                return Err(error.into());
            }
        };

        print_disk_farm_info(disk_farm.directory, disk_farm_index);

        single_disk_farms.push(single_disk_farm);
    }

    let cache_acknowledgement_receiver = piece_cache
        .replace_backing_caches(
            single_disk_farms
                .iter()
                .map(|single_disk_farm| single_disk_farm.piece_cache())
                .collect(),
        )
        .await;

    // Wait for cache initialization before starting plotting
    tokio::spawn(async move {
        if cache_acknowledgement_receiver.await.is_ok() {
            for plotting_delay_sender in plotting_delay_senders {
                // Doesn't matter if receiver is gone
                let _ = plotting_delay_sender.send(());
            }
        }
    });

    // Store piece readers so we can reference them later
    let piece_readers = single_disk_farms
        .iter()
        .map(|single_disk_farm| single_disk_farm.piece_reader())
        .collect::<Vec<_>>();

    info!("Collecting already plotted pieces (this will take some time)...");

    // Collect already plotted pieces
    {
        let mut future_readers_and_pieces = ReadersAndPieces::new(piece_readers);

        for (disk_farm_index, single_disk_farm) in single_disk_farms.iter().enumerate() {
            let disk_farm_index = disk_farm_index.try_into().map_err(|_error| {
                anyhow!(
                    "More than 256 plots are not supported, consider running multiple farmer \
                    instances"
                )
            })?;

            (0 as SectorIndex..)
                .zip(single_disk_farm.plotted_sectors().await)
                .for_each(
                    |(sector_index, plotted_sector_result)| match plotted_sector_result {
                        Ok(plotted_sector) => {
                            future_readers_and_pieces.add_sector(disk_farm_index, &plotted_sector);
                        }
                        Err(error) => {
                            error!(
                                %error,
                                %disk_farm_index,
                                %sector_index,
                                "Failed reading plotted sector on startup, skipping"
                            );
                        }
                    },
                );
        }

        readers_and_pieces.lock().replace(future_readers_and_pieces);
    }

    info!("Finished collecting already plotted pieces successfully");

    let handlers = Arc::new(Handlers::default());

    piece_cache
        .on_sync_progress(Arc::new({
            let handlers = Arc::clone(&handlers);

            move |progress| {
                handlers.piece_cache_sync_progress.call_simple(progress);
            }
        }))
        .detach();

    let initial_plotting_states = single_disk_farms
        .iter()
        .map(|single_disk_farm| async {
            if usize::from(single_disk_farm.total_sectors_count().await)
                == single_disk_farm.plotted_sectors_count().await
            {
                PlottingState::Idle
            } else {
                PlottingState::Unknown
            }
        })
        .collect::<FuturesOrdered<_>>()
        .collect()
        .await;

    let mut single_disk_farms_stream = single_disk_farms
        .into_iter()
        .enumerate()
        .map(|(disk_farm_index, single_disk_farm)| {
            let disk_farm_index = disk_farm_index.try_into().expect(
                "More than 256 plots are not supported, this is checked above already; qed",
            );
            let readers_and_pieces = Arc::clone(&readers_and_pieces);
            let span = info_span!("farm", %disk_farm_index);
            let handlers = Arc::clone(&handlers);
            let last_sector_plotted = Arc::new(Atomic::new(None));

            single_disk_farm
                .on_sector_plotting(Arc::new({
                    let handlers = Arc::clone(&handlers);
                    let last_sector_plotted = Arc::clone(&last_sector_plotted);

                    move |plotting_details| {
                        let state = PlottingState::Plotting {
                            kind: if plotting_details.replotting {
                                PlottingKind::Replotting
                            } else {
                                PlottingKind::Initial
                            },
                            progress: plotting_details.progress,
                            speed: None,
                        };

                        handlers
                            .plotting_state_change
                            .call_simple(&(disk_farm_index as usize), &state);

                        if plotting_details.last_queued {
                            last_sector_plotted
                                .store(Some(plotting_details.sector_index), Ordering::Release);
                        }
                    }
                }))
                .detach();

            // Collect newly plotted pieces
            let on_plotted_sector_callback =
                move |(plotted_sector, maybe_old_plotted_sector): &(
                    PlottedSector,
                    Option<PlottedSector>,
                )| {
                    let _span_guard = span.enter();

                    {
                        let mut readers_and_pieces = readers_and_pieces.lock();
                        let readers_and_pieces = readers_and_pieces
                            .as_mut()
                            .expect("Initial value was populated above; qed");

                        if let Some(old_plotted_sector) = maybe_old_plotted_sector {
                            readers_and_pieces.delete_sector(disk_farm_index, old_plotted_sector);
                        }
                        readers_and_pieces.add_sector(disk_farm_index, plotted_sector);

                        if last_sector_plotted
                            .compare_exchange(
                                Some(plotted_sector.sector_index),
                                None,
                                Ordering::AcqRel,
                                Ordering::Relaxed,
                            )
                            .is_ok()
                        {
                            handlers
                                .plotting_state_change
                                .call_simple(&(disk_farm_index as usize), &PlottingState::Idle);
                        }
                    }
                };
            single_disk_farm
                .on_sector_plotted(Arc::new(on_plotted_sector_callback))
                .detach();

            single_disk_farm.run()
        })
        .collect::<FuturesUnordered<_>>()
        .boxed();

    // Drop original instance such that the only remaining instances are in `SingleDiskFarm`
    // event handlers
    drop(readers_and_pieces);

    let farm_fut = Box::pin(
        async move {
            while let Some(result) = single_disk_farms_stream.next().await {
                match result {
                    Ok(id) => {
                        info!(%id, "Farm exited successfully");
                    }
                    Err(error) => {
                        return Err(error);
                    }
                }
            }
            anyhow::Ok(())
        }
        .in_current_span(),
    );

    anyhow::Ok(Farmer {
        farm_fut,
        piece_cache_worker_fut,
        initial_plotting_states,
        handlers,
    })
}

fn print_disk_farm_info(directory: PathBuf, disk_farm_index: usize) {
    println!("Single disk farm {disk_farm_index}:");
    match SingleDiskFarm::collect_summary(directory) {
        SingleDiskFarmSummary::Found { info, directory } => {
            println!("  ID: {}", info.id());
            println!("  Genesis hash: 0x{}", hex::encode(info.genesis_hash()));
            println!("  Public key: 0x{}", hex::encode(info.public_key()));
            println!(
                "  Allocated space: {} ({})",
                bytesize::to_string(info.allocated_space(), true),
                bytesize::to_string(info.allocated_space(), false)
            );
            println!("  Directory: {}", directory.display());
        }
        SingleDiskFarmSummary::NotFound { directory } => {
            println!("  Plot directory: {}", directory.display());
            println!("  No farm found here yet");
        }
        SingleDiskFarmSummary::Error { directory, error } => {
            println!("  Directory: {}", directory.display());
            println!("  Failed to open farm info: {error}");
        }
    }
}
