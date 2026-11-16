welcome_title = Welcome
welcome_message =
    Space Acres es una aplicación gráfica (GUI) diseñada para unirte como granjero en la Red de Autonomys.

    Antes de continuar, necesitas 3 cosas:
    ✔ Una cartera donde recibirás las recompensas (utiliza Subwallet, la extensión de polkadot{"{"}.js{"}"} o cualquier otra cartera compatible con la blockchain Substrate)
    ✔ 100 GB de espacio en un SSD de buena calidad para almacenar los datos del nodo
    ✔ Cualquier SSD (o varios) con tanto espacio como puedas permitirte para fines de agricultura, esto es lo que generará las recompensas
welcome_button_continue = Continuar

upgrade_title = Actualización
upgrade_message =
    ¡Gracias por elegir Space Acres de nuevo!

    La blockchain que estabas utilizando antes de la actualización ya no es compatible con esta versión de Space Acres, probablemente porque estabas participando en la versión anterior de Autonomys.

    Pero no te preocupes, puedes actualizarte a la red actual en un solo clic!
upgrade_button_upgrade = Actualizar a {$chain_name}

loading_title = Cargando
loading_configuration_title = Cargando configuración
loading_configuration_step_loading = Cargando configuración...
loading_configuration_step_reading = Leyendo configuración...
loading_configuration_step_configuration_exists = Se ha encontrado la configuración...
loading_configuration_step_configuration_not_found = No se ha encontrado la configuración...
loading_configuration_step_configuration_checking = Verificando configuración...
loading_configuration_step_configuration_valid = La configuración es válida
loading_configuration_step_decoding_chain_spec = Decodificando especificación de la blockchain...
loading_configuration_step_decoded_chain_spec = Especificación de la blockchain decodificada con éxito
loading_networking_stack_title = Inicializando pila de red
loading_networking_stack_step_checking_node_path = Verificando ruta del nodo...
loading_networking_stack_step_creating_node_path = Creando ruta del nodo...
loading_networking_stack_step_node_path_ready = Ruta del nodo lista
loading_networking_stack_step_preparing = Preparando pila de red...
loading_networking_stack_step_reading_keypair = Leyendo claves de red...
loading_networking_stack_step_generating_keypair = Generando claves de red...
loading_networking_stack_step_writing_keypair_to_disk = Guardando claves de red en el disco...
loading_networking_stack_step_instantiating = Instanciando pila de red...
loading_networking_stack_step_created_successfully = Pila de red creada con éxito
loading_consensus_node_title = Inicializando nodo de consenso
loading_consensus_node_step_creating = Creando nodo de consenso...
loading_consensus_node_step_created_successfully = Nodo de consenso creado con éxito
loading_farmer_title = Instanciando granjero
loading_farmer_step_initializing = Inicializando granjas {$index}/{$farms_total}...
loading_farmer_step_created_successfully = Granjero creado con éxito
loading_wiping_farmer_data_title = Borrando datos del granjero
loading_wiping_farmer_data_step_wiping_farm = Borrando granja {$index}/{$farms_total} en {$path}...
loading_wiping_farmer_data_step_success = Todas las granjas borradas con éxito
loading_wiping_node_data_title = Borrando datos del nodo
loading_wiping_node_data_step_wiping_node = Borrando nodo en {$path}...
loading_wiping_node_data_step_success = Datos del nodo borrados con éxito

