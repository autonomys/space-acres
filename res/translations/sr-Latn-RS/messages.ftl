welcome_title = Dobrodošli
welcome_message =
    Space Acres je specijalizovana GUI aplikacija za farmovanje na Autonomys mreži.

    Pre nego što nastavite, potrebna su vam 3 stvari:
    ✔ Adresa novčanika gde ćete primati nagrade (koristite Subwallet, polkadot{"{"}.js{"}"} ekstenziju ili bilo koji drugi novčanik kompatibilan sa Substrate lancom)
    ✔ 100G prostora na kvalitetnom SSD-u za skladištenje podataka o čvoru
    ✔ bilo koji SSD (ili više njih) sa što više prostora koji možete priuštiti za farmovanje, to će generisati nagrade
welcome_button_continue = Nastavi

upgrade_title = Nadogradnja
upgrade_message =
    Hvala što ste ponovo izabrali Space Acres!

    Lanac na kojem ste radili pre nadogradnje više nije kompatibilan sa ovim izdanjem Space Acresa, verovatno zato što ste učestvovali u prethodnoj verziji Autonomysa.

    Ali, ne brinite, možete se nadograditi na trenutno podržanu mrežu jednim klikom na dugme!
upgrade_button_upgrade = Nadogradi na {$chain_name}

loading_title = Učitavanje
loading_configuration_title = Učitavanje konfiguracije
loading_configuration_step_loading = Učitavanje konfiguracije...
loading_configuration_step_reading = Čitanje konfiguracije...
# TODO: Translate
loading_configuration_step_configuration_exists = Provera konfiguracije...
# TODO: Translate
loading_configuration_step_configuration_not_found = Provera konfiguracije...
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
loading_consensus_node_title = Inicijalizacija konsenzus čvora
loading_consensus_node_step_creating = Kreiranje konsenzus čvora...
loading_consensus_node_step_created_successfully = Konsenzus čvor uspešno kreiran
loading_farmer_title = Inicijalizacija farmera
loading_farmer_step_initializing = Inicijalizacija farmi {$index}/{$farms_total}...
loading_farmer_step_created_successfully = Farmer uspešno kreiran
loading_wiping_farmer_data_title = Brisanje podataka farmera
loading_wiping_farmer_data_step_wiping_farm = Brisanje farme {$index}/{$farms_total} na {$path}...
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
configuration_node_migrate_button = Migrate...
# configuration_node_migrate_button = Migriraj...
configuration_node_migrate_tooltip = Migrate or reset node database
# configuration_node_migrate_tooltip = Migriraj ili resetuj bazu podataka čvora
configuration_node_size = Node size: {$size}
# configuration_node_size = Veličina čvora: {$size}
configuration_node_volume_free_space = Free space: {$size}
# configuration_node_volume_free_space = Slobodan prostor: {$size}
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
configuration_farm_size_kind_fixed = Fiksna veličina
configuration_farm_size_kind_free_percentage = % slobodnog prostora
configuration_farm_fixed_size_placeholder = Primer: 4T, 2.5TB, 500GiB itd.
configuration_farm_fixed_size_tooltip = Veličina farme u jedinicama koje preferirate, bilo koja količina prostora iznad 2 GB funkcioniše
configuration_farm_free_percentage_size_placeholder = Primer: 100%, 1.1%, itd.
configuration_farm_free_percentage_size_tooltip = Procenat slobodnog prostora koji ova farma zauzima, sve preko 0% funkcioniše, ostavite minimum 2GB prostora da izbegnete greške
configuration_farm_delete = Obriši ovu farmu
configuration_advanced = Napredna konfiguracija
configuration_advanced_farmer = Konfiguracija farmera
configuration_advanced_farmer_reduce_plotting_cpu_load = Smanjeno opterećenje procesora
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = Inicijalno plotovanje koristi sva jegra na procesoru, dok će sa ovom opcijom koristiti jednu polovinu dostupnih jezgra, ovo poboljšava odaziv i performanse ostalih zadataka
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

