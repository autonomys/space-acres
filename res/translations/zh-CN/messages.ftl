welcome_title = 欢迎使用
welcome_message =
    Space Acres 是用于在 Autonomys 网络进行耕种的图形化用户界面.

    开始前请阅读并知悉以下信息:
    ✔ 需要准备一个钱包地址来收取奖励，该地址可由 Subwallet、polkadot 扩展或任何兼容 Substrate 链的钱包来生成
    ✔ 需要 100G 大小的高性能 SSD 硬盘来存储节点数据
    ✔ 尽你可能使用更大容量或多个的 SSD 硬盘来存储耕种数据，以获得更多奖励
welcome_button_continue = 继续

upgrade_title = 升级
upgrade_message =
    感谢再次使用 Space Acres!

    你在升级前存储的链数据不再与该 Space Acres 版本兼容，这可能是因为你之前参与了更早版本的 Autonomys 。

    别担心，你可以一键升级到当前最新网络
upgrade_button_upgrade = 升级到 {$chain_name}

loading_title = 加载中
loading_configuration_title = 加载配置
loading_configuration_step_loading = 加载配置...
loading_configuration_step_reading = 读取配置...
loading_configuration_step_configuration_exists = 检查配置...
loading_configuration_step_configuration_not_found = 检查配置...
loading_configuration_step_configuration_checking = 检查配置...
loading_configuration_step_configuration_valid = 配置文件校验通过
loading_configuration_step_decoding_chain_spec = chain spec 解析中...
loading_configuration_step_decoded_chain_spec = chain spec 解析成功
loading_networking_stack_title = 初始化网络工作栈
loading_networking_stack_step_checking_node_path = 检查节点目录...
loading_networking_stack_step_creating_node_path = 创建节点目录...
loading_networking_stack_step_node_path_ready = 节点目录准备完毕
loading_networking_stack_step_preparing = 准备网络工作栈...
loading_networking_stack_step_reading_keypair = 读取网络密钥对...
loading_networking_stack_step_generating_keypair = 生成网络密钥对...
loading_networking_stack_step_writing_keypair_to_disk = 写入网络密钥对到磁盘...
loading_networking_stack_step_instantiating = 实例化网络工作栈...
loading_networking_stack_step_created_successfully = 网络工作栈创建成功
loading_consensus_node_title = 初始化共识节点
loading_consensus_node_step_creating = 创建共识节点...
loading_consensus_node_step_created_successfully = 共识节点创建成功
loading_farmer_title = 实例化农民
loading_farmer_step_initializing = 初始化农场 {$index}/{$farms_total}...
loading_farmer_step_created_successfully = 农民创建成功
loading_wiping_farmer_data_title = 擦除农民数据
loading_wiping_farmer_data_step_wiping_farm = 擦除农场 {$index}/{$farms_total} at {$path}...
loading_wiping_farmer_data_step_success = 所有农场数据擦除成功
loading_wiping_node_data_title = 擦除节点数据
loading_wiping_node_data_step_wiping_node = 擦除该目录的节点数据 {$path}...
loading_wiping_node_data_step_success = 节点数据擦除成功

