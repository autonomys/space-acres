pub(super) mod direct_node_client;
pub(super) mod maybe_node_client;

use crate::FarmerPosTable;
use crate::backend::farmer::maybe_node_client::MaybeNodeClient;
use crate::backend::utils::{Handler, HandlerFn};
use anyhow::anyhow;
use async_lock::{Mutex as AsyncMutex, RwLock as AsyncRwLock, Semaphore};
use bytesize::ByteSize;
use event_listener_primitives::HandlerId;
use futures::channel::{mpsc, oneshot};
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt, select};
use parking_lot::Mutex;
use std::future::pending;
use std::hash::Hash;
use std::num::{NonZeroU8, NonZeroUsize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use std::{fmt, fs};
use subspace_core_primitives::PublicKey;
use subspace_core_primitives::pieces::Record;
use subspace_core_primitives::sectors::SectorIndex;
use subspace_data_retrieval::piece_getter::PieceGetter;
use subspace_erasure_coding::ErasureCoding;
use subspace_farmer::farm::plotted_pieces::PlottedPieces;
use subspace_farmer::farm::{
    FarmingNotification, PlottedSectors, SectorPlottingDetails, SectorUpdate,
};
use subspace_farmer::farmer_cache::{FarmerCache, FarmerCacheWorker};
use subspace_farmer::farmer_piece_getter::FarmerPieceGetter;
use subspace_farmer::farmer_piece_getter::piece_validator::SegmentCommitmentPieceValidator;
use subspace_farmer::node_client::NodeClient;
use subspace_farmer::plotter::Plotter;
use subspace_farmer::plotter::cpu::CpuPlotter;
use subspace_farmer::plotter::gpu::GpuPlotter;
use subspace_farmer::plotter::gpu::wgpu::WgpuRecordsEncoder;
use subspace_farmer::single_disk_farm::{
    SingleDiskFarm, SingleDiskFarmError, SingleDiskFarmOptions,
};
use subspace_farmer::utils::{
    create_plotting_thread_pool_manager, recommended_number_of_farming_threads,
    thread_pool_core_indices,
};
use subspace_farmer_components::plotting::PlottedSector;
use subspace_farmer_components::reading::ReadSectorRecordChunksMode;
use subspace_kzg::Kzg;
use subspace_process::{AsyncJoinOnDrop, run_future_in_dedicated_thread};
use subspace_proof_of_space_wgpu::{Backend, Device, DeviceType, WgpuDevice};
use thread_priority::ThreadPriority;
use tokio::sync::watch;
use tracing::{Instrument, debug, error, info, info_span, warn};

/// Minimal cache percentage, there is no need in setting it higher
pub(super) const CACHE_PERCENTAGE: NonZeroU8 = NonZeroU8::MIN;
/// NOTE: for large gaps between the plotted part and the end of the file plot cache will result in
/// very long period of writing zeroes on Windows, see <https://stackoverflow.com/q/78058306/3806795>
const MAX_SPACE_PLEDGED_FOR_PLOT_CACHE_ON_WINDOWS: u64 = ByteSize::tib(7).as_u64();
const FARM_ERROR_PRINT_INTERVAL: Duration = Duration::from_secs(30);
const MAX_PLOTTING_SECTORS_PER_FARM: NonZeroUsize = NonZeroUsize::new(2).expect("Not zero; qed");

#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub struct InitialFarmState {
    pub total_sectors_count: SectorIndex,
    pub plotted_sectors_count: SectorIndex,
}

#[derive(Debug, Clone)]
pub enum FarmerNotification<FarmIndex> {
    SectorUpdate {
        farm_index: FarmIndex,
        sector_index: SectorIndex,
        update: SectorUpdate,
    },
    FarmingNotification {
        farm_index: FarmIndex,
        notification: FarmingNotification,
    },
    FarmerCacheSyncProgress {
        /// Progress so far in %
        progress: f32,
    },
    FarmError {
        farm_index: FarmIndex,
        error: Arc<anyhow::Error>,
    },
}

