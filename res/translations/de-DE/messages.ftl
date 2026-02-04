welcome_title = Willkommen
welcome_message =
    Space Acres ist eine konventionsgeleitete GUI-Anwendung für Farming im Autonomys-Netzwerk.

    Bevor du fortfährst, benötigst du 3 Dinge:
    ✔ Eine Wallet-Adresse, an die du deine Rewards erhältst (verwende Subwallet, die polkadot{"{"}.js{"}"} -Erweiterung oder jede andere Wallet, die mit der Substrate-Chain kompatibel ist)
    ✔ 100 GB Speicherplatz auf einer qualitativ hochwertigen SSD, um die Node-Daten zu speichern
    ✔ Beliebige SSDs (oder mehrere SSDs) mit so viel Speicherplatz, wie du dir für Farming-Zwecke leisten kannst – dies bestimmt, wie viel Rewards generiert werden
welcome_button_continue = Weiter

upgrade_title = Upgrade
upgrade_message =
    Danke, dass du dich wieder für Space Acres entschieden hast!

    Die Chain, die du vor dem Upgrade betrieben hast, ist mit dieser Version von Space Acres nicht mehr kompatibel, wahrscheinlich weil du an einer vorherigen Version von Autonomys teilgenommen hast.

    Aber keine Sorge, du kannst mit einem einzigen Klick auf die aktuell unterstützte Netzwerkversion upgraden!
upgrade_button_upgrade = Upgrade to {$chain_name}

loading_title = lade
loading_configuration_title = lade Konfiguration
loading_configuration_step_loading = lade Konfiguration...
loading_configuration_step_reading = lese Konfiguration...
loading_configuration_step_configuration_exists = überprüfe Konfiguration...
loading_configuration_step_configuration_not_found = überprüfe Konfiguration...
loading_configuration_step_configuration_checking = überprüfe Konfiguration...
loading_configuration_step_configuration_valid = konfiguration ist gültig
loading_configuration_step_decoding_chain_spec = entschlüssele die Chain-Spezifikation...
loading_configuration_step_decoded_chain_spec = Chain-Spezifikation erfolgreich entschlüsselt
loading_networking_stack_title = initialisiere Netzwerk-Stack
loading_networking_stack_step_checking_node_path = überprüfe Node-Pfad...
loading_networking_stack_step_creating_node_path = erstelle Node-Pfad...
loading_networking_stack_step_node_path_ready = Node-Pfad bereit
loading_networking_stack_step_preparing = bereite Netzwerk-Stack vor...
loading_networking_stack_step_reading_keypair = lese Netzwerk-Schlüsselpaar...
loading_networking_stack_step_generating_keypair = erstelle Netzwerk-Schlüsselpaar...
loading_networking_stack_step_writing_keypair_to_disk = schreibe Netzwerk-Schlüsselpaar auf die Festplatte...
loading_networking_stack_step_instantiating = instanziiere Netzwerk-Stack...
loading_networking_stack_step_created_successfully = Netzwerk-Stack erfolgreich erstellt
loading_consensus_node_title = initialisiere Konsens-Node
loading_consensus_node_step_creating = erstelle Konsens-Node...
loading_consensus_node_step_created_successfully = Konsens-Node erfolgreich erstellt
loading_farmer_title = instanziiere Farmer
loading_farmer_step_initializing = Initialisiere Farms {$index}/{$farms_total}...
loading_farmer_step_created_successfully = Farmer erfolgreich erstellt
loading_wiping_farmer_data_title = lösche Farmer-Daten
loading_wiping_farmer_data_step_wiping_farm = lösche Farm-Daten {$index}/{$farms_total} unter {$path}...
loading_wiping_farmer_data_step_success = alle Farms erfolgreich gelöscht
loading_wiping_node_data_title = lösche Node-Daten
loading_wiping_node_data_step_wiping_node = lösche Node unter {$path}...
loading_wiping_node_data_step_success = Node-Daten erfolgreich gelöscht

