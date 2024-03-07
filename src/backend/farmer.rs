pub(super) mod maybe_node_client;

use crate::backend::farmer::maybe_node_client::MaybeNodeRpcClient;
use crate::backend::utils::{Handler, HandlerFn};
use crate::backend::PieceGetterWrapper;
use crate::PosTable;
use anyhow::anyhow;
use event_listener_primitives::HandlerId;
use futures::channel::oneshot;
use futures::future::BoxFuture;
use futures::stream::{FuturesOrdered, FuturesUnordered};
use futures::{select, FutureExt, StreamExt};
use parking_lot::Mutex;
use rayon::prelude::*;
use std::num::{NonZeroU8, NonZeroUsize};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fmt, fs};
use subspace_core_primitives::crypto::kzg::Kzg;
use subspace_core_primitives::{PublicKey, Record, SectorIndex};
use subspace_erasure_coding::ErasureCoding;
use subspace_farmer::farmer_cache::{FarmerCache, FarmerCacheWorker};
use subspace_farmer::single_disk_farm::farming::FarmingNotification;
use subspace_farmer::single_disk_farm::{
    SectorPlottingDetails, SectorUpdate, SingleDiskFarm, SingleDiskFarmError,
    SingleDiskFarmOptions, SingleDiskFarmSummary,
};
use subspace_farmer::utils::plotted_pieces::PlottedPieces;
use subspace_farmer::utils::{
    all_cpu_cores, create_plotting_thread_pool_manager, recommended_number_of_farming_threads,
    run_future_in_dedicated_thread, thread_pool_core_indices, CpuCoreSet,
};
use subspace_farmer::NodeClient;
use subspace_farmer_components::plotting::PlottedSector;
use thread_priority::ThreadPriority;
use tokio::runtime::Handle;
use tokio::sync::Semaphore;
use tracing::{error, info, info_span, Instrument};

/// Minimal cache percentage, there is no need in setting it higher
const CACHE_PERCENTAGE: NonZeroU8 = NonZeroU8::MIN;

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct InitialFarmState {
    pub total_sectors_count: SectorIndex,
    pub plotted_sectors_count: SectorIndex,
}

#[derive(Debug, Clone)]
pub enum FarmerNotification {
    SectorUpdate {
        farm_index: u8,
        sector_index: SectorIndex,
        update: SectorUpdate,
    },
    FarmingNotification {
        farm_index: u8,
        notification: FarmingNotification,
    },
    FarmerCacheSyncProgress {
        /// Progress so far in %
        progress: f32,
    },
}

type Notifications = Handler<FarmerNotification>;

pub(super) struct Farmer {
    farm_fut: BoxFuture<'static, anyhow::Result<()>>,
    farmer_cache_worker_fut: BoxFuture<'static, ()>,
    initial_farm_states: Vec<InitialFarmState>,
    farm_during_initial_plotting: bool,
    notifications: Arc<Notifications>,
    /// Whether one of the farms was resized during initialization
    resized: bool,
}

impl Farmer {
    pub(super) async fn run(self) -> anyhow::Result<()> {
        let farmer_cache_worker_fut = match run_future_in_dedicated_thread(
            move || self.farmer_cache_worker_fut,
            "farmer-cache-worker".to_string(),
        ) {
            Ok(farmer_cache_worker_fut) => farmer_cache_worker_fut,
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Failed to spawn farmer cache future in background thread: {error}"
                ));
            }
        };

        let farm_fut = match run_future_in_dedicated_thread(
            move || self.farm_fut,
            "farmer-farm".to_string(),
        ) {
            Ok(farmer_cache_worker_fut) => farmer_cache_worker_fut,
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Failed to spawn farm future in background thread: {error}"
                ));
            }
        };

        select! {
            _ = farmer_cache_worker_fut.fuse() => {
                // Nothing to do, just exit
            }
            result = farm_fut.fuse() => {
                result??;
            }
        }

        Ok(())
    }

    pub(super) fn initial_farm_states(&self) -> &[InitialFarmState] {
        &self.initial_farm_states
    }

    pub(super) fn farm_during_initial_plotting(&self) -> bool {
        self.farm_during_initial_plotting
    }

    /// Whether one of the farms was resized during initialization
    pub(super) fn resized(&self) -> bool {
        self.resized
    }

    pub(super) fn on_notification(&self, callback: HandlerFn<FarmerNotification>) -> HandlerId {
        self.notifications.add(callback)
    }
}