node_migration_button_cancel = Cancel
# node_migration_button_cancel = Otkaži
node_migration_button_reset = Reset Node
# node_migration_button_reset = Resetuj čvor
node_migration_button_start = Start Migration
# node_migration_button_start = Pokreni migraciju
node_migration_destination_free_space = Free space: {$size}
# node_migration_destination_free_space = Slobodan prostor: {$size}
node_migration_destination_label = New node location:
# node_migration_destination_label = Nova lokacija čvora:
node_migration_destination_placeholder = Select destination folder
# node_migration_destination_placeholder = Izaberi odredišni folder
node_migration_dialog_title = Migrate Node Database
# node_migration_dialog_title = Migriraj bazu podataka čvora
node_migration_insufficient_space_warning = Warning: Not enough free space at destination
# node_migration_insufficient_space_warning = Upozorenje: Nema dovoljno slobodnog prostora na odredištu
node_migration_mode_fresh_sync = Fresh sync to new location
# node_migration_mode_fresh_sync = Nova sinhronizacija na novoj lokaciji
node_migration_mode_fresh_sync_explanation = Syncs a fresh database from the network at the new location. Often faster than migrating, especially if your node is out of sync, and requires less destination space.
# node_migration_mode_fresh_sync_explanation = Sinhronizuje novu bazu podataka sa mreže na novoj lokaciji. Često brže od migracije, posebno ako vaš čvor nije sinhronizovan, i zahteva manje prostora na odredištu.
node_migration_mode_migrate = Migrate database
# node_migration_mode_migrate = Migriraj bazu podataka
node_migration_mode_migrate_explanation = Moves the existing database to the new location. Requires enough destination space for the current database.
# node_migration_mode_migrate_explanation = Premešta postojeću bazu podataka na novu lokaciju. Zahteva dovoljno prostora na odredištu za trenutnu bazu podataka.
node_migration_mode_reset = Reset and resync in place
# node_migration_mode_reset = Resetuj i ponovo sinhronizuj na mestu
node_migration_mode_reset_explanation = Resets your node by wiping the database and syncing fresh from the network. Use this if your node database is corrupted or significantly out of sync.
# node_migration_mode_reset_explanation = Resetuje vaš čvor brisanjem baze podataka i sinhronizacijom iznova sa mreže. Koristite ovo ako je baza podataka vašeg čvora oštećena ili značajno desinhronizovana.
node_migration_non_node_data_warning = Note: Non-node data detected in this directory and will not be migrated
# node_migration_non_node_data_warning = Napomena: Otkriveni su podaci koji nisu vezani za čvor u ovom direktorijumu i neće biti migrirani
node_migration_source_label = Current location:
# node_migration_source_label = Trenutna lokacija:
node_migration_status_completed = Migration completed successfully!
# node_migration_status_completed = Migracija je uspešno završena!
node_migration_status_copying = Migrating database: {$percentage}%
# node_migration_status_copying = Migracija baze podataka: {$percentage}%
node_migration_status_deleting_source = Removing previous database...
# node_migration_status_deleting_source = Uklanjanje prethodne baze podataka...
node_migration_status_failed = Migration failed: {$error}
# node_migration_status_failed = Migracija nije uspela: {$error}
node_migration_status_restarting = Restarting Space Acres...
# node_migration_status_restarting = Ponovno pokretanje Space Acres...
node_migration_status_shutting_down = Shutting down node...
# node_migration_status_shutting_down = Gašenje čvora...
node_migration_status_updating_config = Updating configuration...
# node_migration_status_updating_config = Ažuriranje konfiguracije...
node_migration_status_verifying = Verifying database...
# node_migration_status_verifying = Verifikacija baze podataka...
node_migration_title = Migrating Node Database
# node_migration_title = Migracija baze podataka čvora

running_title = U radu
running_node_title = {$chain_name} konsenzus čvor
running_node_title_tooltip = Kliknite da otvorite u upravitelju datotekama
# TODO: Translate
running_node_free_disk_space_tooltip = Slobodan prostor na disku: preostalo {$size}
running_node_connections_tooltip = {$connected_peers}/{$expected_peers} peers connected, click for details about required P2P ports
running_node_status_connecting = Povezivanje sa mrežom, najbolji blok #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blokova/s
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blokova/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} sati preostalo)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blokova/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} minuta preostalo)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blokova/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} sekundi preostalo)
running_node_status_syncing =
    {$sync_kind ->
        [dsn] Sinhronizacija sa DSN
        [regular] Redovna sinhronizacija
        *[unknown] Nepoznat tip sinhronizacije {$sync_kind}
    } #{$best_block_number}/{$target_block}{$sync_speed}
running_node_status_synced = Sinhronizovano, najbolji blok #{$best_block_number}
running_farmer_title = Farmer
running_farmer_button_expand_details = Proširi detalje o svakoj farmi
running_farmer_button_pause_plotting = Pauziraj plotovanje/preplotovanje, imajte na umu da trenutno kodiranje sektora neće biti prekinuto
running_farmer_button_resume_plotting = Nastavi plotovanje
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
running_farmer_farm_reward_signatures_tooltip = {$successful_signatures}/{$total_signatures} uspešnih potpisa nagrada, proširi detalje farme da vidiš više informacija
running_farmer_farm_auditing_performance_tooltip = Provera performansi: prosečno vreme {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, vremensko ograničenje {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_proving_performance_tooltip = Dokazivanje performansi: prosečno vreme {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, vremensko ograničenje {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_non_fatal_error_tooltip = Dogodila se ne-fatalna greška u farmovanju i uspešno je ispravljena, pogledajte dnevnik za više detalja: {$error}
running_farmer_farm_crashed = Farma se srušila: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} m/sektoru, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} sektora/h)
running_farmer_farm_plotting_initial =
    {$pausing_state ->
        [pausing] Pauziranje početnog plotovanja
        [paused] Početno plotovanje pauzirano
        *[no] Početno plotovanje
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] farmovanje
        *[no] ne farmuje
    }
