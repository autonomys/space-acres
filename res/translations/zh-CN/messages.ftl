welcome_title = 欢迎使用
welcome_message =
    Space Acres 是用于在 Autonomys 网络进行耕种的图形化用户界面.

    开始前请阅读并知悉以下信息:
    ✔ 需要准备一个钱包地址来收取奖励，该地址可由 Subwallet、polkadot 扩展或任何兼容 Substrate 链的钱包来生成
    ✔ 需要 100G 大小的高性能 SSD 硬盘来存储节点数据
    ✔ 尽你可能使用更大容量或多个的 SSD 硬盘来存储耕种数据，以获得更多奖励”
welcome_button_continue = 继续

upgrade_title = 升级
upgrade_message =
    感谢再次使用 Space Acres!

    你在升级前存储的链数据不再与该 Space Acres 版本兼容，这可能是因为你之前参与了更早版本的 Autonomys 。

    别担心，你可以一键升级到当前最新网络”
upgrade_button_upgrade = 升级到 {$chain_name}

loading_title = 加载中
loading_configuration_title = 加载配置
loading_configuration_step_loading = 加载配置...
loading_configuration_step_reading = 读取配置...
loading_configuration_step_configuration_exists = 读取配置...
loading_configuration_step_configuration_not_found = 读取配置...
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
loading_consensus_node_title = 初始化网络工作栈
loading_consensus_node_step_creating = 创建共识节点...
loading_consensus_node_step_created_successfully = 共识节点创建成功
loading_farmer_title = 实例化农民
loading_farmer_step_initializing = 初始化农场 {$a_index}/{$b_farms_total}...
loading_farmer_step_created_successfully = 农民创建成功
loading_wiping_farmer_data_title = 擦除农民数据
loading_wiping_farmer_data_step_wiping_farm = 擦除农场 {$a_index}/{$b_farms_total} at {$c_path}...
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
configuration_farm_size_placeholder = 示例: 4T, 2.5TB, 500GiB, 等.
configuration_farm_size_tooltip = 农场单元的大小，可以使用任何大于 2GB 的值
configuration_farm_delete = 删除这个农场
configuration_advanced = 高级配置
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

running_title = 运行中
running_node_title = {$chain_name} 共识节点
running_node_title_tooltip = 在文件管理器中打开
running_node_free_disk_space_tooltip = 空闲磁盘大小: {$size}
running_node_status_connecting = 连接网络中，最新区块 #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s (预计 ~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} 小时)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s (预计 ~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} 分钟)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocks/s (预计 ~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} 秒)
running_node_status_syncing =
    {$a_sync_kind ->
        [dsn] DSN 网络同步
        [regular] 常规同步
        *[unknown] 未知同步类型 {$a_sync_kind}
    } #{$b_best_block_number}/{$c_target_block}{$d_sync_speed}
running_node_status_synced = 同步完成, 最新区块 #{$best_block_number}
running_farmer_title = 农民
running_farmer_button_expand_details = 各农场的详细信息
running_farmer_button_pause_plotting = 暂停绘图/重新绘图，当前的编码扇区不会被中断
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
running_farmer_farm_reward_signatures_tooltip = {$a_successful_signatures}/{$b_total_signatures} 奖励签名成功，打开农场查看更多信息
running_farmer_farm_auditing_performance_tooltip = 审计性能: 平均时长 {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}秒, 时间限制 {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}秒
running_farmer_farm_proving_performance_tooltip = 证明性能: 平均时长 {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}秒, 时间限制 {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}秒
running_farmer_farm_non_fatal_error_tooltip = 非致命错误发生并已经恢复，在日志中查看更多信息: {$error}
running_farmer_farm_crashed = 农场崩溃: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} 分钟/扇区, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} 扇区/小时)
running_farmer_farm_plotting_initial =
    {$a_pausing_state ->
        [pausing] 初始绘制暂停中
        [paused] 初始绘制暂停
        *[no] 初始绘制中
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$c_plotting_speed}, {$d_farming ->
        [yes] 耕种中
        *[no] 未耕种
    }
running_farmer_farm_replotting =
    {$a_pausing_state ->
        [pausing] 初始绘制暂停中
        [paused] 初始绘制暂停
        *[default] 重新绘制
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$c_plotting_speed}, {$d_farming ->
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

stopped_title = 暂停
stopped_message = 暂停 🛑
stopped_message_with_error = 由于错误暂停: {$error}
stopped_button_show_logs = 打开日志
stopped_button_help_from_community = 获得社区支持

error_title = 错误
error_message = 错误: {$error}
error_message_failed_to_send_config_to_backend = 发送到后端过程出错: {$error}
error_message_failed_to_send_pause_plotting_to_backend = 发送暂停任务到后端出错: {$error}
error_button_show_logs = 打开日志
error_button_help_from_community = 获得社区支持

new_version_available = 版本 {$version} 可用 🎉
new_version_available_button_open = 打开版本发布页面

main_menu_show_logs = 在文件管理器中打开日志
main_menu_change_configuration = 修改配置
main_menu_share_feedback = 分享反馈
main_menu_about = 关于

status_bar_message_configuration_is_invalid = 配置不可用: {$error}
status_bar_message_restart_is_needed_for_configuration = 重启以使配置修改生效
status_bar_message_failed_to_save_configuration = 保存配置修改失败: {$error}
status_bar_button_restart = 重启
status_bar_button_ok = 正常

about_system_information =
    配置目录: {$a_config_directory}
    数据目录 (包括日志): {$b_data_directory}
