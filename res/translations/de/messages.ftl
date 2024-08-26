welcome_title = Willkommen
welcome_message =
    Space Acres ist eine leicht zu bedienende grafische Anwendung zum Farmen im Autonomys-Netzwerk.

    Bevor Du fortfährst, benötigst Du drei Dinge:
    ✔ Wallet-Adresse, an die Du Belohnungen erhältst (verwende Subwallet, die polkadot{"{"}.js{"}"}-Erweiterung oder eine andere Wallet, die mit der Substrate-Chain kompatibel ist)
    ✔ 100 GB Speicherplatz auf einer hochwertigen SSD, um Node-Daten zu speichern
    ✔ beliebige SSDs (oder mehrere) mit so viel Speicherplatz, wie Du Dir leisten kannst, um zu farmen – dies generiert Belohnungen
button_continue = Weiter

upgrade_title = Upgrade
upgrade_message =
    Vielen Dank, dass Du Dich wieder für Space Acres entschieden hast!

    Die Chain, die Du vor dem Upgrade betrieben hast, ist mit dieser Version von Space Acres nicht mehr kompatibel, wahrscheinlich weil Du an einer älteren Version von Autonomys teilgenommen hast.

    Aber keine Sorge, Du kannst mit einem einzigen Klick auf die aktuell unterstützte Chain upgraden!
upgrade_button_upgrade = Upgrade auf {$chain_name}

loading_title = Wird geladen
loading_configuration_title = Lade Konfiguration
loading_configuration_step_loading = Lade Konfiguration...
loading_configuration_step_reading = Lese Konfiguration
loading_configuration_step_configuration_exists = Lese konfiguration...
loading_configuration_step_configuration_not_found = Lese konfiguration...
loading_configuration_step_configuration_checking = Überprüfe Konfiguration...
loading_configuration_step_configuration_valid = Konfiguration ist gültig
loading_configuration_step_decoding_chain_spec = Dekodiere Chain-Spezifikation...
loading_configuration_step_decoded_chain_spec = Chain-Spezifikation erfolgreich dekodiert
loading_networking_stack_title = Initialisiere Netzwerk-Stack
loading_networking_stack_step_checking_node_path = Überprüfe Node-Pfad...
loading_networking_stack_step_creating_node_path = Erstelle Node-Pfad...
loading_networking_stack_step_node_path_ready = Node-Pfad ist bereit
loading_networking_stack_step_preparing = Bereite Netzwerk-Stack vor...
loading_networking_stack_step_reading_keypair = Lese Netzwerk-Schlüsselpaar...
loading_networking_stack_step_generating_keypair = Generiere Netzwerk-Schlüsselpaar...
loading_networking_stack_step_writing_keypair_to_disk = Schreibe Netzwerk-Schlüsselpaar auf die Festplatte...
loading_networking_stack_step_instantiating = Instanziiere Netzwerk-Stack...
loading_networking_stack_step_created_successfully = Netzwerk-Stack erfolgreich erstellt
loading_consensus_node_title = Initialisiere Netzwerk-Stack
loading_consensus_node_step_creating = Erstelle Konsensus-Node...
loading_consensus_node_step_created_successfully = Konsensus-Node erfolgreich erstellt
loading_farmer_title = Instanziiere Farmer
loading_farmer_step_initializing = Initialisiere Farmen {$index}/{$farms_total}...
loading_farmer_step_created_successfully = Farmer erfolgreich erstellt
loading_wiping_farmer_data_title = Lösche Farmer-Daten
loading_wiping_farmer_data_step_wiping_farm = Lösche Farm {$index}/{$farms_total} in {$path}...
loading_wiping_farmer_data_step_success = Alle Farmen erfolgreich gelöscht
loading_wiping_node_data_title = Lösche Node-Daten
loading_wiping_node_data_step_wiping_node = Lösche Node in {$path}...
loading_wiping_node_data_step_success = Knotendaten erfolgreich gelöscht