configuration_title = Configuración
reconfiguration_title = Reconfiguración
configuration_node_path = Ruta del nodo
configuration_node_path_placeholder = Ejemplo: {$path}
configuration_node_path_tooltip = Ruta absoluta donde se almacenarán los archivos del nodo, prepárate para dedicar al menos 100 GiB de espacio para ello, se recomienda un SSD de buena calidad
configuration_node_path_button_select = Seleccionar
configuration_node_path_error_doesnt_exist_or_write_permissions = La carpeta no existe o el usuario no tiene permisos de escritura
configuration_node_migrate_button = Migrar...
configuration_node_migrate_tooltip = Migrar o restablecer la base de datos del nodo
configuration_node_size = Tamaño del nodo: {$size}
configuration_node_volume_free_space = Espacio libre: {$size}
configuration_reward_address = Dirección de la cartera de recompensas
configuration_reward_address_placeholder = Ejemplo: {$address}
configuration_reward_address_tooltip = Usa Subwallet o la extensión de polkadot{"{"}.js{"}"} o cualquier otra cartera compatible con Substrate para crearla primero (una dirección para cualquier blockchain de Substrate en formato SS58 funciona)
configuration_reward_address_button_create_wallet = Crear cartera
configuration_reward_address_error_evm_address = Debe ser una dirección de Substrate (SS58) (cualquier blockchain servirá), no una dirección EVM
configuration_farm = Ruta a la granja {$index} y su tamaño
configuration_farm_path_placeholder = Ejemplo: {$path}
configuration_farm_path_tooltip = Ruta absoluta donde se almacenarán los archivos de la granja, cualquier SSD sirve, no es necesario que sea de alta resistencia
configuration_farm_path_button_select = Seleccionar
configuration_farm_path_error_doesnt_exist_or_write_permissions = La carpeta no existe o el usuario no tiene permisos de escritura
configuration_farm_size_kind_fixed = Tamaño fijo
configuration_farm_size_kind_free_percentage = % de espacio libre
configuration_farm_fixed_size_placeholder = Ejemplo: 4T, 2.5TB, 500GiB, etc.
configuration_farm_fixed_size_tooltip = Tamaño de la granja en las unidades que prefieras, cualquier cantidad de espacio superior a 2 GB funciona
configuration_farm_free_percentage_size_placeholder = Ejemplo: 100%, 1.1%, etc.
configuration_farm_free_percentage_size_tooltip = Porcentaje de espacio libre en disco que ocupará esta granja, cualquier valor superior al 0% funciona, pero al menos 2 GB de espacio libre deben permanecer en el disco para evitar errores
configuration_farm_delete = Eliminar esta granja
configuration_advanced = Configuración avanzada
configuration_advanced_farmer = Configuración del granjero
configuration_advanced_farmer_reduce_plotting_cpu_load = Reducir carga de CPU durante la creación de parcelas
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = La creación inicial de parcelas utiliza todos los núcleos de la CPU por defecto, mientras que con esta opción comenzará a usar la mitad de los núcleos como en el sustitución de parcelas, mejorando la capacidad de respuesta del sistema para otras tareas
configuration_advanced_network = Configuración de red
configuration_advanced_network_default_port_number_tooltip = El número de puerto predeterminado es {$port}
configuration_advanced_network_substrate_port = Puerto P2P de Substrate (blockchain) (TCP):
configuration_advanced_network_subspace_port = Puerto P2P de Subspace (DSN) (TCP):
configuration_advanced_network_faster_networking = Red más rápida:
configuration_advanced_network_faster_networking_tooltip = Por defecto, la red está optimizada para routers convencionales, pero si tienes una configuración más potente, una red más rápida puede mejorar la velocidad de sincronización y otros procesos
configuration_button_add_farm = Agregar granja
configuration_button_help = Ayuda
configuration_button_cancel = Cancelar
configuration_button_back = Atrás
configuration_button_save = Guardar
configuration_button_start = Iniciar
configuration_dialog_button_select = Seleccionar
configuration_dialog_button_cancel = Cancelar