#[derive(Debug, Clone)]
pub enum FarmerAction {
    /// Pause (or resume) plotting
    PausePlotting(bool),
}

type Notifications<FarmIndex> = Handler<FarmerNotification<FarmIndex>>;

pub(super) struct Farmer<FarmIndex>
where
    FarmIndex: 'static,
{
    farmer_fut: BoxFuture<'static, anyhow::Result<()>>,
    farmer_cache_worker_fut: BoxFuture<'static, ()>,
    initial_farm_states: Vec<InitialFarmState>,
    notifications: Arc<Notifications<FarmIndex>>,
    action_sender: mpsc::Sender<FarmerAction>,
}

impl<FarmIndex> Farmer<FarmIndex>
where
    FarmIndex: 'static,
{
    pub(super) async fn run(self) -> anyhow::Result<()> {
        let Farmer {
            farmer_fut,
            farmer_cache_worker_fut,
            initial_farm_states,
            notifications,
            action_sender,
        } = self;

        // Explicitly drop unnecessary things, especially senders to make sure farmer can exit
        // gracefully when `fn run()`'s future is dropped
        drop(initial_farm_states);
        drop(notifications);
        drop(action_sender);

        let farmer_cache_worker_fut = match run_future_in_dedicated_thread(
            move || farmer_cache_worker_fut,
            "farmer-cache-worker".to_string(),
        ) {
            Ok(farmer_cache_worker_fut) => farmer_cache_worker_fut,
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Failed to spawn farmer cache future in background thread: {error}"
                ));
            }
        };

        let farm_fut =
            match run_future_in_dedicated_thread(move || farmer_fut, "farmer-farmer".to_string()) {
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

    pub(super) fn action_sender(&self) -> mpsc::Sender<FarmerAction> {
        self.action_sender.clone()
    }

    pub(super) fn on_notification(
        &self,
        callback: HandlerFn<FarmerNotification<FarmIndex>>,
    ) -> HandlerId {
        self.notifications.add(callback)
    }
}

impl<FarmIndex> fmt::Debug for Farmer<FarmIndex> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("Farmer").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone)]
pub struct DiskFarm {
    pub directory: PathBuf,
    pub allocated_space: u64,
}

/// Arguments for farmer
#[derive(Debug)]
pub(super) struct FarmerOptions<FarmIndex, OnFarmInitialized> {
    pub(super) reward_address: PublicKey,
    pub(super) disk_farms: Vec<DiskFarm>,
    pub(super) node_client: MaybeNodeClient,
    pub(super) piece_getter: FarmerPieceGetter<
        FarmIndex,
        SegmentCommitmentPieceValidator<MaybeNodeClient>,
        MaybeNodeClient,
    >,

    pub(super) plotted_pieces: Arc<AsyncRwLock<PlottedPieces<FarmIndex>>>,
    pub(super) farmer_cache: FarmerCache,
    pub(super) farmer_cache_worker: FarmerCacheWorker<MaybeNodeClient>,
    pub(super) kzg: Kzg,
    pub(super) reduce_plotting_cpu_load: bool,
    pub(super) on_farm_initialized: OnFarmInitialized,
}