running_farmer_farm_replotting =
    {$pausing_state ->
        [pausing] Pauziranje početnog plotovanja
        [paused] Početno plotovanje pauzirano
        *[default] Preplotovanje
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
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

shutting_down_title = Gašenje
shutting_down_description = Ovo može da potraje od nekoliko sekundi do nekoliko minuta u zavisnosti od toga šta je aplikacija radila u tom trenutku

stopped_title = Zaustavljeno
stopped_message = Zaustavljeno 🛑
stopped_message_with_error = Zaustavljeno sa greškom: {$error}
stopped_button_show_logs = Prikaz dnevnika
stopped_button_help_from_community = Pomoć zajednice

error_title = Greška
error_message = Greška: {$error}
error_message_failed_to_send_config_to_backend = Nije uspelo slanje konfiguracije na backend: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Nije uspelo slanje pauze plotovanja na backend: {$error}
error_button_help_from_community = Pomoć zajednice
# error_button_help_from_community = Pomoć zajednice
error_button_reset_node = Reset node
# error_button_reset_node = Resetuj čvor
error_button_reset_node_tooltip = Wipe node data and sync fresh from the network
# error_button_reset_node_tooltip = Obriši podatke čvora i ponovo sinhronizuj sa mreže
error_button_show_logs = Prikaz dnevnika
# error_button_show_logs = Prikaz dnevnika

new_version_available = Dostupna je nova verzija {$version} 🎉
new_version_available_button_open = Otvori stranicu sa izdanjima

main_menu_show_logs = Prikaz dnevnika u upravitelju datotekama
main_menu_change_configuration = Promeni konfiguraciju
main_menu_share_feedback = Deli povratne informacije
main_menu_about = O aplikaciji
main_menu_exit = Izlaz

status_bar_message_configuration_is_invalid = Konfiguracija je nevažeća: {$error}
status_bar_message_restart_is_needed_for_configuration = Potreban je restart aplikacije za primenu promena u konfiguraciji
status_bar_message_failed_to_save_configuration = Nije uspelo čuvanje promena konfiguracije: {$error}
status_bar_message_restarted_after_crash = Space Acres se automatski restartovao nakon kraha, proveri dnevnik aplikacije za više informacija
status_bar_button_migrate = Migrate
# status_bar_button_migrate = Migriraj
status_bar_button_ok = U redu
status_bar_button_restart = Ponovo pokreni

about_system_information =
    Konfiguracioni direktorijum: {$config_directory}
    Direktorijum podataka (uključujući dnevnike): {$data_directory}

tray_icon_open = Otvori
# TODO: Check translation
tray_icon_quit = Izlaz

notification_app_minimized_to_tray = Space Acres je minimiziran u sistemsku traku
    .body = Možete ga ponovo otvoriti ili potpuno izaći koristeći meni ikone u sistemskoj traci
notification_stopped_with_error = Space Acres je zaustavljen zbog greške
    .body = Došlo je do greške koja zahteva intervenciju korisnika za rešavanje
notification_farm_error = Jedna od farmi u Space Acresu nije uspela
    .body = Došlo je do greške koja zahteva intervenciju korisnika za rešavanje
notification_node_low_disk_space = Low Node Disk Space
    .body = Node volume has only {$free_space} remaining
# notification_node_low_disk_space = Malo prostora na disku čvora
#     .body = Na volumenu čvora je preostalo samo {$free_space}
notification_missed_reward = Potpisivanje nagrade nije uspelo 😞
    .body = To je nesreća, ali biće još prilika uskoro
notification_signed_reward_successfully = Uspešno potpisana nova nagrada 🥳
    .body = Hvala vam što osiguravate mrežu 🙌

warning_low_disk_space = Low disk space on node volume
# warning_low_disk_space = Malo prostora na disku čvora
warning_low_disk_space_detail = Only {$free_space} remaining. Consider migrating to a larger drive.
# warning_low_disk_space_detail = Preostalo je samo {$free_space}. Razmislite o migraciji na veći disk.