node_migration_button_cancel = Cancelar
node_migration_button_reset = Reiniciar nodo
node_migration_button_start = Iniciar migración
node_migration_destination_free_space = Espacio libre: {$size}
node_migration_destination_label = Nueva ubicación del nodo:
node_migration_destination_placeholder = Seleccionar carpeta de destino
node_migration_dialog_title = Migrar base de datos del nodo
node_migration_insufficient_space_warning = Advertencia: No hay suficiente espacio libre en el destino
node_migration_mode_fresh_sync = Sincronización nueva en una ubicación nueva
node_migration_mode_fresh_sync_explanation = Sincroniza una base de datos nueva desde la red en la nueva ubicación. A menudo es más rápido que migrar, especialmente si su nodo está desincronizado, requiere menos espacio en el destino.
node_migration_mode_migrate = Migrar base de datos
node_migration_mode_migrate_explanation = Mueve la base de datos existente a la nueva ubicación. Requiere suficiente espacio en el destino para la base de datos actual.
node_migration_mode_reset = Reiniciar y resincronizar
node_migration_mode_reset_explanation = Reinicia su nodo borrando la base de datos y sincronizando desde la red. Use esto si la base de datos de su nodo está corrupta o significativamente desincronizada.
node_migration_non_node_data_warning = Nota: Se detectaron datos no relacionados con el nodo en este directorio y no serán migrados
node_migration_source_label = Ubicación actual:
node_migration_status_completed = ¡Migración completada con éxito!
node_migration_status_copying = Migrando base de datos: {$percentage}%
node_migration_status_deleting_source = Eliminando la base de datos anterior...
node_migration_status_failed = Migración fallida: {$error}
node_migration_status_restarting = Reiniciando Space Acres...
node_migration_status_shutting_down = Apagando nodo...
node_migration_status_updating_config = Actualizando configuración...
node_migration_status_verifying = Verificando base de datos...
node_migration_title = Migrando la base de datos del nodo

running_title = Ejecutando
running_node_title = {$chain_name} nodo de consenso
running_node_title_tooltip = Abrir sistema de archivos
running_node_free_disk_space_tooltip = Espacio libre en disco: {$size} restante
running_node_connections_tooltip = {$connected_peers}/{$expected_peers} pares conectados, clique para ver más detalles de los puertos P2P necesarios
running_node_status_connecting = Conectando a la red, mejor bloque #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} bloques/s
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} bloques/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} horas restantes)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} bloques/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} minutos restantes)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} bloques/s (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} segundos restantes)
running_node_status_syncing =
    {$sync_kind ->
        [dsn] Syncing from DSN
        [regular] Regular sync
        *[unknown] Desconocida sync kind {$sync_kind}
    } #{$best_block_number}/{$target_block}{$sync_speed}
running_node_status_synced = Sincronización completada, mejor bloque #{$best_block_number}
running_farmer_title = Granjero
running_farmer_button_expand_details = Amplía detalles del granjero
running_farmer_button_pause_plotting = Pausar creación y sustitución de parcelas, ten en cuenta que el procesamiento de los sectores no será interrumpido
running_farmer_button_resume_plotting = Reanudar la creación de la parcela
running_farmer_account_balance_tooltip = Balance total de la cuenta y monedas granjeadas desde que la aplicación se inició, clica para ver más detalles en Astral
running_farmer_piece_cache_sync = Caché sincronizada {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Próxima recompensa estimada para: {$eta_string ->
        [any_time_now] En cualquier momento
        [less_than_an_hour] En menos de una hora
        [today] Hoy
        [this_week] Esta semana
        [more_than_a_week] Más de una semana
        *[unknown] Desconocido
    }
running_farmer_farm_tooltip = Abrir sistema de archivos
running_farmer_farm_reward_signatures_tooltip = {$successful_signatures}/{$total_signatures} firmas de recompensas existosas, obtén más información en los detalles de la granja
running_farmer_farm_auditing_performance_tooltip = Auditando eficiencia: tiempo medio {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, tiempo límite {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_proving_performance_tooltip = Demostrando eficiencia: tiempo medio {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, tiempo límite {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_non_fatal_error_tooltip = Ha ocurrido un error pero se ha conseguido recuperar, mira la traza para más información: {$error}
running_farmer_farm_crashed = Ha ocurrido un error en la granja que no se ha conseguido recuperar: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} m/sector, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} sectores/h)
running_farmer_farm_plotting_initial =
    {$pausing_state ->
        [pausing] Pausando la creación de parcelas
        [paused] Pausada la creación de parcelas
        *[no] Creando parcelas
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] Granjeando
        *[no] No granjeando
    }
running_farmer_farm_replotting =
    {$pausing_state ->
        [pausing] Pausando la creación de parcelas
        [paused] Pausada la creación de parcelas
        *[default] Sustituyendo parcelas
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] Granjeando
        *[no] No granjeando
    }
