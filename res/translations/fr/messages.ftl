welcome_title = Bienvenue
welcome_message =
    Space Acres est une application autonome avec une interface graphique pour le farming sur le réseau Autonomys

    Avant de continuer, vous aurez besoin de trois choses :
    ✔ Une adresse de portefeuille pour recevoir vos récompenses (utilisez Subwallet, l'extension polkadot{"{"}.js{"}"} ou tout autre portefeuille compatible avec Substrate)
    ✔ 100 Go d'espace libre sur un SSD de bonne qualité pour stocker les données de la blockchain
    ✔ Un ou plusieurs SSD avec la plus grande capacité possible pour le farming. Plus la capacité est grande, plus la récompense est élevée
welcome_button_continue = Continuer

upgrade_title = Mise à jour
upgrade_message =
    Merci d'avoir à nouveau choisi Space Acres !

    Le réseau que vous utilisiez avant la mise à jour n'est plus compatible avec cette version de Space Acres. Cela est probablement dû à votre participation à la version précédente d'Autonomys

    Pas de soucis ! Vous pouvez passer au réseau actuellement pris en charge en appuyant simplement sur un bouton !
upgrade_button_upgrade = Mettre à jour vers {$chain_name}

loading_title = Chargement
loading_configuration_title = Chargement de la configuration
loading_configuration_step_loading = Chargement de la configuration...
loading_configuration_step_reading = Lecture de la configuration...
loading_configuration_step_configuration_exists = Configuration trouvée...
loading_configuration_step_configuration_not_found = Configuration absente...
loading_configuration_step_configuration_checking = Vérification de la configuration...
loading_configuration_step_configuration_valid = La configuration a été validée sans erreurs
loading_configuration_step_decoding_chain_spec = Traitement de la spécification de la blockchain...
loading_configuration_step_decoded_chain_spec = Spécification de la blockchain prête
loading_networking_stack_title = Initialisation de la pile réseau
loading_networking_stack_step_checking_node_path = Vérification du chemin des données de la blockchain...
loading_networking_stack_step_creating_node_path = Création du dossier des données de la blockchain...
loading_networking_stack_step_node_path_ready = Le dossier des données de la blockchain est prêt
loading_networking_stack_step_preparing = Préparation de la pile réseau...
loading_networking_stack_step_reading_keypair = Lecture de la paire de clés réseau...
loading_networking_stack_step_generating_keypair = Génération de la paire de clés réseau...
loading_networking_stack_step_writing_keypair_to_disk = Écriture de la paire de clés réseau sur le disque...
loading_networking_stack_step_instantiating = Initialisation de la pile réseau...
loading_networking_stack_step_created_successfully = Pile réseau créée avec succès
loading_consensus_node_title = Initialisation du nœud de consensus
loading_consensus_node_step_creating = Création du nœud de consensus...
loading_consensus_node_step_created_successfully = Nœud de consensus créé avec succès
loading_farmer_title = Création de la ferme
loading_farmer_step_initializing = Initialisation de la ferme {$index}/{$farms_total}...
loading_farmer_step_created_successfully = Ferme créée avec succès
loading_wiping_farmer_data_title = Suppression des données de la ferme
loading_wiping_farmer_data_step_wiping_farm = Suppression de la ferme {$index}/{$farms_total} dans {$path}...
loading_wiping_farmer_data_step_success = Toutes les fermes ont été supprimées avec succès
loading_wiping_node_data_title = Suppression des données de la blockchain
loading_wiping_node_data_step_wiping_node = Suppression des données de la blockchain dans {$path}...
loading_wiping_node_data_step_success = Données de la blockchain supprimées avec succès

configuration_title = Configuration
reconfiguration_title = Reconfiguration
configuration_node_path = Chemin vers le dossier de la blockchain
configuration_node_path_placeholder = Exemple : {$path}
configuration_node_path_tooltip = Chemin absolu vers le dossier où seront stockées les données de la blockchain. Allouez au moins 100Go d'espace libre, l'utilisation d'un SSD de bonne qualité est recommandée
configuration_node_path_button_select = Sélectionner
configuration_node_path_error_doesnt_exist_or_write_permissions = Le dossier n'existe pas ou l'utilisateur n'a pas les permissions d'écriture
configuration_node_migrate_button = Migrate...
# configuration_node_migrate_button = Migrer...
configuration_node_migrate_tooltip = Migrate or reset node database
# configuration_node_migrate_tooltip = Migrer ou réinitialiser la base de données du nœud
configuration_node_size = Node size: {$size}
# configuration_node_size = Taille du nœud : {$size}
configuration_node_volume_free_space = Free space: {$size}
# configuration_node_volume_free_space = Espace libre : {$size}
configuration_reward_address = Adresse pour recevoir les récompenses
configuration_reward_address_placeholder = Exemple : {$address}
configuration_reward_address_tooltip = Utilisez l'extension Subwallet ou polkadot{"{"}.js{"}"}, ou tout autre portefeuille Substrate pour créer d'abord le portefeuille (une adresse au format SS58 pour toute chaîne Substrate fonctionne)
configuration_reward_address_button_create_wallet = Créer un portefeuille
configuration_reward_address_error_evm_address = Cela doit être une adresse au format Substrate (SS58) (n'importe quel réseau), et non une adresse EVM
configuration_farm = Chemin vers la ferme {$index} et sa taille
configuration_farm_path_placeholder = Exemple : {$path}
configuration_farm_path_tooltip = Chemin absolu vers le dossier où seront stockées les données de la ferme. Un SSD de haute performance n'est pas nécessaire
configuration_farm_path_button_select = Sélectionner
configuration_farm_path_error_doesnt_exist_or_write_permissions = Le dossier n'existe pas ou l'utilisateur n'a pas les permissions d'écriture
configuration_farm_size_kind_fixed = Taille fixe
configuration_farm_size_kind_free_percentage = % de l'espace libre
configuration_farm_fixed_size_placeholder = Exemple : 4T, 2,5To, 500Gio, etc.
configuration_farm_fixed_size_tooltip = Taille de la ferme en fonction des unités que vous préférez. Toute taille supérieure à 2 Go convient
configuration_farm_free_percentage_size_placeholder = Exemple : 100%, 1,1%, etc.
configuration_farm_free_percentage_size_tooltip = Pourcentage de l'espace disque libre à occuper par cette ferme. Tout ce qui dépasse 0 % fonctionne, mais il est conseillé de laisser au moins 2 Go d'espace libre sur le disque pour éviter les erreurs
configuration_farm_delete = Supprimer cette ferme
configuration_advanced = Configuration avancée
configuration_advanced_farmer = Configuration de la ferme
configuration_advanced_farmer_reduce_plotting_cpu_load = Réduire la charge du processeur pendant le plotting
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = Le plotting initial utilise tous les cœurs du processeur par défaut. Cette option réduit la charge à la moitié des cœurs, ce qui permet d'utiliser l'ordinateur pour d'autres tâches
configuration_advanced_network = Configuration réseau
configuration_advanced_network_default_port_number_tooltip = Le numéro de port par défaut est {$port}
configuration_advanced_network_substrate_port = Port P2P Substrate (blockchain) (TCP) :
configuration_advanced_network_subspace_port = Port P2P Subspace (DSN) (TCP) :
configuration_advanced_network_faster_networking = Réseau rapide :
configuration_advanced_network_faster_networking_tooltip = Par défaut, les paramètres réseau sont optimisés pour les routeurs domestiques. Si vous disposez d'un équipement plus performant, cette option peut améliorer la vitesse de synchronisation et d'autres processus
configuration_button_add_farm = Ajouter une ferme
configuration_button_help = Aide
configuration_button_cancel = Annuler
configuration_button_back = Retour
configuration_button_save = Sauvegarder
configuration_button_start = Démarrer
configuration_dialog_button_select = Sélectionner
configuration_dialog_button_cancel = Annuler

node_migration_button_cancel = Cancel
# node_migration_button_cancel = Annuler
node_migration_button_reset = Reset Node
# node_migration_button_reset = Réinitialiser le nœud
node_migration_button_start = Start Migration
# node_migration_button_start = Démarrer la migration
node_migration_destination_free_space = Free space: {$size}
# node_migration_destination_free_space = Espace libre : {$size}
node_migration_destination_label = New node location:
# node_migration_destination_label = Nouvel emplacement du nœud :
node_migration_destination_placeholder = Select destination folder
# node_migration_destination_placeholder = Sélectionner le dossier de destination
node_migration_dialog_title = Migrate Node Database
# node_migration_dialog_title = Migrer la base de données du nœud
node_migration_insufficient_space_warning = Warning: Not enough free space at destination
# node_migration_insufficient_space_warning = Avertissement : Espace libre insuffisant à la destination
node_migration_mode_fresh_sync = Fresh sync to new location
# node_migration_mode_fresh_sync = Nouvelle synchronisation vers un nouvel emplacement
node_migration_mode_fresh_sync_explanation = Syncs a fresh database from the network at the new location. Often faster than migrating, especially if your node is out of sync, and requires less destination space.
# node_migration_mode_fresh_sync_explanation = Synchronise une nouvelle base de données depuis le réseau au nouvel emplacement. Souvent plus rapide que la migration, surtout si votre nœud est désynchronisé, et nécessite moins d'espace de destination.
node_migration_mode_migrate = Migrate database
# node_migration_mode_migrate = Migrer la base de données
node_migration_mode_migrate_explanation = Moves the existing database to the new location. Requires enough destination space for the current database.
# node_migration_mode_migrate_explanation = Déplace la base de données existante vers le nouvel emplacement. Nécessite suffisamment d'espace de destination pour la base de données actuelle.
node_migration_mode_reset = Reset and resync in place
# node_migration_mode_reset = Réinitialiser et resynchroniser sur place
node_migration_mode_reset_explanation = Resets your node by wiping the database and syncing fresh from the network. Use this if your node database is corrupted or significantly out of sync.
# node_migration_mode_reset_explanation = Réinitialise votre nœud en effaçant la base de données et en resynchronisant depuis le réseau. Utilisez ceci si la base de données de votre nœud est corrompue ou significativement désynchronisée.
node_migration_non_node_data_warning = Note: Non-node data detected in this directory and will not be migrated
# node_migration_non_node_data_warning = Remarque : Des données non liées au nœud ont été détectées dans ce répertoire et ne seront pas migrées
node_migration_source_label = Current location:
# node_migration_source_label = Emplacement actuel :
node_migration_status_completed = Migration completed successfully!
# node_migration_status_completed = Migration terminée avec succès !
node_migration_status_copying = Migrating database: {$percentage}%
# node_migration_status_copying = Migration de la base de données : {$percentage}%
node_migration_status_deleting_source = Removing previous database...
# node_migration_status_deleting_source = Suppression de l'ancienne base de données...
node_migration_status_failed = Migration failed: {$error}
# node_migration_status_failed = Échec de la migration : {$error}
node_migration_status_restarting = Restarting Space Acres...
# node_migration_status_restarting = Redémarrage de Space Acres...
node_migration_status_shutting_down = Shutting down node...
# node_migration_status_shutting_down = Arrêt du nœud...
node_migration_status_updating_config = Updating configuration...
# node_migration_status_updating_config = Mise à jour de la configuration...
node_migration_status_verifying = Verifying database...
# node_migration_status_verifying = Vérification de la base de données...
node_migration_title = Migrating Node Database
# node_migration_title = Migration de la base de données du nœud

running_title = En cours
running_node_title = {$chain_name} nœud de la blockchain
running_node_title_tooltip = Cliquez pour ouvrir dans le gestionnaire de fichiers
running_node_free_disk_space_tooltip = Espace disque libre restant : {$size}
running_node_connections_tooltip = {$connected_peers}/{$expected_peers} pairs connectées, cliquez pour plus de détails sur les ports P2P requis
running_node_status_connecting = Connexion au réseau, meilleur bloc #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocs/s
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocs/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} heures restantes)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocs/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} minutes restantes)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} blocs/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} secondes restantes)
running_node_status_syncing =
    {$sync_kind ->
        [dsn] Synchronisation depuis DSN
        [regular] Synchronisation régulière
        *[unknown] Type de synchronisation inconnu {$sync_kind}
    } #{$best_block_number}/{$target_block}{$sync_speed}