configuration_title = Konfiguration
reconfiguration_title = Neukonfiguration
configuration_node_path = Node-Pfad
configuration_node_path_placeholder = Beispiel: {$path}
configuration_node_path_tooltip = Absoluter Pfad, an dem die Node-Daten gespeichert werden. Plane mindestens 100 GiB Speicherplatz dafür ein, eine hochwertige SSD wird empfohlen.
configuration_node_path_button_select = Auswählen
configuration_node_path_error_doesnt_exist_or_write_permissions = Ordner existiert nicht oder Benutzer hat keine Schreibberechtigung
configuration_reward_address = Belohnungsadresse
configuration_reward_address_placeholder = Beispiel: {$address}
configuration_reward_address_tooltip = Verwende Subwallet oder die polkadot{"{"}.js{"}"}-Erweiterung oder eine andere Substrate-Wallet, um die Belohnungsadresse zu erstellen (jede Substrate-Chain-Adresse im SS58-Format funktioniert)
configuration_reward_address_button_create_wallet = Wallet erstellen
configuration_reward_address_error_evm_address = Dies sollte eine Substrate-Adresse (SS58) sein (jede Chain funktioniert) und keine EVM-Adresse
configuration_farm = Pfad zur Farm {$index} und ihre Größe
configuration_farm_path_placeholder = Beispiel: {$path}
configuration_farm_path_tooltip = Absoluter Pfad, an dem die Farm-Dateien gespeichert werden, jede SSD funktioniert, hohe Lebensdauer ist nicht erforderlich
configuration_farm_path_button_select = Auswählen
configuration_farm_path_error_doesnt_exist_or_write_permissions = Ordner existiert nicht oder Benutzer hat keine Schreibberechtigung
configuration_farm_size_placeholder = Beispiel: 4T, 2,5TB, 500GiB, etc.
configuration_farm_size_tooltip = Größe der Farm in beliebigen Einheiten, jede Angabe über 2 GB ist möglich
configuration_farm_delete = Diese Farm löschen
configuration_advanced = Erweiterte Konfiguration
configuration_advanced_network = Netzwerkkonfiguration
configuration_advanced_network_default_port_number_tooltip = Standardportnummer ist {$port}
configuration_advanced_network_substrate_port = Substrate (Blockchain) P2P-Port (TCP):
configuration_advanced_network_subspace_port = Subspace (DSN) P2P-Port (TCP):
configuration_advanced_network_faster_networking = Schnelleres Netzwerk:
configuration_advanced_network_faster_networking_tooltip = Standardmäßig ist das Netzwerk für Consumer-Router optimiert, aber wenn Du einen leistungsstärkeren Router hast, kann ein schnelleres Netzwerk die Synchronisationsgeschwindigkeit und andere Prozesse beschleunigen
configuration_button_add_farm = Farm hinzufügen
configuration_button_help = Hilfe
configuration_button_cancel = Abbrechen
configuration_button_back = Zurück
configuration_button_save = Speichern
configuration_button_start = Starten
configuration_dialog_button_select = Auswählen
configuration_dialog_button_cancel = Abbrechen

running_title = Läuft
running_node_title = {$chain_name} Konsensus-Node
running_node_title_tooltip = Klicke, um im Dateimanager zu öffnen
running_node_free_disk_space_tooltip = Freier Speicherplatz: {$size} verbleibend
running_node_status_connecting = Verbindung zum Netzwerk wird hergestellt, bester Block #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} Blöcke/s
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} Blöcke/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} Stunden verbleibend)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} Blöcke/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} Minuten verbleibend)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} Blöcke/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} Sekunden verbleibend)
running_node_status_syncing =
    {$a_sync_kind ->
        [dsn] Synchronisierung von DSN
        [regular] Reguläre Synchronisierung
        *[unknown] Unbekannte Synchronisierungsart {$a_sync_kind}
    } #{$b_best_block_number}/{$c_target_block}{$d_sync_speed}