configuration_title = Konfiguration
reconfiguration_title = Rekonfiguration
configuration_node_path = Node-Pfad
configuration_node_path_placeholder = Beispiel: {$path}
configuration_node_path_tooltip = Absoluter Pfad, an dem die Node-Dateien gespeichert werden. Stelle sicher, dass du mindestens 100 GiB Speicherplatz dafür bereitstellst. Eine qualitativ hochwertige SSD wird empfohlen.
configuration_node_path_button_select = Auswählen
configuration_node_path_error_doesnt_exist_or_write_permissions = Ordner existiert nicht oder Benutzer hat keine Schreibberechtigung
configuration_node_migrate_button = Migrieren...
configuration_node_migrate_tooltip = Node-Datenbank migrieren oder zurücksetzen
configuration_node_size = Node-Größe: {$size}
configuration_node_volume_free_space = Freier Speicherplatz: {$size}
configuration_reward_address = Rewards-Adresse
configuration_reward_address_placeholder = Beispiel: {$address}
configuration_reward_address_tooltip = Verwende Subwallet, die polkadot{"{"}.js{"}"}-Erweiterung oder eine andere Substrate-Wallet, um diese zuerst zu erstellen (eine Adresse für jede Substrate-Chain im SS58-Format funktioniert).
configuration_reward_address_button_create_wallet = Wallet erstellen
configuration_reward_address_error_evm_address = Dies sollte eine Substrate (SS58) Adresse sein (jede Substrate-Chain ist in Ordnung), keine EVM-Adresse.
configuration_farm = Pfad zur Farm {$index} und deren Größe
configuration_farm_path_placeholder = Beispiel: {$path}
configuration_farm_path_tooltip = Absoluter Pfad, an dem die Farm-Dateien gespeichert werden. Jede SSD ist geeignet, hohe Qualittät oder Ausdauer ist nicht erforderlich.
configuration_farm_path_button_select = Auswählen
configuration_farm_path_error_doesnt_exist_or_write_permissions = Ordner existiert nicht oder Benutzer hat keine Schreibberechtigung
configuration_farm_size_kind_fixed = Feste Größe
configuration_farm_size_kind_free_percentage = % des freien Speicherplatzes
configuration_farm_fixed_size_placeholder = Beispiel: 4T, 2.5TB, 500GiB, etc.
configuration_farm_fixed_size_tooltip = Größe der Farm in beliebiger Einheit, jeder Speicherplatz über 2 GB ist geeignet.
configuration_farm_free_percentage_size_placeholder = Beispiel: 100%, 1.1%, etc.
configuration_farm_free_percentage_size_tooltip = Prozentsatz des freien Speicherplatzes, den diese Farm belegen soll. Jeder Wert über 0 % ist geeignet, aber es sollten mindestens 2 GB freier Speicherplatz auf der Festplatte verbleiben, um Fehler zu vermeiden
configuration_farm_delete = Diese Farm löschen
configuration_advanced = Erweiterte Konfiguration
configuration_advanced_farmer = Farmer-Konfiguration
configuration_advanced_farmer_reduce_plotting_cpu_load = CPU-Belastung beim Plotten reduzieren
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = Das initiale Plotten verwendet standardmäßig alle CPU-Kerne. Mit dieser Option wird es jedoch nur die Hälfte der Kerne nutzen, ähnlich wie beim Replotten, wodurch die Systemreaktionsfähigkeit für andere Aufgaben verbessert wird
configuration_advanced_network = Netzwerkkonfiguration
configuration_advanced_network_default_port_number_tooltip = Der Standardport ist {$port}
configuration_advanced_network_substrate_port = Substrate (Blockchain) P2P-Port (TCP):
configuration_advanced_network_subspace_port = Subspace (DSN) P2P-Port (TCP):
configuration_advanced_network_faster_networking = Schnelles Netzwerk:
configuration_advanced_network_faster_networking_tooltip = Standardmäßig ist das Netzwerk für Konsumenten-Router optimiert. Wenn du jedoch eine leistungsstärkere Konfiguration hast, kann "Schnelles Netzwerk" die Synchronisationsgeschwindigkeit und andere Prozesse verbessern
configuration_button_add_farm = Farm hinzufügen
configuration_button_help = Hilfe
configuration_button_cancel = Abbrechen
configuration_button_back = Zurück
configuration_button_save = Speichern
configuration_button_start = Start
configuration_dialog_button_select = Auswählen
configuration_dialog_button_cancel = Abbrechen

