welcome_title = Добро Пожаловать!
welcome_message =
    Space Acres это приложение с графическим интерфейсом для фарминга в Autonomys Network.

    Прежде чем продолжить, проверьте что у вас есть следующие 3 вещи:
    ✔ Адрес кошелька на который будут поступать награды (можно использовать Subwallet, polkadot{"{"}.js{"}"} расширение для браузера или любой другой кошелек который совместим с блокчейнами Substrate)
    ✔ 100 Гигабайт свободного места на диске SSD, чтобы хранить данные узла
    ✔ Любой SSD (или несколько) с как можно большим количества свободного места на диске для фарминга. Это и будет генерировать награды."
welcome_button_continue = Продолжить

upgrade_title = Обновить
upgrade_message =
    Спасибо за выбор Space Acres!

    Узел которым вы ранее управляли более не совместим с текущей версией приложения Space Acres, вероятно потому что вы участвовали в предыдущих версиях Autonomys Network.

    Но ничего страшного! Вы можете обновить приложение к текущей поддерживаемое версии всего в одно нажатие клавиши!"
upgrade_button_upgrade = Обновиться на {$chain_name}

loading_title = Загрузка
loading_configuration_title = Загрузка конфигурации
loading_configuration_step_loading = Загружаю конфигурации...
loading_configuration_step_reading = Считываю конфигурацию...
loading_configuration_step_configuration_exists = Считываю конфигурацию...
loading_configuration_step_configuration_not_found = Считываю конфигурацию...
loading_configuration_step_configuration_checking = Считываю конфигурацию...
loading_configuration_step_configuration_valid = Верная конфигурация
loading_configuration_step_decoding_chain_spec = Decoding chain specification...
loading_configuration_step_decoded_chain_spec = Decoded chain specification successfully
loading_networking_stack_title = Initializing networking stack
loading_networking_stack_step_checking_node_path = Checking node path...
loading_networking_stack_step_creating_node_path = Creating node path...
loading_networking_stack_step_node_path_ready = Node path ready
loading_networking_stack_step_preparing = Preparing networking stack...
loading_networking_stack_step_reading_keypair = Reading network keypair...
loading_networking_stack_step_generating_keypair = Generating network keypair...
loading_networking_stack_step_writing_keypair_to_disk = Writing network keypair to disk...
loading_networking_stack_step_instantiating = Instantiating networking stack...
loading_networking_stack_step_created_successfully = Networking stack created successfully
loading_consensus_node_title = Initializing consensus node
loading_consensus_node_step_creating = Creating consensus node...
loading_consensus_node_step_created_successfully = Consensus node created successfully
loading_farmer_title = Instantiating farmer
loading_farmer_step_initializing = Initializing farms {$index}/{$farms_total}...
loading_farmer_step_created_successfully = Farmer created successfully
loading_wiping_farmer_data_title = Wiping farmer data
loading_wiping_farmer_data_step_wiping_farm = Wiping farm {$index}/{$farms_total} at {$path}...
loading_wiping_farmer_data_step_success = All farms wiped successfully
loading_wiping_node_data_title = Wiping node data
loading_wiping_node_data_step_wiping_node = Wiping node at {$path}...
loading_wiping_node_data_step_success = Node data wiped successfully

configuration_title = Configuration
reconfiguration_title = Reconfiguration
configuration_node_path = Node path
configuration_node_path_placeholder = Example: {$path}
configuration_node_path_tooltip = Absolute path where node files will be stored, prepare to dedicate at least 100 GiB of space for it, good quality SSD recommended
configuration_node_path_button_select = Select
configuration_node_path_error_doesnt_exist_or_write_permissions = Folder doesn't exist or user is lacking write permissions
configuration_reward_address = Rewards address
configuration_reward_address_placeholder = Example: {$address}
configuration_reward_address_tooltip = Use Subwallet or polkadot{"{"}.js{"}"} extension or any other Substrate wallet to create it first (address for any Substrate chain in SS58 format works)
configuration_reward_address_button_create_wallet = Create wallet
configuration_reward_address_error_evm_address = This should be a Substrate (SS58) address (any chain will do), not EVM address
configuration_farm = Path to farm {$index} and its size
configuration_farm_path_placeholder = Example: {$path}
configuration_farm_path_tooltip = Absolute path where farm files will be stored, any SSD works, high endurance not necessary
configuration_farm_path_button_select = Select
configuration_farm_path_error_doesnt_exist_or_write_permissions = Folder doesn't exist or user is lacking write permissions
configuration_farm_size_kind_fixed = Fixed size
configuration_farm_size_kind_free_percentage = % of free disk space
configuration_farm_fixed_size_placeholder = Example: 4T, 2.5TB, 500GiB, etc.
configuration_farm_fixed_size_tooltip = Size of the farm in whichever units you prefer, any amount of space above 2 GB works
configuration_farm_free_percentage_size_placeholder = Example: 100%, 1.1%, etc.
configuration_farm_free_percentage_size_tooltip = Percentage of free disk space to occupy by this farm, anything above 0% works, but at least 2 GB of free space should remain on disk to avoid errors
configuration_farm_delete = Delete this farm
configuration_advanced = Advanced configuration
configuration_advanced_farmer = Farmer configuration
configuration_advanced_farmer_reduce_plotting_cpu_load = Reduce plotting CPU load
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = Initial plotting uses all CPU cores by default, while with this option it will start using half of the cores like replotting, improving system responsiveness for other tasks
configuration_advanced_network = Network configuration
configuration_advanced_network_default_port_number_tooltip = Default port number is {$port}
configuration_advanced_network_substrate_port = Substrate (blockchain) P2P port (TCP):
configuration_advanced_network_subspace_port = Subspace (DSN) P2P port (TCP):
configuration_advanced_network_faster_networking = Faster networking:
configuration_advanced_network_faster_networking_tooltip = By default networking is optimized for consumer routers, but if you have more powerful setup, faster networking may improve sync speed and other processes
configuration_button_add_farm = Add farm
configuration_button_help = Help
configuration_button_cancel = Cancel
configuration_button_back = Back
configuration_button_save = Save
configuration_button_start = Start
configuration_dialog_button_select = Select
configuration_dialog_button_cancel = Cancel