configuration_title = 配置
reconfiguration_title = 重新配置
configuration_node_path = 节点路径
configuration_node_path_placeholder = 示例: {$path}
configuration_node_path_tooltip = 节点文件的绝对路径，建议预留不少于 100G 的高性能 SSD 空间
configuration_node_path_button_select = 选择
configuration_node_path_error_doesnt_exist_or_write_permissions = 文件目录不存在或当前用户无写入权限
configuration_node_migrate_button = Migrate...
# configuration_node_migrate_button = 迁移...
configuration_node_migrate_tooltip = Migrate or reset node database
# configuration_node_migrate_tooltip = 迁移或重置节点数据库
configuration_node_size = Node size: {$size}
# configuration_node_size = 节点大小：{$size}
configuration_node_volume_free_space = Free space: {$size}
# configuration_node_volume_free_space = 可用空间：{$size}
configuration_reward_address = 奖励地址
configuration_reward_address_placeholder = 示例: {$address}
configuration_reward_address_tooltip = 使用 Subwallet 或 polkadot{"{"}.js{"}"} 扩展来创建地址，任何 Substrate 链的 SS58 格式地址都可用作奖励地址
configuration_reward_address_button_create_wallet = 创建钱包
configuration_reward_address_error_evm_address = 应使用 SS58 格式的 Substrate 地址，而不是 EVM 地址
configuration_farm = 农场 {$index} 的目录和大小
configuration_farm_path_placeholder = 示例: {$path}
configuration_farm_path_tooltip = 存储农场数据文件的绝对路径，可使用任何类型的 SSD
configuration_farm_path_button_select = 选择
configuration_farm_path_error_doesnt_exist_or_write_permissions = 文件目录不存在或当前用户无写入权限
configuration_farm_size_kind_fixed = 固定大小
configuration_farm_size_kind_free_percentage = % 空闲磁盘大小
configuration_farm_fixed_size_placeholder = 示例: 4T, 2.5TB, 500GiB, 等.
configuration_farm_fixed_size_tooltip = 农场单元的大小，可以使用任何大于 2GB 的值
configuration_farm_free_percentage_size_placeholder = 示例: 100%, 1.1%, 等.
configuration_farm_free_percentage_size_tooltip = 用于该农场的磁盘大小百分比，可以指定任意大小，但需保留最少2GB的剩余空间避免出现问题
configuration_farm_delete = 删除这个农场
configuration_advanced = 高级配置
configuration_advanced_farmer = 农民配置
configuration_advanced_farmer_reduce_plotting_cpu_load = 降低绘图时CPU负载
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = 初次绘图默认会使用所有CPU核心，这个选项可以在重新绘图时只占用50%的CPU来使系统响应其他任务更加流畅
configuration_advanced_network = 网络配置
configuration_advanced_network_default_port_number_tooltip = 默认端口是 {$port}
configuration_advanced_network_substrate_port = Substrate (blockchain) P2P 端口 (TCP):
configuration_advanced_network_subspace_port = Subspace (DSN) P2P 端口 (TCP):
configuration_advanced_network_faster_networking = 快速网络:
configuration_advanced_network_faster_networking_tooltip = 默认的网络配置已为消费级路由优化，但如果你有高性能的配置，快速网络设置可能提升节点同步速度和其他流程
configuration_button_add_farm = 新增农场
configuration_button_help = 帮助
configuration_button_cancel = 取消
configuration_button_back = 返回
configuration_button_save = 保存
configuration_button_start = 开始
configuration_dialog_button_select = 选择
configuration_dialog_button_cancel = 取消