running_node_status_synced = Synchronisé, meilleur bloc #{$best_block_number}
running_farmer_title = Ferme
running_farmer_button_expand_details = Afficher les détails de chaque ferme
running_farmer_button_pause_plotting = Suspendre le plotting/replotting. Notez que le codage en cours des secteurs ne sera pas interrompu
running_farmer_button_resume_plotting = Reprendre le plotting
running_farmer_account_balance_tooltip = Solde total et pièces gagnées depuis le lancement de l'application. Cliquez pour voir les détails dans Astral
running_farmer_piece_cache_sync = Synchronisation du cache de morceaux à {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Prochaine récompense : {$eta_string ->
        [any_time_now] à tout moment
        [less_than_an_hour] moins d'une heure
        [today] aujourd'hui
        [this_week] cette semaine
        [more_than_a_week] plus d'une semaine
        *[unknown] inconnu
    }
running_farmer_farm_tooltip = Cliquez pour ouvrir dans le gestionnaire de fichiers
running_farmer_farm_reward_signatures_tooltip = {$successful_signatures}/{$total_signatures} signatures de récompense réussies. Consultez les détails de la ferme pour plus d'informations
running_farmer_farm_auditing_performance_tooltip = Performance de l'audit : temps moyen {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, limite de temps {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_proving_performance_tooltip = Performance de la preuve : temps moyen {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, limite de temps {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_non_fatal_error_tooltip = Une erreur est survenue lors du farming, mais elle a été corrigée. Consultez le journal pour plus de détails : {$error}
running_farmer_farm_crashed = Le farming a échoué : {$error}
running_farmer_farm_plotting_speed = ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} min/secteur, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} secteur/heure)
running_farmer_farm_plotting_initial =
    {$pausing_state ->
        [pausing] Suspension du plotting initial
        [paused] Plotting initial suspendu
        *[no] Plotting initial
    } à {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}% {$plotting_speed}, {$farming ->
        [yes] En farming
        *[no] Pas en farming
    }
