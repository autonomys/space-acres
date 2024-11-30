welcome_title = Hoşgeldin
welcome_message =
    Space Acres is an opinionated GUI application for farming on Autonomys Network.

    Before continuing you need 3 things:
    ✔ Ödülleri alacağınız bir cüzdan adresi (use Subwallet, polkadot{"{"}.js{"}"} uzantısı veya Substrate ağı ile uyumlu herhangi bir cüzdan kullanabilirsiniz)
    ✔  Node verilerini depolamak için kaliteli bir SSD'de 100GB alan
    ✔ Farm amacıyla kullanabileceğiniz herhangi bir SSD (veya birden fazla), ne kadar alan ayırabilirseniz o kadar ödül kazanırsınız
welcome_button_continue = Devam Et

upgrade_title = Yükseltme
upgrade_message =
    Space Acres'i tekrar tercih ettiğiniz için teşekkürler!

    Daha önce çalıştırdığınız ağ, muhtemelen Autonomys'in önceki sürümüne katıldığınız için artık Space Acres'in bu sürümüyle uyumlu değil.

    Ancak endişelenmeyin, tek bir tıklama ile desteklenen mevcut ağa geçiş yapabilirsiniz!
upgrade_button_upgrade = Upgrade to {$chain_name}

loading_title = Yükleniyor
loading_configuration_title = Konfigürasyon Yükleniyor
loading_configuration_step_loading = Konfigürasyon Yükleniyor...
loading_configuration_step_reading = Konfigürasyon okunuyor...
loading_configuration_step_configuration_exists = Konfigürasyon mevcut
loading_configuration_step_configuration_not_found = Konfigürasyon bulunamadı
loading_configuration_step_configuration_checking = Konfigürasyon Kontrol Ediliyor...
loading_configuration_step_configuration_valid = Konfigürasyon Geçerli
loading_configuration_step_decoding_chain_spec = Ağ özellikleri çözülüyor...
loading_configuration_step_decoded_chain_spec = Ağ özellikleri başarıyla çözüldü
loading_networking_stack_title = Initializing networking stack
loading_networking_stack_step_checking_node_path = Node dosya yolu kontrol ediliyor...
loading_networking_stack_step_creating_node_path = Node dosya yolu oluşturuluyor...
loading_networking_stack_step_node_path_ready = Node dosya yolu hazır
loading_networking_stack_step_preparing = Ağ yapısı hazırlanıyor...
loading_networking_stack_step_reading_keypair = Reading network keypair...
loading_networking_stack_step_generating_keypair = Generating network keypair...
loading_networking_stack_step_writing_keypair_to_disk = Writing network keypair to disk...
loading_networking_stack_step_instantiating = Instantiating networking stack...
loading_networking_stack_step_created_successfully = Networking stack created successfully
loading_consensus_node_title = Konsensüs Node'u Başlatılıyor
loading_consensus_node_step_creating = Konsensüs Node'u oluşturuluyor...
loading_consensus_node_step_created_successfully = Konsensüs Node'u başarıyla oluşturuldu
loading_farmer_title = Çiftçi başlatılıyor
loading_farmer_step_initializing = Çiftlikler Başlatılıyor {$index}/{$farms_total}...
loading_farmer_step_created_successfully = Çiftlikler başarıyla oluşturuldu
loading_wiping_farmer_data_title = Çiftçi Verileri Temizleniyor
loading_wiping_farmer_data_step_wiping_farm = Çiftlikler {$index}/{$farms_total} konumunda temizleniyor {$path}...
loading_wiping_farmer_data_step_success = Tüm çiftlikler başarıyla temizlendi
loading_wiping_node_data_title = Node verileri temizleniyor
loading_wiping_node_data_step_wiping_node = Node verilerii bu konudma temizleniyor {$path}...
loading_wiping_node_data_step_success = Node verileri başarıyla temizlendi