running_title = Running
running_node_title = {$chain_name} consensus node
running_node_title_tooltip = Click to open in file manager
running_node_free_disk_space_tooltip = Free disk space: {$size} remaining
running_node_status_connecting = Connecting to the network, best block #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} hours remaining)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} minutes remaining)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} seconds remaining)
running_node_status_syncing =
    {$sync_kind ->
        [dsn] Syncing from DSN
        [regular] Regular sync
        *[unknown] Unknown sync kind {$sync_kind}
    } #{$best_block_number}/{$target_block}{$sync_speed}
running_node_status_synced = Synced, best block #{$best_block_number}
running_farmer_title = Farmer
running_farmer_button_expand_details = Expand details about each farm
running_farmer_button_pause_plotting = Pause plotting/replotting, note that currently encoding sectors will not be interrupted
running_farmer_account_balance_tooltip = Total account balance and coins farmed since application started, click to see details in Astral
running_farmer_piece_cache_sync = Piece cache sync {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Next reward estimate: {$eta_string ->
        [any_time_now] any time now
        [less_than_an_hour] less than an hour
        [today] today
        [this_week] this week
        [more_than_a_week] more than a week
        *[unknown] unknown
    }
running_farmer_farm_tooltip = Click to open in file manager
running_farmer_farm_reward_signatures_tooltip = {$successful_signatures}/{$total_signatures} successful reward signatures, expand farm details to see more information
running_farmer_farm_auditing_performance_tooltip = Auditing performance: average time {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, time limit {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_proving_performance_tooltip = Proving performance: average time {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, time limit {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_non_fatal_error_tooltip = Non-fatal farming error happened and was recovered, see logs for more details: {$error}
running_farmer_farm_crashed = Farm crashed: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} m/sector, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} sectors/h)
running_farmer_farm_plotting_initial =
    {$pausing_state ->
        [pausing] Pausing initial plotting
        [paused] Paused initial plotting
        *[no] Initial plotting
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] farming
        *[no] not farming
    }
running_farmer_farm_replotting =
    {$pausing_state ->
        [pausing] Pausing initial plotting
        [paused] Paused initial plotting
        *[default] Replotting
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] farming
        *[no] not farming
    }
running_farmer_farm_farming = Farming
running_farmer_farm_waiting_for_node_to_sync = Waiting for node to sync
running_farmer_farm_sector = Sector {$sector_index}
running_farmer_farm_sector_up_to_date = Sector {$sector_index}: up to date
running_farmer_farm_sector_waiting_to_be_plotted = Sector {$sector_index}: waiting to be plotted
running_farmer_farm_sector_about_to_expire = Sector {$sector_index}: about to expire, waiting to be replotted
running_farmer_farm_sector_expired = Sector {$sector_index}: expired, waiting to be replotted
running_farmer_farm_sector_downloading = Sector {$sector_index}: downloading
running_farmer_farm_sector_encoding = Sector {$sector_index}: encoding
running_farmer_farm_sector_writing = Sector {$sector_index}: writing

shutting_down_title = Shutting down
shutting_down_description = This may take a couple of seconds to a few minutes depending on what application is doing

stopped_title = Stopped
stopped_message = Stopped 🛑
stopped_message_with_error = Stopped with error: {$error}
stopped_button_show_logs = Show logs
stopped_button_help_from_community = Help from community

error_title = Error
error_message = Error: {$error}
error_message_failed_to_send_config_to_backend = Failed to send config to backend: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Failed to send pause plotting to backend: {$error}
error_button_show_logs = Show logs
error_button_help_from_community = Help from community

new_version_available = Version {$version} available 🎉
new_version_available_button_open = Open releases page

main_menu_show_logs = Show logs in file manager
main_menu_change_configuration = Change configuration
main_menu_share_feedback = Share feedback
main_menu_about = About
main_menu_exit = Exit

status_bar_message_configuration_is_invalid = Configuration is invalid: {$error}
status_bar_message_restart_is_needed_for_configuration = Application restart is needed for configuration changes to take effect
status_bar_message_failed_to_save_configuration = Failed to save configuration changes: {$error}
status_bar_message_restarted_after_crash = Space Acres automatically restarted after crash, check application and system logs for details
status_bar_button_restart = Restart
status_bar_button_ok = Ok

about_system_information =
    Config directory: {$config_directory}
    Data directory (including logs): {$data_directory}

tray_icon_open = Open
tray_icon_close = Close

notification_app_minimized_to_tray = Space Acres was minimized to tray
    .body = You can open it again or exit completely using tray icon menu
notification_stopped_with_error = Space Acres stopped with error
    .body = An error happened and requires user intervention to resolve
notification_farm_error = One of the farms failed in Space Acres
    .body = An error happened and requires user intervention to resolve
notification_signed_reward_successfully = Signed new reward successfully 🥳
    .body = Thank you for securing the network 🙌
notification_missed_reward = Reward signing failed 😞
    .body = This is unfortunate, but there will be another chance soon
