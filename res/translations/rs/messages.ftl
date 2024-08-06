welcome_title = Dobrodošli
welcome_message =
    Space Acres je specijalizovana GUI aplikacija za farmovanje na Autonomys mreži.

    Pre nego što nastavite, potrebna su vam 3 stvari:
    ✔ Adresa novčanika gde ćete primati nagrade (koristite Subwallet, polkadot{"{"}.js{"}"} ekstenziju ili bilo koji drugi novčanik kompatibilan sa Substrate lancom)
    ✔ 100G prostora na kvalitetnom SSD-u za skladištenje podataka o čvoru
    ✔ bilo koji SSD (ili više njih) sa što više prostora koji možete priuštiti za farmovanje, to će generisati nagrade"
welcome_button_continue = Nastavi

upgrade_title = Nadogradnja
upgrade_message =
    Hvala što ste ponovo izabrali Space Acres!

    Lanac na kojem ste radili pre nadogradnje više nije kompatibilan sa ovim izdanjem Space Acresa, verovatno zato što ste učestvovali u prethodnoj verziji Autonomysa.

    Ali, ne brinite, možete se nadograditi na trenutno podržanu mrežu jednim klikom na dugme!"
upgrade_button_upgrade = Nadogradi na {$chain_name}

loading_title = Učitavanje
loading_configuration_title = Učitavanje konfiguracije
loading_configuration_step_loading = Učitavanje konfiguracije...
loading_configuration_step_reading = Čitanje konfiguracije...
loading_configuration_step_configuration_exists = Čitanje konfiguracije...
loading_configuration_step_configuration_not_found = Čitanje konfiguracije...
loading_configuration_step_configuration_checking = Provera konfiguracije...
loading_configuration_step_configuration_valid = Konfiguracija je važeća
loading_configuration_step_decoding_chain_spec = Dekodiranje specifikacije lanca...
loading_configuration_step_decoded_chain_spec = Uspešno dekodirana specifikacija lanca
loading_networking_stack_title = Inicijalizacija mrežnog steka
loading_networking_stack_step_checking_node_path = Provera putanje čvora...
loading_networking_stack_step_creating_node_path = Kreiranje putanje čvora...
loading_networking_stack_step_node_path_ready = Putanja čvora je spremna
loading_networking_stack_step_preparing = Priprema mrežnog steka...
loading_networking_stack_step_reading_keypair = Čitanje mrežnog para ključeva...
loading_networking_stack_step_generating_keypair = Generisanje mrežnog para ključeva...
loading_networking_stack_step_writing_keypair_to_disk = Pisanje mrežnog para ključeva na disk...
loading_networking_stack_step_instantiating = Stvaranje mrežnog steka...
loading_networking_stack_step_created_successfully = Mrežni stek je uspešno kreiran
loading_consensus_node_title = Inicijalizacija mrežnog steka
loading_consensus_node_step_creating = Kreiranje konsenzus čvora...
loading_consensus_node_step_created_successfully = Konsenzus čvor uspešno kreiran
loading_farmer_title = Inicijalizacija farmera
loading_farmer_step_initializing = Inicijalizacija farmi {$a_index}/{$b_farms_total}...
loading_farmer_step_created_successfully = Farmer uspešno kreiran
loading_wiping_farmer_data_title = Brisanje podataka farmera
loading_wiping_farmer_data_step_wiping_farm = Brisanje farme {$a_index}/{$b_farms_total} na {$c_path}...
loading_wiping_farmer_data_step_success = Sve farme su uspešno obrisane
loading_wiping_node_data_title = Brisanje podataka čvora
loading_wiping_node_data_step_wiping_node = Brisanje čvora na {$path}...
loading_wiping_node_data_step_success = Podaci o čvoru uspešno obrisani