node_migration_button_cancel = Abbrechen
node_migration_button_reset = Node zurücksetzen
node_migration_button_start = Migration starten
node_migration_destination_free_space = Freier Speicherplatz: {$size}
node_migration_destination_label = Neuer Node-Speicherort:
node_migration_destination_placeholder = Zielordner auswählen
node_migration_dialog_title = Node-Datenbank migrieren
node_migration_insufficient_space_warning = Warnung: Nicht genügend freier Speicherplatz am Zielort
node_migration_mode_fresh_sync = Neue Synchronisierung am neuen Speicherort
node_migration_mode_fresh_sync_explanation = Synchronisiert eine neue Datenbank aus dem Netzwerk am neuen Speicherort. Oft schneller als Migrieren, besonders wenn Ihr Node nicht synchron ist, und benötigt weniger Speicherplatz am Zielort.
node_migration_mode_migrate = Datenbank migrieren
node_migration_mode_migrate_explanation = Verschiebt die bestehende Datenbank an den neuen Speicherort. Erfordert genügend Speicherplatz am Zielort für die aktuelle Datenbank.
node_migration_mode_reset = Zurücksetzen und vor Ort neu synchronisieren
node_migration_mode_reset_explanation = Setzt Ihren Node zurück, indem die Datenbank gelöscht und neu aus dem Netzwerk synchronisiert wird. Verwenden Sie dies, wenn Ihre Node-Datenbank beschädigt oder stark desynchronisiert ist.
node_migration_non_node_data_warning = Hinweis: Nicht-Node-Daten in diesem Verzeichnis erkannt, diese werden nicht migriert
node_migration_source_label = Aktueller Speicherort:
node_migration_status_completed = Migration erfolgreich abgeschlossen!
node_migration_status_copying = Datenbank wird migriert: {$percentage}%
node_migration_status_deleting_source = Vorherige Datenbank wird entfernt...
node_migration_status_failed = Migration fehlgeschlagen: {$error}
node_migration_status_restarting = Space Acres wird neu gestartet...
node_migration_status_shutting_down = Node wird heruntergefahren...
node_migration_status_updating_config = Konfiguration wird aktualisiert...
node_migration_status_verifying = Datenbank wird überprüft...
node_migration_title = Node-Datenbank wird migriert

running_title = Wird ausgeführt
running_node_title = {$chain_name} Konsens-Node
running_node_title_tooltip = Klicken, um im Dateimanager zu öffnen
running_node_free_disk_space_tooltip = Freier Speicherplatz: {$size} verbleibend
running_node_connections_tooltip = {$connected_peers}/{$expected_peers} Peers verbunden, klicke hier für Details zu den benötigten P2P-Ports
running_node_status_connecting = Verbindung zum Netzwerk wird hergestellt, bester Block #{$block_number}
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
running_farmer_button_expand_details = Details zu jeder Farm erweitern
running_farmer_button_pause_plotting = Plotten/Replotten pausieren, bitte beachten, dass aktuell laufende Encoding-Prozesse nicht unterbrochen werden
running_farmer_button_resume_plotting = Plotten fortsetzen
running_farmer_account_balance_tooltip = Gesamtsaldo des Kontos und gefarmte Coins seit Start der Anwendung, klicken, um Details in Astral anzuzeigen
running_farmer_piece_cache_sync = Piece-Cache-Synchronisation {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Nächste Reward-Schätzung: {$eta_string ->
        [any_time_now] jederzeit jetzt
        [less_than_an_hour] weniger als eine Stunde
        [today] heute
        [this_week] diese Woche
        [more_than_a_week] mehr als eine Woche
        *[unknown] unbekannt
    }
running_farmer_farm_tooltip = Klicken, um im Dateimanager zu öffnen
running_farmer_farm_reward_signatures_tooltip = {$successful_signatures}/{$total_signatures} Erfolgreiche Reward-Signaturen, erweitere die Farm-Details, um mehr Informationen zu sehen.
running_farmer_farm_auditing_performance_tooltip = Leistungsüberprüfung: Durchschnittliche Zeit {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, time limit {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_proving_performance_tooltip = Nachweis der Leistung: Durchschnittliche Zeit {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, time limit {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_non_fatal_error_tooltip = Ein nicht-kritischer Fehler beim Farming ist aufgetreten und wurde behoben, siehe Protokolle für weitere Details: {$error}
running_farmer_farm_crashed = Farm abgestürzt: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} m/sector, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} sectors/h)
running_farmer_farm_plotting_initial =
    {$pausing_state ->
        [pausing] pausiere initiales Plotten
        [paused] Initiales Plotten pausiert
        *[no] Initiales Plotten
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] farming
        *[no] kein farming
    }
running_farmer_farm_replotting =
    {$pausing_state ->
        [pausing] pausiere initiales Plotten
        [paused] Initiales Plotten pausiert
        *[default] replotten
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] farming
        *[no] kein farming
    }