running_farmer_farm_farming = Granjeando
running_farmer_farm_waiting_for_node_to_sync = Esperando a que el nodo se sincronice
running_farmer_farm_sector = Sector {$sector_index}
running_farmer_farm_sector_up_to_date = Sector {$sector_index}: al día
running_farmer_farm_sector_waiting_to_be_plotted = Sector {$sector_index}: esperando a que se cree su parcela
running_farmer_farm_sector_about_to_expire = Sector {$sector_index}: a punto de expirar, esperando a que se sustituya su parcela
running_farmer_farm_sector_expired = Sector {$sector_index}: expirada, esperando a que se cree su parcela
running_farmer_farm_sector_downloading = Sector {$sector_index}: descargando
running_farmer_farm_sector_encoding = Sector {$sector_index}: procesando
running_farmer_farm_sector_writing = Sector {$sector_index}: guardando

shutting_down_title = Apagando
shutting_down_description = Puede ser que tarde unos minutos depende la actividad de la aplicación

stopped_title = Parado
stopped_message = Parado 🛑
stopped_message_with_error = Parado con error: {$error}
stopped_button_show_logs = Mira la traza
stopped_button_help_from_community = Ayuda de la comunidad

error_title = Error
error_message = Error: {$error}
error_message_failed_to_send_config_to_backend = Fallo al enviar la configuración al backend: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Fallo al enviar la pausa de la granja al backend: {$error}
error_button_help_from_community = Ayuda de la comunidad
error_button_reset_node = Reiniciar nodo
error_button_reset_node_tooltip = Borrar datos del nodo y sincronizar de nuevo desde la red
error_button_show_logs = Mira la traza

new_version_available = Versión {$version} disponible 🎉
new_version_available_button_open = Abrir página de actualizaciones

main_menu_show_logs = Mira la traza en el sistema de archivos
main_menu_change_configuration = Cambiar configuración
main_menu_share_feedback = Compartir feedback
main_menu_about = Sobre la apliación
main_menu_exit = Salir

status_bar_message_configuration_is_invalid = La configuración es invalida: {$error}
status_bar_message_restart_is_needed_for_configuration = La aplicación necesita reiniciarse para que los cambios tengan efecto
status_bar_message_failed_to_save_configuration = Fallo al guardar los cambios: {$error}
status_bar_message_restarted_after_crash = Space Acres se ha reiniciado automáticamente después de un error, mira la traza de la aplicación y del sistema para más detalles.
status_bar_message_running_elevated = Space Acres se está ejecutando como root o administrador, lo cual no se recomienda. Reinícialo como usuario normal
status_bar_button_migrate = Migrar
status_bar_button_ok = Vale
status_bar_button_restart = Reiniciar

about_system_information =
    Carpeta de configuración: {$config_directory}
    Carpeta de datos (incluyendo trazas): {$data_directory}

tray_icon_open = Abierto
tray_icon_quit = Cerrado

notification_app_minimized_to_tray = Space Acres fue minimizado a la bandeja
    .body = Puedes abrirlo de nuevo o salir completamente usando el menú del icono en la bandeja
notification_stopped_with_error = Space Acres se detuvo con un error
    .body = Ocurrió un error y se requiere la intervención del usuario para resolverlo
notification_farm_error = Una de las granjas falló en Space Acres
    .body = Ocurrió un error y se requiere la intervención del usuario para resolverlo
notification_node_low_disk_space = Poco espacio en disco del nodo
    .body = El volumen del nodo sólo tiene {$free_space} restante
notification_missed_reward = Falló la firma de la recompensa 😞
    .body = Esto es desafortunado, pero habrá otra oportunidad pronto
notification_signed_reward_successfully = Nueva recompensa firmada con éxito 🥳
    .body = Gracias por asegurar la red 🙌

warning_low_disk_space = Poco espacio en disco en el volumen del nodo
warning_low_disk_space_detail = Solo quedan {$free_space}. Considere migrar a un disco más grande.