configuration_title = Konfiguracija
reconfiguration_title = Rekonfiguracija
configuration_node_path = Putanja čvora
configuration_node_path_placeholder = Primer: {$path}
configuration_node_path_tooltip = Apsolutna putanja gde će se čuvati datoteke čvora, pripremite najmanje 100 GiB prostora za to, preporučuje se kvalitetan SSD
configuration_node_path_button_select = Izaberi
configuration_node_path_error_doesnt_exist_or_write_permissions = Folder ne postoji ili korisnik nema dozvolu za pisanje
configuration_reward_address = Adresa za nagrade
configuration_reward_address_placeholder = Primer: {$address}
configuration_reward_address_tooltip = Koristite Subwallet ili polkadot.js ekstenziju ili bilo koji drugi Substrate novčanik za njegovo kreiranje (adresa za bilo koji Substrate lanac u SS58 formatu funkcioniše)
configuration_reward_address_button_create_wallet = Kreiraj novčanik
configuration_reward_address_error_evm_address = Ovo bi trebala biti Substrate (SS58) adresa (bilo koji lanac će raditi), a ne EVM adresa
configuration_farm = Putanja do farme {$index} i njena veličina
configuration_farm_path_placeholder = Primer: {$path}
configuration_farm_path_tooltip = Apsolutna putanja gde će se čuvati datoteke farme, bilo koji SSD funkcioniše, visoka izdržljivost nije neophodna
configuration_farm_path_button_select = Izaberi
configuration_farm_path_error_doesnt_exist_or_write_permissions = Folder ne postoji ili korisnik nema dozvolu za pisanje
configuration_farm_size_placeholder = Primer: 4T, 2.5TB, 500GiB itd.
configuration_farm_size_tooltip = Veličina farme u jedinicama koje preferirate, bilo koja količina prostora iznad 2 GB funkcioniše
configuration_farm_delete = Obriši ovu farmu
configuration_advanced = Napredna konfiguracija
configuration_advanced_network = Konfiguracija mreže
configuration_advanced_network_default_port_number_tooltip = Podrazumevani broj porta je {$port}
configuration_advanced_network_substrate_port = Substrate (blockchain) P2P port (TCP):
configuration_advanced_network_subspace_port = Subspace (DSN) P2P port (TCP):
configuration_advanced_network_faster_networking = Brže umrežavanje:
configuration_advanced_network_faster_networking_tooltip = Podrazumevano, umrežavanje je optimizovano za kućne rutere, ali ako imate jaču opremu, brže umrežavanje može poboljšati brzinu sinhronizacije i druge procese
configuration_button_add_farm = Dodaj farmu
configuration_button_help = Pomoć
configuration_button_cancel = Otkaži
configuration_button_back = Nazad
configuration_button_save = Sačuvaj
configuration_button_start = Pokreni
configuration_dialog_button_select = Izaberi
configuration_dialog_button_cancel = Otkaži

running_title = U radu
running_node_title = {$chain_name} konsenzus čvor
running_node_title_tooltip = Kliknite da otvorite u upravitelju datotekama
running_node_free_disk_space_tooltip = Slobodan prostor na disku: preostalo {$size}
running_node_status_connecting = Povezivanje sa mrežom, najbolji blok #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blokova/s
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blokova/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} sati preostalo)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blokova/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} minuta preostalo)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blokova/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} sekundi preostalo)
running_node_status_syncing =
    {$a_sync_kind ->
        [dsn] Sinhronizacija sa DSN
        [regular] Redovna sinhronizacija
        *[unknown] Nepoznat tip sinhronizacije {$a_sync_kind}
    } #{$b_best_block_number}/{$c_target_block}{$d_sync_speed}