running_farmer_farm_replotting =
    {$pausing_state ->
        [pausing] Mise en pause du plotting initial
        [paused] Plotting initial en pause
        *[default] Replotting
    } à {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}% {$plotting_speed}, {$farming ->
        [yes] En farming
        *[no] Pas en farming
    }
running_farmer_farm_farming = Farming
running_farmer_farm_waiting_for_node_to_sync = En attente de la synchronisation du nœud de la blockchain
running_farmer_farm_sector = Secteur {$sector_index}
running_farmer_farm_sector_up_to_date = Secteur {$sector_index} : à jour
running_farmer_farm_sector_waiting_to_be_plotted = Secteur {$sector_index} : en attente de plotting
running_farmer_farm_sector_about_to_expire = Secteur {$sector_index} : sur le point d'expirer, en attente de replotting
running_farmer_farm_sector_expired = Secteur {$sector_index} : expiré, en attente de replotting
running_farmer_farm_sector_downloading = Secteur {$sector_index} : téléchargement
running_farmer_farm_sector_encoding = Secteur {$sector_index} : encodage
running_farmer_farm_sector_writing = Secteur {$sector_index} : écriture

shutting_down_title = Fermeture en cours
shutting_down_description = Cela peut prendre de quelques secondes à quelques minutes, selon les processus en cours