impl fmt::Debug for Farmer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Farmer").finish_non_exhaustive()
    }
}

fn should_farm_during_initial_plotting() -> bool {
    let total_cpu_cores = all_cpu_cores()
        .iter()
        .flat_map(|set| set.cpu_cores())
        .count();
    total_cpu_cores > 8
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
    pub(super) node_client: MaybeNodeRpcClient,
    pub(super) piece_getter: PieceGetterWrapper,
    pub(super) plotted_pieces: Arc<Mutex<Option<PlottedPieces>>>,
    pub(super) farmer_cache: FarmerCache,
    pub(super) farmer_cache_worker: FarmerCacheWorker<MaybeNodeRpcClient>,
    pub(super) kzg: Kzg,
}

pub(super) async fn create_farmer(farmer_options: FarmerOptions) -> anyhow::Result<Farmer> {
    let span = info_span!("Farmer");
    let _enter = span.enter();

    let FarmerOptions {
        reward_address,
        disk_farms,
        node_client,
        piece_getter,
        plotted_pieces,
        farmer_cache,
        farmer_cache_worker,
        kzg,
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

    let farmer_app_info = node_client
        .farmer_app_info()
        .await
        .map_err(|error| anyhow::anyhow!(error))?;

    let erasure_coding = ErasureCoding::new(
        NonZeroUsize::new(Record::NUM_S_BUCKETS.next_power_of_two().ilog2() as usize)
            .expect("Not zero; qed"),
    )
    .map_err(|error| anyhow::anyhow!(error))?;

    let farmer_cache_worker_fut = Box::pin(
        farmer_cache_worker
            .run(piece_getter.downgrade())
            .in_current_span(),
    );

    let farm_during_initial_plotting = should_farm_during_initial_plotting();
    let mut plotting_thread_pool_core_indices = thread_pool_core_indices(None, None);
    let mut replotting_thread_pool_core_indices = {
        let mut replotting_thread_pool_core_indices = thread_pool_core_indices(None, None);
        // The default behavior is to use all CPU cores, but for replotting we just want half
        replotting_thread_pool_core_indices
            .iter_mut()
            .for_each(|set| set.truncate(set.cpu_cores().len() / 2));
        replotting_thread_pool_core_indices
    };

    if plotting_thread_pool_core_indices.len() > 1 {
        info!(
            l3_cache_groups = %plotting_thread_pool_core_indices.len(),
            "Multiple L3 cache groups detected"
        );

        if plotting_thread_pool_core_indices.len() > disk_farms.len() {
            plotting_thread_pool_core_indices =
                CpuCoreSet::regroup(&plotting_thread_pool_core_indices, disk_farms.len());
            replotting_thread_pool_core_indices =
                CpuCoreSet::regroup(&replotting_thread_pool_core_indices, disk_farms.len());

            info!(
                farms_count = %disk_farms.len(),
                "Regrouped CPU cores to match number of farms, more farms may leverage CPU more efficiently"
            );
        }
    }

    let downloading_semaphore =
        Arc::new(Semaphore::new(plotting_thread_pool_core_indices.len() + 1));

    let record_encoding_concurrency = {
        let cpu_cores = plotting_thread_pool_core_indices
            .first()
            .expect("Guaranteed to have some CPU cores; qed");

        NonZeroUsize::new((cpu_cores.cpu_cores().len() / 2).min(8))
            .expect("Guaranteed to have some CPU cores; qed")
    };

    let plotting_thread_pool_manager = create_plotting_thread_pool_manager(
        plotting_thread_pool_core_indices
            .into_iter()
            .zip(replotting_thread_pool_core_indices),
        Some(ThreadPriority::Min),
    )?;

    let (single_disk_farms, plotting_delay_senders, resized) = tokio::task::block_in_place(|| {
        let handle = Handle::current();
        let (plotting_delay_senders, plotting_delay_receivers) = (0..disk_farms.len())
            .map(|_| oneshot::channel())
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let resized = &AtomicBool::new(false);

        let single_disk_farms = disk_farms
            .into_par_iter()
            .zip(plotting_delay_receivers)
            .enumerate()
            .map(
                move |(disk_farm_index, (disk_farm, plotting_delay_receiver))| {
                    let _tokio_handle_guard = handle.enter();

                    let resized_local =
                        match SingleDiskFarm::collect_summary(disk_farm.directory.clone()) {
                            SingleDiskFarmSummary::Found { info, .. } => {
                                info.allocated_space() != disk_farm.allocated_plotting_space
                            }
                            SingleDiskFarmSummary::NotFound { .. } => true,
                            SingleDiskFarmSummary::Error { .. } => true,
                        };
                    if resized_local {
                        resized.store(true, Ordering::Release);
                    }

                    let single_disk_farm_fut = SingleDiskFarm::new::<_, _, PosTable>(
                        SingleDiskFarmOptions {
                            directory: disk_farm.directory.clone(),
                            farmer_app_info: farmer_app_info.clone(),
                            // TODO: Check for allocated space change and show warning in UI that
                            //  restart is needed on Windows
                            allocated_space: disk_farm.allocated_plotting_space,
                            max_pieces_in_sector: farmer_app_info
                                .protocol_info
                                .max_pieces_in_sector,
                            node_client: node_client.clone(),
                            reward_address,
                            kzg: kzg.clone(),
                            erasure_coding: erasure_coding.clone(),
                            piece_getter: piece_getter.clone(),
                            cache_percentage: CACHE_PERCENTAGE,
                            downloading_semaphore: Arc::clone(&downloading_semaphore),
                            record_encoding_concurrency,
                            farm_during_initial_plotting,
                            farming_thread_pool_size: recommended_number_of_farming_threads(),
                            plotting_thread_pool_manager: plotting_thread_pool_manager.clone(),
                            plotting_delay: Some(plotting_delay_receiver),
                            disable_farm_locking: false,
                        },
                        disk_farm_index,
                    );

                    let single_disk_farm = match handle.block_on(single_disk_farm_fut) {
                        Ok(single_disk_farm) => single_disk_farm,
                        Err(SingleDiskFarmError::InsufficientAllocatedSpace {
                            min_space,
                            allocated_space,
                        }) => {
                            return Err(anyhow::anyhow!(
                                "Allocated space {} ({}) is not enough, minimum is ~{} (~{}, \
                                {} bytes to be exact)",
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

                    let info = single_disk_farm.info();
                    println!("Single disk farm {disk_farm_index}:");
                    println!("  ID: {}", info.id());
                    println!("  Genesis hash: 0x{}", hex::encode(info.genesis_hash()));
                    println!("  Public key: 0x{}", hex::encode(info.public_key()));
                    println!(
                        "  Allocated space: {} ({})",
                        bytesize::to_string(info.allocated_space(), true),
                        bytesize::to_string(info.allocated_space(), false)
                    );
                    println!("  Directory: {}", disk_farm.directory.display());

                    Ok(single_disk_farm)
                },
            )
            .collect::<Result<Vec<_>, _>>()?;

        anyhow::Ok((
            single_disk_farms,
            plotting_delay_senders,
            resized.load(Ordering::Acquire),
        ))
    })?;

    {
        let handler_id = Arc::new(Mutex::new(None));
        // Wait for piece cache to read already cached contents before starting plotting to improve
        // cache hit ratio
        handler_id
            .lock()
            .replace(farmer_cache.on_sync_progress(Arc::new({
                let handler_id = Arc::clone(&handler_id);
                let plotting_delay_senders = Mutex::new(plotting_delay_senders);

                move |_progress| {
                    for plotting_delay_sender in plotting_delay_senders.lock().drain(..) {
                        // Doesn't matter if receiver is gone
                        let _ = plotting_delay_sender.send(());
                    }

                    // Unsubscribe from this event
                    handler_id.lock().take();
                }
            })));
    }
    farmer_cache
        .replace_backing_caches(
            single_disk_farms
                .iter()
                .map(|single_disk_farm| single_disk_farm.piece_cache())
                .collect(),
            single_disk_farms
                .iter()
                .map(|single_disk_farm| single_disk_farm.plot_cache())
                .collect(),
        )
        .await;

    // Store piece readers so we can reference them later
    let piece_readers = single_disk_farms
        .iter()
        .map(|single_disk_farm| single_disk_farm.piece_reader())
        .collect::<Vec<_>>();

    info!("Collecting already plotted pieces (this will take some time)...");

    // Collect already plotted pieces
    {
        let mut future_plotted_pieces = PlottedPieces::new(piece_readers);

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
                            future_plotted_pieces.add_sector(disk_farm_index, &plotted_sector);
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

        plotted_pieces.lock().replace(future_plotted_pieces);
    }

    info!("Finished collecting already plotted pieces successfully");

    let notifications = Arc::new(Notifications::default());

    farmer_cache
        .on_sync_progress(Arc::new({
            let notifications = Arc::clone(&notifications);

            move |progress| {
                notifications.call_simple(&FarmerNotification::FarmerCacheSyncProgress {
                    progress: *progress,
                });
            }
        }))
        .detach();

    let initial_farm_states = single_disk_farms
        .iter()
        .map(|single_disk_farm| async {
            InitialFarmState {
                total_sectors_count: single_disk_farm.total_sectors_count(),
                plotted_sectors_count: single_disk_farm.plotted_sectors_count().await,
            }
        })
        .collect::<FuturesOrdered<_>>()
        .collect()
        .await;

    let mut single_disk_farms_stream = single_disk_farms
        .into_iter()
        .enumerate()
        .map(|(disk_farm_index, single_disk_farm)| {
            let disk_farm_index = u8::try_from(disk_farm_index).expect(
                "More than 256 plots are not supported, this is checked above already; qed",
            );
            let plotted_pieces = Arc::clone(&plotted_pieces);
            let span = info_span!("farm", %disk_farm_index);

            single_disk_farm
                .on_sector_update(Arc::new({
                    let notifications = Arc::clone(&notifications);

                    move |(sector_index, sector_update)| {
                        notifications.call_simple(&FarmerNotification::SectorUpdate {
                            farm_index: disk_farm_index,
                            sector_index: *sector_index,
                            update: sector_update.clone(),
                        });
                    }
                }))
                .detach();
            single_disk_farm
                .on_farming_notification(Arc::new({
                    let notifications = Arc::clone(&notifications);

                    move |notification| {
                        notifications.call_simple(&FarmerNotification::FarmingNotification {
                            farm_index: disk_farm_index,
                            notification: notification.clone(),
                        });
                    }
                }))
                .detach();

            // Collect newly plotted pieces
            let on_plotted_sector_callback =
                move |plotted_sector: &PlottedSector,
                      maybe_old_plotted_sector: &Option<PlottedSector>| {
                    let _span_guard = span.enter();

                    {
                        let mut plotted_pieces = plotted_pieces.lock();
                        let plotted_pieces = plotted_pieces
                            .as_mut()
                            .expect("Initial value was populated above; qed");

                        if let Some(old_plotted_sector) = &maybe_old_plotted_sector {
                            plotted_pieces.delete_sector(disk_farm_index, old_plotted_sector);
                        }
                        plotted_pieces.add_sector(disk_farm_index, plotted_sector);
                    }
                };
            single_disk_farm
                .on_sector_update(Arc::new(move |(_sector_index, sector_state)| {
                    if let SectorUpdate::Plotting(SectorPlottingDetails::Finished {
                        plotted_sector,
                        old_plotted_sector,
                        ..
                    }) = sector_state
                    {
                        on_plotted_sector_callback(plotted_sector, old_plotted_sector);
                    }
                }))
                .detach();

            single_disk_farm.run()
        })
        .collect::<FuturesUnordered<_>>()
        .boxed();

    // Drop original instance such that the only remaining instances are in `SingleDiskFarm`
    // event handlers
    drop(plotted_pieces);

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
        farmer_cache_worker_fut,
        initial_farm_states,
        farm_during_initial_plotting,
        notifications,
        resized,
    })
}