running_node_status_synced = Synchronisiert, bester Block #{$best_block_number}
running_farmer_title = Farmer
running_farmer_button_expand_details = Details zu jeder Farm erweitern
running_farmer_button_pause_plotting = Plotting/Replotting pausieren, aktuell laufende werden nicht unterbrochen
running_farmer_account_balance_tooltip = Gesamtkontostand und seit Start der Anwendung gefarmte Coins, klicke, um Details in Astral zu sehen
running_farmer_piece_cache_sync = Piece-Cache-Synchronisierung {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Nächste Belohnung voraussichtlich: {$eta_string ->
        [any_time_now] unmittelbar bevorstehend
        [less_than_an_hour] weniger als eine Stunde
        [today] heute
        [this_week] diese Woche
        [more_than_a_week] mehr als eine Woche
        *[unknown] unbekannt
    }
running_farmer_farm_tooltip = Klicke, um im Dateimanager zu öffnen
running_farmer_farm_reward_signatures_tooltip = {$a_successful_signatures}/{$b_total_signatures} erfolgreiche Belohnungssignaturen, erweitere die Farmdetails, um weitere Informationen zu sehen
running_farmer_farm_auditing_performance_tooltip = Audit-Leistung: durchschnittliche Zeit {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, Zeitlimit {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_proving_performance_tooltip = Proving-Leistung: durchschnittliche Zeit {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, Zeitlimit {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_non_fatal_error_tooltip = Ein nicht-fataler Farmfehler ist aufgetreten und wurde behoben, siehe Logs für weitere Details: {$error}
running_farmer_farm_crashed = Farm abgestürzt: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} m/Sektor, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} Sektoren/h)
running_farmer_farm_plotting_initial =
    {$a_pausing_state ->
        [pausing] Pausiere initiales Plotting
        [paused] Initiales Plotting pausiert
        *[no] Initiales Plotting
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$c_plotting_speed}, {$d_farming ->
        [yes] Farming läuft
        *[no] kein Farming
    }
running_farmer_farm_replotting =
    {$a_pausing_state ->
        [pausing] Pausiere Replotting
        [paused] Replotting pausiert
        *[default] Replotting
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$c_plotting_speed}, {$d_farming ->
        [yes] Farming läuft
        *[no] kein Farming
    }
running_farmer_farm_farming = Farming läuft
running_farmer_farm_waiting_for_node_to_sync = Warten auf Node-Synchronisierung
running_farmer_farm_sector = Sektor {$sector_index}
running_farmer_farm_sector_up_to_date = Sektor {$sector_index}: aktuell
running_farmer_farm_sector_waiting_to_be_plotted = Sektor {$sector_index}: wartet auf Plotting
running_farmer_farm_sector_about_to_expire = Sektor {$sector_index}: läuft bald ab, wartet auf Replotting
running_farmer_farm_sector_expired = Sektor {$sector_index}: abgelaufen, wartet auf Replotting
running_farmer_farm_sector_downloading = Sektor {$sector_index}: wird heruntergeladen
running_farmer_farm_sector_encoding = Sektor {$sector_index}: wird kodiert
running_farmer_farm_sector_writing = Sektor {$sector_index}: wird geschrieben

stopped_title = Gestoppt
stopped_message = Gestoppt 🛑
stopped_message_with_error = Mit Fehler gestoppt: {$error}
stopped_button_show_logs = Logs anzeigen
stopped_button_help_from_community = Hilfe von der Community

error_title = Fehler
error_message = Fehler: {$error}
error_message_failed_to_send_config_to_backend = Konfiguration konnte nicht an das Backend gesendet werden: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Pausieren des Plottings konnte nicht an das Backend gesendet werden: {$error}
error_button_show_logs = Logs anzeigen
error_button_help_from_community = Hilfe von der Community

new_version_available = Version {$version} verfügbar 🎉
new_version_available_button_open = Release-Seite öffnen

main_menu_show_logs = Logs im Dateimanager anzeigen
main_menu_change_configuration = Konfiguration ändern
main_menu_share_feedback = Feedback teilen
main_menu_about = Über

status_bar_message_configuration_is_invalid = Konfiguration ist ungültig: {$error}
status_bar_message_restart_is_needed_for_configuration = Anwendung muss neu gestartet werden, damit die Konfigurationsänderungen wirksam werden
status_bar_message_failed_to_save_configuration = Konfigurationsänderungen konnten nicht gespeichert werden: {$error}
status_bar_button_restart = Neustart
status_bar_button_ok = Ok

about_system_information =
    Konfigurationsverzeichnis: {$a_config_directory}
    Datenverzeichnis (inklusive Logs): {$b_data_directory}