node_migration_button_cancel = Cancel
# node_migration_button_cancel = 取消
node_migration_button_reset = Reset Node
# node_migration_button_reset = 重置节点
node_migration_button_start = Start Migration
# node_migration_button_start = 开始迁移
node_migration_destination_free_space = Free space: {$size}
# node_migration_destination_free_space = 可用空间：{$size}
node_migration_destination_label = New node location:
# node_migration_destination_label = 新节点位置：
node_migration_destination_placeholder = Select destination folder
# node_migration_destination_placeholder = 选择目标文件夹
node_migration_dialog_title = Migrate Node Database
# node_migration_dialog_title = 迁移节点数据库
node_migration_insufficient_space_warning = Warning: Not enough free space at destination
# node_migration_insufficient_space_warning = 警告：目标位置空间不足
node_migration_mode_fresh_sync = Fresh sync to new location
# node_migration_mode_fresh_sync = 在新位置重新同步
node_migration_mode_fresh_sync_explanation = Syncs a fresh database from the network at the new location. Often faster than migrating, especially if your node is out of sync, and requires less destination space.
# node_migration_mode_fresh_sync_explanation = 在新位置从网络同步一个新数据库。通常比迁移更快，特别是当您的节点不同步时，并且需要更少的目标空间。
node_migration_mode_migrate = Migrate database
# node_migration_mode_migrate = 迁移数据库
node_migration_mode_migrate_explanation = Moves the existing database to the new location. Requires enough destination space for the current database.
# node_migration_mode_migrate_explanation = 将现有数据库移动到新位置。需要目标位置有足够的空间存放当前数据库。
node_migration_mode_reset = Reset and resync in place
# node_migration_mode_reset = 原地重置并重新同步
node_migration_mode_reset_explanation = Resets your node by wiping the database and syncing fresh from the network. Use this if your node database is corrupted or significantly out of sync.
# node_migration_mode_reset_explanation = 通过删除数据库并从网络重新同步来重置您的节点。如果您的节点数据库已损坏或严重不同步，请使用此选项。
node_migration_non_node_data_warning = Note: Non-node data detected in this directory and will not be migrated
# node_migration_non_node_data_warning = 注意：在此目录中检测到非节点数据，这些数据将不会被迁移
node_migration_source_label = Current location:
# node_migration_source_label = 当前位置：
node_migration_status_completed = Migration completed successfully!
# node_migration_status_completed = 迁移成功完成！
node_migration_status_copying = Migrating database: {$percentage}%
# node_migration_status_copying = 正在迁移数据库：{$percentage}%
node_migration_status_deleting_source = Removing previous database...
# node_migration_status_deleting_source = 正在删除旧数据库...
node_migration_status_failed = Migration failed: {$error}
# node_migration_status_failed = 迁移失败：{$error}
node_migration_status_restarting = Restarting Space Acres...
# node_migration_status_restarting = 正在重启 Space Acres...
node_migration_status_shutting_down = Shutting down node...
# node_migration_status_shutting_down = 正在关闭节点...
node_migration_status_updating_config = Updating configuration...
# node_migration_status_updating_config = 正在更新配置...
node_migration_status_verifying = Verifying database...
# node_migration_status_verifying = 正在验证数据库...
node_migration_title = Migrating Node Database
# node_migration_title = 正在迁移节点数据库

running_title = 运行中
running_node_title = {$chain_name} 共识节点
running_node_title_tooltip = 在文件管理器中打开
running_node_free_disk_space_tooltip = 空闲磁盘大小: {$size}
running_node_connections_tooltip = {$connected_peers}/{$expected_peers} 节点已连接, 点击查看所需 P2P 端口
running_node_status_connecting = 连接网络中，最新区块 #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s (预计 ~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} 小时)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s (预计 ~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} 分钟)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s (预计 ~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} 秒)
running_node_status_syncing =
    {$sync_kind ->
        [dsn] DSN 网络同步
        [regular] 常规同步
        *[unknown] 未知同步类型 {$sync_kind}
    } #{$best_block_number}/{$target_block}{$sync_speed}
running_node_status_synced = 同步完成, 最新区块 #{$best_block_number}
running_farmer_title = 农民
running_farmer_button_expand_details = 各农场的详细信息
running_farmer_button_pause_plotting = 暂停绘图/重新绘图，当前的编码扇区不会被中断
running_farmer_button_resume_plotting = 继续绘图
running_farmer_account_balance_tooltip = 自启动以来耕种到的总奖励币，点击在 Astral 中查看更多详细信息
running_farmer_piece_cache_sync = Piece缓存同步 {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    下次奖励预计: {$eta_string ->
        [any_time_now] 即将发生
        [less_than_an_hour] 1小时内
        [today] 今天
        [this_week] 这周
        [more_than_a_week] 一周以上
        *[unknown] 未知
    }
running_farmer_farm_tooltip = 在文件管理器中打开
running_farmer_farm_reward_signatures_tooltip = {$successful_signatures}/{$total_signatures} 奖励签名成功，打开农场查看更多信息
running_farmer_farm_auditing_performance_tooltip = 审计性能: 平均时长 {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}秒, 时间限制 {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}秒
running_farmer_farm_proving_performance_tooltip = 证明性能: 平均时长 {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}秒, 时间限制 {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}秒
running_farmer_farm_non_fatal_error_tooltip = 非致命错误发生并已经恢复，在日志中查看更多信息: {$error}
running_farmer_farm_crashed = 农场崩溃: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} 分钟/扇区, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} 扇区/小时)
running_farmer_farm_plotting_initial =
    {$pausing_state ->
        [pausing] 初始绘制暂停中
        [paused] 初始绘制暂停
        *[no] 初始绘制中
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] 耕种中
        *[no] 未耕种
    }
