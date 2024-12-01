welcome_title = Hoşgeldin
welcome_message =
    Space Acres, Autonomys Network üzerinde farming yapmak için tasarlanmış, belirli tercihleri olan bir grafik arayüz uygulamasıdır.

    Devam etmeden önce 3 şeye ihtiyacınız var:
    ✔ Ödülleri alacağınız bir cüzdan adresi (use Subwallet, polkadot{"{"}.js{"}"} uzantısı veya Substrate ağı ile uyumlu herhangi bir cüzdan kullanabilirsiniz)
    ✔  Node verilerini depolamak için kaliteli bir SSD'de 100GiB alan
    ✔ Farm amacıyla kullanabileceğiniz herhangi bir SSD (veya birden fazla), ne kadar alan ayırabilirseniz o kadar ödül kazanırsınız
welcome_button_continue = Devam Et

upgrade_title = Yükseltme
upgrade_message =
    Space Acres'i tekrar tercih ettiğiniz için teşekkürler!

    Daha önce çalıştırdığınız ağ, muhtemelen Autonomys'in önceki sürümüne katıldığınız için artık Space Acres'in bu sürümüyle uyumlu değil.

    Ancak endişelenmeyin, tek bir tıklama ile desteklenen mevcut ağa geçiş yapabilirsiniz!
upgrade_button_upgrade = Buna yükselt {$chain_name}

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
loading_networking_stack_title = Ağ yığını başlatılıyor
loading_networking_stack_step_checking_node_path = Node dosya yolu kontrol ediliyor...
loading_networking_stack_step_creating_node_path = Node dosya yolu oluşturuluyor...
loading_networking_stack_step_node_path_ready = Node dosya yolu hazır
loading_networking_stack_step_preparing = Ağ yapısı hazırlanıyor...
loading_networking_stack_step_reading_keypair = Ağ anahtar çifti okunuyor...
loading_networking_stack_step_generating_keypair = Ağ anahtar çifti oluşturuluyor...
loading_networking_stack_step_writing_keypair_to_disk = Ağ anahtar çifti diske yazılıyor...
loading_networking_stack_step_instantiating = Ağ yığını başlatılıyor...
loading_networking_stack_step_created_successfully = Ağ yığını başarılı bir şekilde oluşturuldu
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
configuration_advanced_farmer_reduce_plotting_cpu_load = Çizim CPU yükünü azalt
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = Başlangıç çizimi varsayılan olarak tüm CPU çekirdeklerini kullanır. Bu seçenek etkinleştirildiğinde, yeniden çizimde olduğu gibi sadece çekirdeklerin yarısını kullanır. Bu, diğer görevler için sistemin daha duyarlı olmasını sağlar
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

running_title = Çalışıyor
running_node_title = {$chain_name} konsensüs düğümü
running_node_title_tooltip = Dosya yöneticisinde açmak için tıklayın
running_node_free_disk_space_tooltip = Boş disk alanı: {$size} kaldı
running_node_status_connecting = Ağa bağlanılıyor, en iyi blok #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blok/sn
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blok/sn (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} saat kaldı)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blok/sn (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} dakika kaldı)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blok/sn (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} saniye kaldı)
running_node_status_syncing =
    {$sync_kind ->
        [dsn] DSN'den senkronize ediliyor
        [regular] Normal senkronizasyon
        *[unknown] Bilinmeyen senkronizasyon türü {$sync_kind}
    } #{$best_block_number}/{$target_block}{$sync_speed}
running_node_status_synced = Senkronize edildi, en iyi blok #{$best_block_number}
running_farmer_title = Çiftçi
running_farmer_button_expand_details = Her bir çiftlik hakkında detayları genişlet
running_farmer_button_pause_plotting = Alan oluşturmayı/yeni veri alanı hazırlamayı duraklat, unutmayın, şu anda oluşturulmakta olan alanlar tamamlanacak
running_farmer_account_balance_tooltip = Uygulama başlatıldığından beri toplam hesap bakiyesi ve üretilen coinler, detayları Astral'de görmek için tıklayın
running_farmer_piece_cache_sync = Parça önbelleği senkronizasyonu {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Bir sonraki ödül tahmini: {$eta_string ->
        [any_time_now] hemen şimdi
        [less_than_an_hour] bir saatten az
        [today] bugün
        [this_week] bu hafta
        [more_than_a_week] bir haftadan fazla
        *[unknown] bilinmiyor
    }
running_farmer_farm_tooltip = Dosya yöneticisinde açmak için tıklayın
running_farmer_farm_reward_signatures_tooltip = {$successful_signatures}/{$total_signatures} başarılı ödül imzaları, daha fazla bilgi için çiftlik detaylarını genişletin
running_farmer_farm_auditing_performance_tooltip = Denetim performansı: ortalama süre {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}sn, zaman limiti {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}sn
running_farmer_farm_proving_performance_tooltip = Kanıt performansı: ortalama süre {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}sn, zaman limiti {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}sn
running_farmer_farm_non_fatal_error_tooltip = Riskli olmayan bir çiftçilik hatası oluştu ve düzeltildi, daha fazla detay için loglara bakın: {$error}
running_farmer_farm_crashed = Çiftlik çöktü: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} dakika/sektör, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} sektörler/saat)
running_farmer_farm_plotting_initial =
    {$pausing_state ->
        [pausing] Başlangıç veri alanı oluşturma duraklatılıyor
        [paused] Başlangıç veri alanı oluşturma duraklatıldı
        *[no] Başlangıç veri alanı oluşturma
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] çiftçilik yapılıyor
        *[no] çiftçilik yapılmıyor
    }