running_farmer_farm_farming = farming
running_farmer_farm_waiting_for_node_to_sync = Waiting for node to sync
running_farmer_farm_sector = Sektor {$sector_index}
running_farmer_farm_sector_up_to_date = Sektor {$sector_index}: auf dem neuesten Stand
running_farmer_farm_sector_waiting_to_be_plotted = Sektor {$sector_index}: wartet auf das Plotten
running_farmer_farm_sector_about_to_expire = Sektor {$sector_index}: steht kurz vor dem Ablauf, wartet auf das Replotten
running_farmer_farm_sector_expired = Sektor {$sector_index}: abgelaufen, wartet auf das Replotten
running_farmer_farm_sector_downloading = Sektor {$sector_index}: wird heruntergeladen
running_farmer_farm_sector_encoding = Sektor {$sector_index}: wird codiert
running_farmer_farm_sector_writing = Sektor {$sector_index}: wird geschrieben

shutting_down_title = fährt herunter
shutting_down_description = Dies kann je nach dem, was die Anwendung gerade macht, einige Sekunden bis einige Minuten dauern.

stopped_title = angehalten
stopped_message = angehalten 🛑
stopped_message_with_error = angehalten mit Fehler: {$error}
stopped_button_show_logs = Protokolle anzeigen
stopped_button_help_from_community = Hilfe aus der Community

error_title = Fehler
error_message = Fehler: {$error}
error_message_failed_to_send_config_to_backend = Fehler beim Senden der Konfiguration an das Backend: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Fehler beim Senden der Pause-Anfrage für das Plotten an das Backend: {$error}
error_button_help_from_community = Hilfe aus der Community
error_button_reset_node = Node zurücksetzen
error_button_reset_node_tooltip = Node-Daten löschen und neu vom Netzwerk synchronisieren
error_button_show_logs = Protokolle anzeigen

new_version_available = Version {$version} verfügbar 🎉
new_version_available_button_open = Releases-Seite öffnen

main_menu_show_logs = Protokolle im Dateimanager anzeigen
main_menu_change_configuration = Konfiguration ändern
main_menu_share_feedback = Feedback geben
main_menu_about = Über
main_menu_exit = Beenden

status_bar_message_configuration_is_invalid = Konfiguration ist ungültig: {$error}
status_bar_message_restart_is_needed_for_configuration = Ein Neustart der Anwendung ist erforderlich, damit die Konfigurationsänderungen wirksam werden
status_bar_message_failed_to_save_configuration = Fehler beim Speichern der Konfigurationsänderungen: {$error}
status_bar_message_restarted_after_crash = Space Acres wurde nach einem Absturz automatisch neu gestartet. Überprüfe die Anwendungs- und Systemprotokolle für Details
status_bar_button_migrate = Migrieren
status_bar_button_ok = Ok
status_bar_button_restart = Neustart

about_system_information =
    Konfigurationsverzeichnis: {$config_directory}
    Datenverzeichnis (einschließlich Protokolle): {$data_directory}

tray_icon_open = öffnen
tray_icon_quit = Beenden

notification_app_minimized_to_tray = Space Acres wurde in die Taskleiste minimiert
    .body = Du kannst es wieder öffnen oder komplett beenden, indem du das Menü des Tray-Symbols verwendest
notification_stopped_with_error = Space Acres wurde mit einem Fehler angehalten
    .body = Ein Fehler ist aufgetreten, der eine Benutzerintervention zur Behebung erfordert
notification_farm_error = Eine der Farms in Space Acres ist fehlgeschlagen
    .body = Ein Fehler ist aufgetreten, der eine Benutzerintervention zur Behebung erfordert
notification_node_low_disk_space = Wenig Node-Speicherplatz
    .body = Node-Laufwerk hat nur noch {$free_space} übrig
notification_missed_reward = Signieren der Reward fehlgeschlagen 😞
    .body = Das ist bedauerlich, aber es wird bald eine weitere Gelegenheit geben
notification_signed_reward_successfully = Neue Reward erfolgreich signiert 🥳
    .body = Danke, dass du das Netzwerk sicherst 🙌

warning_low_disk_space = Wenig Speicherplatz auf dem Node-Laufwerk
warning_low_disk_space_detail = Nur noch {$free_space} übrig. Erwägen Sie die Migration auf ein größeres Laufwerk.