pub(super) async fn create_farmer<FarmIndex, OnFarmInitialized>(
    farmer_options: FarmerOptions<FarmIndex, OnFarmInitialized>,
) -> anyhow::Result<Farmer<FarmIndex>>
where
    FarmIndex:
        Hash + Eq + Copy + fmt::Display + fmt::Debug + TryFrom<usize> + Send + Sync + 'static,
    usize: From<FarmIndex>,
    OnFarmInitialized: Fn(FarmIndex),
{
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
        reduce_plotting_cpu_load,
        on_farm_initialized,
    } = farmer_options;

    if disk_farms.is_empty() {
        return Err(anyhow!("There must be at least one disk farm provided"));
    }

    for farm in &disk_farms {
        if !farm.directory.exists()
            && let Err(error) = fs::create_dir(&farm.directory)
        {
            return Err(anyhow!(
                "Directory {} doesn't exist and can't be created: {}",
                farm.directory.display(),
                error
            ));
        }
    }

    let plot_cache = !cfg!(windows)
        || disk_farms
            .iter()
            .map(|farm| farm.allocated_space)
            .sum::<u64>()
            <= MAX_SPACE_PLEDGED_FOR_PLOT_CACHE_ON_WINDOWS;

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

    let mut plotting_thread_pool_core_indices = thread_pool_core_indices(None, None);
    let replotting_thread_pool_core_indices = {
        let mut replotting_thread_pool_core_indices = thread_pool_core_indices(None, None);
        // The default behavior is to use all CPU cores, but for replotting we just want half
        replotting_thread_pool_core_indices
            .iter_mut()
            .for_each(|set| set.truncate(set.cpu_cores().len() / 2));
        replotting_thread_pool_core_indices
    };
    if reduce_plotting_cpu_load {
        plotting_thread_pool_core_indices = replotting_thread_pool_core_indices.clone();
    }

    if plotting_thread_pool_core_indices.len() > 1 {
        info!(
            l3_cache_groups = %plotting_thread_pool_core_indices.len(),
            "Multiple L3 cache groups detected"
        );
    }

    let plotting_thread_pools_count = plotting_thread_pool_core_indices.len();

    let downloading_semaphore =
        Arc::new(Semaphore::new(plotting_thread_pool_core_indices.len() + 1));

    let record_encoding_concurrency = {
        let cpu_cores = plotting_thread_pool_core_indices
            .first()
            .expect("Guaranteed to have some CPU cores; qed");

        NonZeroUsize::new((cpu_cores.cpu_cores().len() / 2).min(8)).unwrap_or(NonZeroUsize::MIN)
    };

    info!(
        ?plotting_thread_pool_core_indices,
        ?replotting_thread_pool_core_indices,
        "Preparing plotting thread pools"
    );

    let plotting_thread_pool_manager = create_plotting_thread_pool_manager(
        plotting_thread_pool_core_indices
            .into_iter()
            .zip(replotting_thread_pool_core_indices),
        Some(ThreadPriority::Min),
    )?;

    let global_mutex = Arc::default();

    let plotter = init_wgpu_plotter(
        piece_getter.clone(),
        Arc::clone(&global_mutex),
        kzg.clone(),
        erasure_coding.clone(),
    )
    .await?
    .map(|gpu_plotter| Arc::new(gpu_plotter) as Arc<dyn Plotter + Send + Sync>);

    let plotter = if let Some(plotter) = plotter {
        info!("CPU plotting was disabled due to detected faster plotting with GPU");

        plotter
    } else {
        let cpu_plotter = Arc::new(CpuPlotter::<_, FarmerPosTable>::new(
            piece_getter.clone(),
            downloading_semaphore,
            plotting_thread_pool_manager.clone(),
            record_encoding_concurrency,
            Arc::clone(&global_mutex),
            kzg.clone(),
            erasure_coding.clone(),
            None,
        ));
        Arc::new(cpu_plotter)
    };

    let (farms, plotting_delay_senders) = {
        let farms_total = disk_farms.len();
        let info_mutex = &AsyncMutex::new(());
        let (plotting_delay_senders, plotting_delay_receivers) = (0..farms_total)
            .map(|_| oneshot::channel())
            .unzip::<_, _, Vec<_>, Vec<_>>();
        let on_farm_initialized = &on_farm_initialized;

        let mut farms = Vec::with_capacity(farms_total);
        let mut farms_stream = disk_farms
            .into_iter()
            .zip(plotting_delay_receivers)
            .enumerate()
            .map(|(farm_index, (disk_farm, plotting_delay_receiver))| {
                let node_client = node_client.clone();
                let farmer_app_info = farmer_app_info.clone();
                let max_pieces_in_sector = farmer_app_info.protocol_info.max_pieces_in_sector;
                let kzg = kzg.clone();
                let erasure_coding = erasure_coding.clone();
                let plotter = Arc::clone(&plotter);
                let global_mutex = Arc::clone(&global_mutex);

                async move {
                    let farm_fut = SingleDiskFarm::new::<_, FarmerPosTable>(
                        SingleDiskFarmOptions {
                            directory: disk_farm.directory.clone(),
                            farmer_app_info,
                            allocated_space: disk_farm.allocated_space,
                            max_pieces_in_sector,
                            node_client,
                            reward_address,
                            plotter,
                            kzg,
                            erasure_coding,
                            cache_percentage: CACHE_PERCENTAGE.get(),
                            farming_thread_pool_size: recommended_number_of_farming_threads(),
                            plotting_delay: Some(plotting_delay_receiver),
                            global_mutex,
                            max_plotting_sectors_per_farm: MAX_PLOTTING_SECTORS_PER_FARM,
                            disable_farm_locking: false,
                            read_sector_record_chunks_mode:
                                ReadSectorRecordChunksMode::ConcurrentChunks,
                            registry: None,
                            create: true,
                        },
                        farm_index,
                    );

                    let farm = match farm_fut.await {
                        Ok(farm) => farm,
                        Err(SingleDiskFarmError::InsufficientAllocatedSpace {
                            min_space,
                            allocated_space,
                        }) => {
                            return (
                                farm_index,
                                Err(anyhow::anyhow!(
                                    "Allocated space {} ({}) is not enough, minimum is ~{} (~{}, \
                                    {} bytes to be exact)",
                                    bytesize::to_string(allocated_space, true),
                                    bytesize::to_string(allocated_space, false),
                                    bytesize::to_string(min_space, true),
                                    bytesize::to_string(min_space, false),
                                    min_space
                                )),
                            );
                        }
                        Err(error) => {
                            return (farm_index, Err(error.into()));
                        }
                    };

                    let _info_guard = info_mutex.lock().await;

                    let info = farm.info();
                    info!("Farm {farm_index}:");
                    info!("  ID: {}", info.id());
                    info!("  Genesis hash: 0x{}", hex::encode(info.genesis_hash()));
                    info!("  Public key: 0x{}", hex::encode(info.public_key()));
                    info!(
                        "  Allocated space: {} ({})",
                        bytesize::to_string(info.allocated_space(), true),
                        bytesize::to_string(info.allocated_space(), false)
                    );
                    info!("  Directory: {}", disk_farm.directory.display());

                    {
                        let Ok(farm_index) = farm_index.try_into() else {
                            return (
                                farm_index,
                                Err(anyhow!(
                                    "More than 256 plots are not supported by Space Acres"
                                )),
                            );
                        };
                        on_farm_initialized(farm_index);
                    }
                    (farm_index, Ok(farm))
                }
                .instrument(info_span!("", %farm_index))
            })
            .collect::<FuturesUnordered<_>>();

        while let Some((farm_index, farm)) = farms_stream.next().await {
            if let Err(error) = &farm {
                let span = info_span!("", %farm_index);
                let _span_guard = span.enter();

                error!(%error, "Single disk creation failed");
            }

            farms.push((farm_index, farm?));
        }

        // Restore order after unordered initialization
        farms.sort_unstable_by_key(|(farm_index, _farm)| *farm_index);

        let farms = farms
            .into_iter()
            .map(|(_farm_index, farm)| farm)
            .collect::<Vec<_>>();

        (farms, plotting_delay_senders)
    };

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
            farms
                .iter()
                .map(|farm| Arc::new(farm.piece_cache()) as Arc<_>)
                .collect(),
            if plot_cache {
                farms
                    .iter()
                    .map(|farm| Arc::new(farm.plot_cache()) as Arc<_>)
                    .collect()
            } else {
                Vec::new()
            },
        )
        .await;

    info!("Collecting already plotted pieces (this will take some time)...");

    // Collect already plotted pieces
    let mut initial_farm_states = Vec::with_capacity(farms.len());

    for (farm_index, farm) in farms.iter().enumerate() {
        let mut plotted_pieces = plotted_pieces.write().await;
        let farm_index = farm_index
            .try_into()
            .map_err(|_error| anyhow!("More than 256 plots are not supported by Space Acres"))?;

        plotted_pieces.add_farm(farm_index, Arc::new(farm.piece_reader()));

        let total_sectors_count = farm.total_sectors_count();
        let mut plotted_sectors_count = 0;
        let plotted_sectors = farm.plotted_sectors();
        let mut plotted_sectors = plotted_sectors.get().await.map_err(|error| {
            anyhow!("Failed to get plotted sectors for farm {farm_index}: {error}")
        })?;

        while let Some(plotted_sector_result) = plotted_sectors.next().await {
            plotted_sectors_count += 1;
            plotted_pieces.add_sector(
                farm_index,
                &plotted_sector_result.map_err(|error| {
                    anyhow!(
                        "Failed reading plotted sector on startup for farm {farm_index}: {error}"
                    )
                })?,
            )
        }

        initial_farm_states.push(InitialFarmState {
            total_sectors_count,
            plotted_sectors_count,
        });
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

    let mut farms_stream = farms
        .into_iter()
        .enumerate()
        .map(|(farm_index, farm)| {
            let Ok(farm_index) = FarmIndex::try_from(farm_index) else {
                unreachable!(
                    "More than 256 plots are not supported, this is checked above already; qed"
                );
            };
            let plotted_pieces = Arc::clone(&plotted_pieces);
            let span = info_span!("farm", %farm_index);

            farm.on_sector_update(Arc::new({
                let notifications = Arc::clone(&notifications);

                move |(sector_index, sector_update)| {
                    notifications.call_simple(&FarmerNotification::SectorUpdate {
                        farm_index,
                        sector_index: *sector_index,
                        update: sector_update.clone(),
                    });
                }
            }))
            .detach();
            farm.on_farming_notification(Arc::new({
                let notifications = Arc::clone(&notifications);

                move |notification| {
                    notifications.call_simple(&FarmerNotification::FarmingNotification {
                        farm_index,
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
                        let mut plotted_pieces = plotted_pieces.write_blocking();

                        if let Some(old_plotted_sector) = &maybe_old_plotted_sector {
                            plotted_pieces.delete_sector(farm_index, old_plotted_sector);
                        }
                        plotted_pieces.add_sector(farm_index, plotted_sector);
                    }
                };
            farm.on_sector_update(Arc::new(move |(_sector_index, sector_state)| {
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

            farm.run().map(move |result| (farm_index, result))
        })
        .collect::<FuturesUnordered<_>>();

    // Drop original instance such that the only remaining instances are in `SingleDiskFarm`
    // event handlers
    drop(plotted_pieces);

    let (action_sender, mut action_receiver) = mpsc::channel(1);
    let (pause_plotting_sender, mut pause_plotting_receiver) = watch::channel(false);

    let pause_plotting_actions_fut = async move {
        let mut thread_pools = Vec::with_capacity(plotting_thread_pools_count);

        loop {
            if *pause_plotting_receiver.borrow_and_update() {
                // Collect all managers so that plotting will be effectively paused
                if thread_pools.len() < plotting_thread_pools_count {
                    thread_pools.push(plotting_thread_pool_manager.get_thread_pools().await);
                    // Allow to un-pause plotting quickly if user requests it
                    continue;
                }
            } else {
                // Returns all thread pools back to the manager
                thread_pools.clear();
            }

            if pause_plotting_receiver.changed().await.is_err() {
                break;
            }
        }
    };

    let process_actions_fut = async move {
        while let Some(action) = action_receiver.next().await {
            match action {
                FarmerAction::PausePlotting(pause_plotting) => {
                    if let Err(error) = pause_plotting_sender.send(pause_plotting) {
                        debug!(%error, "Failed to forward pause plotting");
                    }
                }
            }
        }
        anyhow::Ok(())
    };

    let mut farm_errors = Vec::new();

    let farms_fut = {
        let notifications = Arc::clone(&notifications);

        async move {
            while let Some((farm_index, result)) = farms_stream.next().await {
                match result {
                    Ok(()) => {
                        info!(%farm_index, "Farm exited successfully");
                    }
                    Err(error) => {
                        error!(%farm_index, %error, "Farm exited with error");

                        let error = Arc::new(error);

                        farm_errors.push(AsyncJoinOnDrop::new(
                            tokio::spawn({
                                let error = Arc::clone(&error);

                                async move {
                                    loop {
                                        tokio::time::sleep(FARM_ERROR_PRINT_INTERVAL).await;

                                        error!(
                                            %farm_index,
                                            %error,
                                            "Farm errored and stopped"
                                        );
                                    }
                                }
                            }),
                            true,
                        ));

                        notifications
                            .call_simple(&FarmerNotification::FarmError { farm_index, error });
                    }
                }
            }

            pending::<()>().await;
        }
    };

    let farmer_fut = Box::pin(
        async move {
            select! {
                _ = pause_plotting_actions_fut.fuse() => {
                    Ok(())
                }
                _ = process_actions_fut.fuse() => {
                    Ok(())
                }
                _ = farms_fut.fuse() => {
                    Ok(())
                }
            }
        }
        .in_current_span(),
    );

    anyhow::Ok(Farmer {
        farmer_fut,
        farmer_cache_worker_fut,
        initial_farm_states,
        notifications,
        action_sender,
    })
}

fn select_gpu_devices(device_types: &[DeviceType]) -> Vec<usize> {
    let has_dgpu = device_types.contains(&DeviceType::DiscreteGpu);
    let has_igpu = device_types.contains(&DeviceType::IntegratedGpu);

    device_types
        .iter()
        .enumerate()
        .filter_map(|(index, &device_type)| {
            let use_device = match device_type {
                DeviceType::DiscreteGpu => true,
                DeviceType::IntegratedGpu => !has_dgpu,
                DeviceType::VirtualGpu => !has_dgpu && !has_igpu,
                DeviceType::Other | DeviceType::Cpu => false,
            };
            use_device.then_some(index)
        })
        .collect()
}

async fn init_wgpu_plotter<PG>(
    piece_getter: PG,
    global_mutex: Arc<AsyncMutex<()>>,
    kzg: Kzg,
    erasure_coding: ErasureCoding,
) -> anyhow::Result<Option<GpuPlotter<PG, WgpuRecordsEncoder>>>
where
    PG: PieceGetter + Clone + Send + Sync + 'static,
{
    let number_of_queues = |device_type: DeviceType| match device_type {
        DeviceType::DiscreteGpu => NonZeroU8::new(4).expect("Not zero; qed"),
        DeviceType::Other
        | DeviceType::IntegratedGpu
        | DeviceType::VirtualGpu
        | DeviceType::Cpu => NonZeroU8::new(2).expect("Not zero; qed"),
    };
    let (skipped_metal, all_gpu_devices): (Vec<_>, Vec<_>) = Device::enumerate(number_of_queues)
        .await
        .into_iter()
        .partition(|device| device.backend() == Backend::Metal);
    if !skipped_metal.is_empty() {
        warn!(
            count = skipped_metal.len(),
            "Skipping Metal GPU(s); GPU plotting on Metal is untested"
        );
    }

    let device_types = all_gpu_devices
        .iter()
        .map(Device::device_type)
        .collect::<Vec<_>>();
    let used_indices = select_gpu_devices(&device_types);

    for (index, device) in all_gpu_devices.iter().enumerate() {
        if used_indices.contains(&index) {
            continue;
        }
        match device.device_type() {
            DeviceType::Other => debug!(?device, "Skipping an unknown GPU device type"),
            DeviceType::IntegratedGpu => debug!(?device, "Skipping iGPU in presence of dGPU"),
            DeviceType::VirtualGpu => {
                debug!(
                    ?device,
                    "Skipping virtualized GPU in presence of iGPU or dGPU"
                )
            }
            DeviceType::Cpu => debug!(?device, "Skipping GPU device emulated by the CPU"),
            DeviceType::DiscreteGpu => {}
        }
    }

    let used_gpu_devices = all_gpu_devices
        .into_iter()
        .enumerate()
        .filter_map(|(index, device)| used_indices.contains(&index).then_some(device))
        .collect::<Vec<_>>();

    if used_gpu_devices.is_empty() {
        debug!("No GPU devices were found or used");
        return Ok(None);
    }

    info!("Using GPUs:");
    for device in &used_gpu_devices {
        let device_type = match device.device_type() {
            DeviceType::Other => "other",
            DeviceType::IntegratedGpu => "Integrated GPU",
            DeviceType::DiscreteGpu => "Discrete GPU",
            DeviceType::VirtualGpu => "Virtual GPU",
            DeviceType::Cpu => "CPU emulation",
        };
        info!("{}: {} ({device_type})", device.id(), device.name());
    }

    let downloading_semaphore = Arc::new(Semaphore::new(used_gpu_devices.len() * 3));

    let gpu_plotter = GpuPlotter::new(
        piece_getter,
        downloading_semaphore,
        used_gpu_devices
            .into_iter()
            .map(|wgpu_device| {
                let id = wgpu_device.id();
                let queue_devices = wgpu_device
                    .create_proofs_encoder_instances()
                    .into_iter()
                    .map(|instance| WgpuDevice::new(instance, erasure_coding.clone()))
                    .collect();
                WgpuRecordsEncoder::new(id, queue_devices, Arc::clone(&global_mutex))
            })
            .collect::<Result<_, _>>()
            .map_err(|error| anyhow::anyhow!("Failed to create wgpu records encoder: {error}"))?,
        global_mutex,
        kzg,
        erasure_coding,
        None,
    )
    .map_err(|error| anyhow::anyhow!("Failed to initialize wgpu plotter: {error}"))?;

    Ok(Some(gpu_plotter))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_device_selection() {
        use DeviceType::{Cpu, DiscreteGpu, IntegratedGpu, Other, VirtualGpu};

        assert_eq!(
            select_gpu_devices(&[DiscreteGpu, IntegratedGpu]),
            vec![0],
            "dGPU wins over iGPU"
        );
        assert_eq!(
            select_gpu_devices(&[IntegratedGpu, IntegratedGpu]),
            vec![0, 1],
            "All iGPUs are used when there is no dGPU"
        );
        assert_eq!(
            select_gpu_devices(&[Cpu]),
            Vec::<usize>::new(),
            "CPU emulation is never auto-selected"
        );
        assert_eq!(
            select_gpu_devices(&[DiscreteGpu, DiscreteGpu, IntegratedGpu]),
            vec![0, 1],
            "All dGPUs are used, iGPU skipped"
        );
        assert_eq!(
            select_gpu_devices(&[IntegratedGpu, VirtualGpu]),
            vec![0],
            "Virtual GPU skipped while an iGPU is present"
        );
        assert_eq!(
            select_gpu_devices(&[VirtualGpu]),
            vec![0],
            "Virtual GPU used as the last resort"
        );
        assert_eq!(
            select_gpu_devices(&[Cpu, Other, VirtualGpu]),
            vec![2],
            "Virtual GPU still wins over unknown/CPU adapters as the last resort"
        );
        assert_eq!(
            select_gpu_devices(&[Cpu, Other]),
            Vec::<usize>::new(),
            "Unknown and CPU adapters alone select nothing"
        );
    }
}