stopped_title = Arrêté
stopped_message = Arrêté 🛑
stopped_message_with_error = Arrêté avec erreur : {$error}
stopped_button_show_logs = Voir le journal
stopped_button_help_from_community = Aide de la communauté

error_title = Erreur
error_message = Erreur : {$error}
error_message_failed_to_send_config_to_backend = Échec de l'envoi de la configuration au système interne : {$error}
error_message_failed_to_send_pause_plotting_to_backend = Échec de l'envoi de la mise en pause du plotting au système interne : {$error}
error_button_help_from_community = Aide de la communauté
# error_button_help_from_community = Aide de la communauté
error_button_reset_node = Reset node
# error_button_reset_node = Réinitialiser le nœud
error_button_reset_node_tooltip = Wipe node data and sync fresh from the network
# error_button_reset_node_tooltip = Effacer les données du nœud et synchroniser à nouveau depuis le réseau
error_button_show_logs = Voir le journal
# error_button_show_logs = Voir le journal

new_version_available = Nouvelle version {$version} disponible 🎉
new_version_available_button_open = Aller aux versions

main_menu_show_logs = Voir le journal dans le gestionnaire de fichiers
main_menu_change_configuration = Modifier la configuration
main_menu_share_feedback = Donner un avis
main_menu_about = À propos
main_menu_exit = Quitter