running_farmer_farm_replotting =
    {$pausing_state ->
        [pausing] Yeniden veri alanı oluşturma duraklatılıyor
        [paused] Yeniden veri alanı oluşturma duraklatıldı
        *[default] Yeniden veri alanı oluşturma
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] çiftçilik yapılıyor
        *[no] çiftçilik yapılmıyor
    }
running_farmer_farm_farming = Çiftçilik yapılıyor
running_farmer_farm_waiting_for_node_to_sync = Node senkronize olmak için bekliyor
running_farmer_farm_sector = Sektör {$sector_index}
running_farmer_farm_sector_up_to_date = Sektör {$sector_index}: güncel
running_farmer_farm_sector_waiting_to_be_plotted = Sektör {$sector_index}: veri alanı oluşturulmak için bekleniyor
running_farmer_farm_sector_about_to_expire = Sektör {$sector_index}: süresi dolmak üzere, yeniden veri alanı oluşturulmak için bekleniyor
running_farmer_farm_sector_expired = Sektör {$sector_index}: süresi dolmuş, yeniden veri alanı oluşturulmak için bekleniyor
running_farmer_farm_sector_downloading = Sektör {$sector_index}: indiriliyor
running_farmer_farm_sector_encoding = Sektör {$sector_index}: kodlanıyor
running_farmer_farm_sector_writing = Sektör {$sector_index}: yazılıyor

shutting_down_title = Kapatılıyor
shutting_down_description = Uygulamanın yaptığı işleme bağlı olarak bu birkaç saniyeden birkaç dakikaya kadar sürebilir

stopped_title = Durduruldu
stopped_message = Durduruldu 🛑
stopped_message_with_error = Hata ile durduruldu: {$error}
stopped_button_show_logs = Logları göster
stopped_button_help_from_community = Topluluktan yardım iste

error_title = Hata
error_message = Hata: {$error}
error_message_failed_to_send_config_to_backend = Konfigürasyon verileri arka uca iletilemedi: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Çizimi duraklatmayı arka uca göndermek başarısız oldu: {$error}
error_button_show_logs = Logları göster
error_button_help_from_community = Topluluktan yardım iste

new_version_available = Sürüm {$version} mevcut 🎉
new_version_available_button_open = Sürümler sayfasını aç

main_menu_show_logs = Logları dosya yöneticisinde göster
main_menu_change_configuration = Konfigürasyonu değiştir
main_menu_share_feedback = Geribildirim bırak
main_menu_about = Hakkında
main_menu_exit = Çıkış

status_bar_message_configuration_is_invalid = Konfigürasyon geçersiz: {$error}
status_bar_message_restart_is_needed_for_configuration = Konfigürasyon değişikliklerinin etkili olması için uygulamanın yeniden başlatılması gerekiyor
status_bar_message_failed_to_save_configuration = Konfigürasyon değişiklikleri kaydedilemedi: {$error}
status_bar_message_restarted_after_crash = Space Acres çökme sonrası otomatik olarak yeniden başlatıldı, ayrıntılar için uygulama ve sistem loglarını kontrol edin
status_bar_button_restart = Yeniden Başlat
status_bar_button_ok = Tamam

about_system_information =
    Konfigürasyon dizini: {$config_directory}
    Veri dizini (loglar dahil): {$data_directory}

tray_icon_open = Aç
tray_icon_quit = Çıkış

notification_app_minimized_to_tray = Space Acres simge duruma küçültüldü
    .body = Tekrar açmak veya tamamen çıkış yapmak için görev çubuğu menüsünü kullanabilirsiniz
notification_stopped_with_error = Space Acres bir hata nedeniyle durdu
    .body = Bir hata meydana geldi ve çözüm için kullanıcı müdahalesi gerekiyor
notification_farm_error =  Space Acres içerisindeki çiftliklerden biri başarısız oldu
    .body = Bir hata meydana geldi ve çözüm için kullanıcı müdahalesi gerekiyor
notification_signed_reward_successfully = Yeni ödül başarıyla imzalandı 🥳
    .body = Ağı güvence altına aldığınız için teşekkürler 🙌
notification_missed_reward = Ödül imzalama başarısız oldu 😞
    .body = Bu üzücü bir durum, ancak yakında başka bir şansınız olacak