configuration_title = Konfigürasyon
reconfiguration_title = Yeniden konfigürrasyon
configuration_node_path = Node dosya yolu
configuration_node_path_placeholder = Örnek: {$path}
configuration_node_path_tooltip = Node dosyalarının saklanacağı mutlak dosya yolu. En az 100 GiB alan ayırmaya hazırlıklı olun, kaliteli bir SSD önerilir
configuration_node_path_button_select = Seç
configuration_node_path_error_doesnt_exist_or_write_permissions = Klasör mevcut değil ya da kullanıcı yazma iznine sahip değil
configuration_reward_address = Ödül adresi
configuration_reward_address_placeholder = Örnek: {$address}
configuration_reward_address_tooltip = Subwallet, polkadot{"{"}.js{"}"} uzantısı veya herhangi bir Substrate cüzdanını kullanarak bu adresi oluşturun (SS58 formatındaki herhangi bir Substrate ağ adresi kullanılabilir)
configuration_reward_address_button_create_wallet = Cüzdan oluştur
configuration_reward_address_error_evm_address = Bu bir Substrate (SS58) adresi olmalı (herhangi bir ağ uygun), EVM adresi olmamalı
configuration_farm = Maden {$index} Yolu ve Boyutu
configuration_farm_path_placeholder = Örnek: {$path}
configuration_farm_path_tooltip = Çiftlik dosyalarının saklanacağı mutlak dosya yolu. Herhangi bir SSD uygundur, yüksek dayanıklılık gerekli değildir
configuration_farm_path_button_select = Seç
configuration_farm_path_error_doesnt_exist_or_write_permissions = Klasör mevcut değil ya da kullanıcı yazma iznine sahip değil
configuration_farm_size_kind_fixed = Sabit boyut
configuration_farm_size_kind_free_percentage = Boş alanın %'si
configuration_farm_fixed_size_placeholder = Örnek: 4T, 2.5TB, 500GiB, vb.
configuration_farm_fixed_size_tooltip = Çiftlik boyutunu istediğiniz birimle girin. 2 GB üzerinde herhangi bir alan uygundur
configuration_farm_free_percentage_size_placeholder = Örnek: 100%, 1.1%, vb.
configuration_farm_free_percentage_size_tooltip = Bu çiftliğin kaplayacağı boş disk alanının yüzdesi 0%'dan büyük bir değer olmalıdır, ancak hataları önlemek için disk üzerinde en az 2 GB boş alan kalmalıdır
configuration_farm_delete = Bu çiftliği sil
configuration_advanced = Gelişmiş Konfigürasyon
configuration_advanced_farmer = Çiftçi Konfigürasyonu
configuration_advanced_farmer_reduce_plotting_cpu_load = Reduce plotting CPU load
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = Initial plotting uses all CPU cores by default, while with this option it will start using half of the cores like replotting, improving system responsiveness for other tasks
configuration_advanced_network = Ağ konfigürasyonu
configuration_advanced_network_default_port_number_tooltip = Varsayılan port numarası {$port}
configuration_advanced_network_substrate_port = Substrate (blok zinciri) P2P portu (TCP):
configuration_advanced_network_subspace_port = Subspace (DSN) P2P portu (TCP):
configuration_advanced_network_faster_networking = Daha Hızlı Ağ:
configuration_advanced_network_faster_networking_tooltip = Varsayılan olarak ağ, tüketici yönlendiricilerine göre optimize edilmiştir. Ancak daha güçlü bir kurulumunuz varsa, daha hızlı ağ seçeneği senkronizasyon hızını ve diğer süreçleri iyileştirebilir
configuration_button_add_farm = Çiftlik Ekle
configuration_button_help = Yardım
configuration_button_cancel = İptal
configuration_button_back = Geri
configuration_button_save = Kaydet
configuration_button_start = Başlat
configuration_dialog_button_select = Seç
configuration_dialog_button_cancel = İptal

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
running_farmer_title = Çiftçi
running_farmer_button_expand_details = Expand details about each farm
running_farmer_button_pause_plotting = Pause plotting/replotting, note that currently encoding sectors will not be interrupted
running_farmer_account_balance_tooltip = Total account balance and coins farmed since application started, click to see details in Astral
running_farmer_piece_cache_sync = Piece cache sync {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Next reward estimate: {$eta_string ->
        [any_time_now] any time now
        [less_than_an_hour] bir saatten az
        [today] bugün
        [this_week] bu hafta
        [more_than_a_week] bir haftadan fazla
        *[unknown] bilinmiyor
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
running_farmer_farm_waiting_for_node_to_sync = Node senkronize olmak için bekliyor
running_farmer_farm_sector = Sector {$sector_index}
running_farmer_farm_sector_up_to_date = Sector {$sector_index}: up to date
running_farmer_farm_sector_waiting_to_be_plotted = Sector {$sector_index}: waiting to be plotted
running_farmer_farm_sector_about_to_expire = Sector {$sector_index}: about to expire, waiting to be replotted
running_farmer_farm_sector_expired = Sector {$sector_index}: expired, waiting to be replotted
running_farmer_farm_sector_downloading = Sector {$sector_index}: downloading
running_farmer_farm_sector_encoding = Sector {$sector_index}: encoding
running_farmer_farm_sector_writing = Sector {$sector_index}: writing

shutting_down_title = Kapatılıyor
shutting_down_description = This may take a couple of seconds to a few minutes depending on what application is doing

stopped_title = Durduruldu
stopped_message = Durduruldu 🛑
stopped_message_with_error = Stopped with error: {$error}
stopped_button_show_logs = Logları göster
stopped_button_help_from_community = Topluluktan yardım iste

error_title = Error
error_message = Error: {$error}
error_message_failed_to_send_config_to_backend = Failed to send config to backend: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Failed to send pause plotting to backend: {$error}
error_button_show_logs = Show logs
error_button_help_from_community = Topluluktan yardım iste

new_version_available = Version {$version} available 🎉
new_version_available_button_open = Open releases page

main_menu_show_logs = Show logs in file manager
main_menu_change_configuration = Konfigürasyonu değiştir
main_menu_share_feedback = Geribildirim bırak
main_menu_about = Hakkında
main_menu_exit = Çıkış

status_bar_message_configuration_is_invalid = Configuration is invalid: {$error}
status_bar_message_restart_is_needed_for_configuration = Application restart is needed for configuration changes to take effect
status_bar_message_failed_to_save_configuration = Failed to save configuration changes: {$error}
status_bar_message_restarted_after_crash = Space Acres automatically restarted after crash, check application and system logs for details
status_bar_button_restart = Yeniden Başlat
status_bar_button_ok = Tamam

about_system_information =
    Config directory: {$config_directory}
    Data directory (including logs): {$data_directory}

tray_icon_open = Aç
tray_icon_quit = Çıkış

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