status_bar_message_configuration_is_invalid = La configuration est invalide : {$error}
status_bar_message_restart_is_needed_for_configuration = Redémarrez l'application pour que les modifications de configuration prennent effet
status_bar_message_failed_to_save_configuration = Échec de la sauvegarde de la configuration : {$error}
status_bar_message_restarted_after_crash = Space Acres s'est automatiquement redémarré après un crash. Consultez l'application et le journal système pour plus de détails
status_bar_button_migrate = Migrate
# status_bar_button_migrate = Migrer
status_bar_button_ok = OK
status_bar_button_restart = Redémarrer

about_system_information =
    Répertoire de configuration : {$config_directory}
    Répertoire des données (y compris le journal) : {$data_directory}

tray_icon_open = Ouvrir
tray_icon_quit = Quitter

notification_app_minimized_to_tray = Space Acres a été réduit dans la barre d'icônes
    .body = Vous pouvez le rouvrir ou quitter complètement en utilisant le menu de l'icône dans la barre de notification
notification_stopped_with_error = Space Acres s'est arrêté avec une erreur
    .body = Une erreur est survenue et nécessite une intervention de l'utilisateur pour la résoudre
notification_farm_error = L'une des fermes de Space Acres a rencontré une erreur
    .body = Une erreur est survenue et nécessite une intervention de l'utilisateur pour la résoudre
notification_node_low_disk_space = Low Node Disk Space
    .body = Node volume has only {$free_space} remaining
# notification_node_low_disk_space = Espace disque du nœud faible
#     .body = Le volume du nœud n'a plus que {$free_space} disponible
notification_missed_reward = Échec de la signature de la récompense 😞
    .body = C'est regrettable, mais il y aura bientôt une autre opportunité
notification_signed_reward_successfully = Nouvelle récompense signée avec succès 🥳
    .body = Merci pour votre contribution à la sécurité du réseau 🙌

warning_low_disk_space = Low disk space on node volume
# warning_low_disk_space = Espace disque faible sur le volume du nœud
warning_low_disk_space_detail = Only {$free_space} remaining. Consider migrating to a larger drive.
# warning_low_disk_space_detail = Il ne reste que {$free_space}. Envisagez de migrer vers un disque plus grand.