running_node_status_synced = Sinhronizovano, najbolji blok #{$best_block_number}
running_farmer_title = Farmer
running_farmer_button_expand_details = Proširi detalje o svakoj farmi
running_farmer_button_pause_plotting = Pauziraj plotovanje/preplotovanje, imajte na umu da trenutno kodiranje sektora neće biti prekinuto
running_farmer_account_balance_tooltip = Ukupni saldo i kovanice zarđene od početka rada aplikacije, kliknite da vidite detalje u Astral
running_farmer_piece_cache_sync = Sinhronizacija delova keša {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Sledeća procena nagrade: {$eta_string ->
        [any_time_now] bilo kada
        [less_than_an_hour] manje od sat vremena
        [today] danas
        [this_week] ove nedelje
        [more_than_a_week] više od nedelje
        *[unknown] nepoznato
    }
running_farmer_farm_tooltip = Kliknite da otvorite u upravitelju datotekama
running_farmer_farm_reward_signatures_tooltip = {$a_successful_signatures}/{$b_total_signatures} uspešnih potpisa nagrada, proširi detalje farme da vidiš više informacija
running_farmer_farm_auditing_performance_tooltip = Provera performansi: prosečno vreme {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, vremensko ograničenje {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_proving_performance_tooltip = Dokazivanje performansi: prosečno vreme {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, vremensko ograničenje {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_non_fatal_error_tooltip = Dogodila se ne-fatalna greška u farmovanju i uspešno je ispravljena, pogledajte dnevnik za više detalja: {$error}
running_farmer_farm_crashed = Farma se srušila: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} m/sektoru, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} sektora/h)
running_farmer_farm_plotting_initial =
    {$a_pausing_state ->
        [pausing] Pauziranje početnog plotovanja
        [paused] Početno plotovanje pauzirano
        *[no] Početno plotovanje
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$c_plotting_speed}, {$d_farming ->
        [yes] farmovanje
        *[no] ne farmuje
    }
running_farmer_farm_replotting =
    {$a_pausing_state ->
        [pausing] Pauziranje početnog plotovanja
        [paused] Početno plotovanje pauzirano
        *[default] Preplotovanje
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$c_plotting_speed}, {$d_farming ->
        [yes] farmovanje
        *[no] ne farmuje
    }
running_farmer_farm_farming = Farmovanje
running_farmer_farm_waiting_for_node_to_sync = Čeka se sinhronizacija čvora
running_farmer_farm_sector = Sektor {$sector_index}
running_farmer_farm_sector_up_to_date = Sektor {$sector_index}: ažuriran
running_farmer_farm_sector_waiting_to_be_plotted = Sektor {$sector_index}: čeka plotovanje
running_farmer_farm_sector_about_to_expire = Sektor {$sector_index}: uskoro ističe, čeka preplotovanje
running_farmer_farm_sector_expired = Sektor {$sector_index}: istekao, čeka preplotovanje
running_farmer_farm_sector_downloading = Sektor {$sector_index}: preuzimanje
running_farmer_farm_sector_encoding = Sektor {$sector_index}: kodiranje
running_farmer_farm_sector_writing = Sektor {$sector_index}: pisanje

stopped_title = Zaustavljeno
stopped_message = Zaustavljeno 🛑
stopped_message_with_error = Zaustavljeno sa greškom: {$error}
stopped_button_show_logs = Prikaz dnevnika
stopped_button_help_from_community = Pomoć zajednice

error_title = Greška
error_message = Greška: {$error}
error_message_failed_to_send_config_to_backend = Nije uspelo slanje konfiguracije na backend: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Nije uspelo slanje pauze plotovanja na backend: {$error}
error_button_show_logs = Prikaz dnevnika
error_button_help_from_community = Pomoć zajednice

new_version_available = Dostupna je nova verzija {$version} 🎉
new_version_available_button_open = Otvori stranicu sa izdanjima

main_menu_show_logs = Prikaz dnevnika u upravitelju datotekama
main_menu_change_configuration = Promeni konfiguraciju
main_menu_share_feedback = Deli povratne informacije
main_menu_about = O aplikaciji

status_bar_message_configuration_is_invalid = Konfiguracija je nevažeća: {$error}
status_bar_message_restart_is_needed_for_configuration = Potreban je restart aplikacije za primenu promena u konfiguraciji
status_bar_message_failed_to_save_configuration = Nije uspelo čuvanje promena konfiguracije: {$error}
status_bar_button_restart = Ponovo pokreni
status_bar_button_ok = U redu

about_system_information =
    Konfiguracioni direktorijum: {$a_config_directory}
    Direktorijum podataka (uključujući dnevnike): {$b_data_directory}