running_farmer_farm_replotting =
    {$pausing_state ->
        [pausing] 初始绘制暂停中
        [paused] 初始绘制暂停
        *[default] 重新绘制
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] 耕种中
        *[no] 未耕种
    }
running_farmer_farm_farming = 耕种
running_farmer_farm_waiting_for_node_to_sync = 等待节点同步
running_farmer_farm_sector = 扇区 {$sector_index}
running_farmer_farm_sector_up_to_date = 扇区 {$sector_index}: 有效
running_farmer_farm_sector_waiting_to_be_plotted = 扇区 {$sector_index}: 等待绘制
running_farmer_farm_sector_about_to_expire = 扇区 {$sector_index}: 即将过期等待重新绘制
running_farmer_farm_sector_expired = 扇区 {$sector_index}: 已过期等待重新绘制
running_farmer_farm_sector_downloading = 扇区 {$sector_index}: 下载中
running_farmer_farm_sector_encoding = 扇区 {$sector_index}: 编码中
running_farmer_farm_sector_writing = 扇区 {$sector_index}: 写入中

shutting_down_title = 关闭
shutting_down_description = 这可能需要几秒到几分钟的时间，取决于你运行的程序

stopped_title = 暂停
stopped_message = 暂停 🛑
stopped_message_with_error = 由于错误暂停: {$error}
stopped_button_show_logs = 打开日志
stopped_button_help_from_community = 获得社区支持

error_title = 错误
error_message = 错误: {$error}
error_message_failed_to_send_config_to_backend = 发送到后端过程出错: {$error}
error_message_failed_to_send_pause_plotting_to_backend = 发送暂停任务到后端出错: {$error}
error_button_help_from_community = 获得社区支持
# error_button_help_from_community = 获得社区支持
error_button_reset_node = Reset node
# error_button_reset_node = 重置节点
error_button_reset_node_tooltip = Wipe node data and sync fresh from the network
# error_button_reset_node_tooltip = 清除节点数据并从网络重新同步
error_button_show_logs = 打开日志
# error_button_show_logs = 打开日志

new_version_available = 版本 {$version} 可用 🎉
new_version_available_button_open = 打开版本发布页面

main_menu_show_logs = 在文件管理器中打开日志
main_menu_change_configuration = 修改配置
main_menu_share_feedback = 分享反馈
main_menu_about = 关于
main_menu_exit = 退出

status_bar_message_configuration_is_invalid = 配置不可用: {$error}
status_bar_message_restart_is_needed_for_configuration = 重启以使配置修改生效
status_bar_message_failed_to_save_configuration = 保存配置修改失败: {$error}
status_bar_message_restarted_after_crash = Space Acres在崩溃后自动重启，请在日志中查看详细信息
status_bar_button_migrate = Migrate
# status_bar_button_migrate = 迁移
status_bar_button_ok = 正常
status_bar_button_restart = 重启

about_system_information =
    配置目录: {$config_directory}
    数据目录 (包括日志): {$data_directory}

tray_icon_open = 打开
tray_icon_quit = 退出

notification_app_minimized_to_tray = Space Acres已最小化到托盘
    .body = 你可以关闭或从托盘中重新打开
notification_stopped_with_error = Space Acres由于错误暂停
    .body = 出现一个错误，需要手动解决
notification_farm_error = 一个Space Acres农场出错
    .body = 出现一个错误，需要手动解决
notification_node_low_disk_space = Low Node Disk Space
    .body = Node volume has only {$free_space} remaining
# notification_node_low_disk_space = 节点磁盘空间不足
#     .body = 节点卷仅剩余 {$free_space}
notification_missed_reward = 签署奖励失败 😞
    .body = 很不幸，但很快就会有下一次机会
notification_signed_reward_successfully = 成功签署一份奖励 🥳
    .body = 感谢参与 🙌

warning_low_disk_space = Low disk space on node volume
# warning_low_disk_space = 节点卷磁盘空间不足
warning_low_disk_space_detail = Only {$free_space} remaining. Consider migrating to a larger drive.
# warning_low_disk_space_detail = 仅剩 {$free_space}。请考虑迁移到更大的驱动器。
