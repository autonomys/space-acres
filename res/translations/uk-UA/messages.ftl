welcome_title = Ласкаво просимо
welcome_message =
    Space Acres — це функціональний GUI-додаток для фармінгу в мережі Autonomys Network

    Перш ніж продовжити, вам потрібні 3 речі:
    ✔ Адреса гаманця, на яку ви отримуватимете винагороди (використовуйте Subwallet, розширення polkadot{"{"}.js{"}"} або будь-який інший гаманець, сумісний з Substrate).
    ✔ 100 ГБ простору на якісному SSD для зберігання даних вузла
    ✔ Будь-які SSD (або кілька), з максимально можливим обсягом простору, який ви можете собі дозволити для цілей фармінгу — саме це буде генерувати винагороди.
welcome_button_continue = Продовжити

upgrade_title = Оновлення
upgrade_message =
    Дякуємо, що обрали Space Acres!

    Мережа на якій ви працювали до оновлення більше не сумісна з цією версією Space Acres, ймовірно, через вашу участь у попередній версії Autonomys.

    Але не хвилюйтеся, ви можете оновитись до підтримуваної мережі всього одним натисканням кнопки!
upgrade_button_upgrade = Оновити до {$chain_name}

loading_title = Завантаження
loading_configuration_title = Завантаження конфігурацій
loading_configuration_step_loading = Завантаження конфігурації...
loading_configuration_step_reading = Зчитування конфігурації...
loading_configuration_step_configuration_exists = Конфігурація знайдена
loading_configuration_step_configuration_not_found = Конфігурацію не знайдено
loading_configuration_step_configuration_checking = Перевірка конфігурації...
loading_configuration_step_configuration_valid = Конфігурація дійсна
loading_configuration_step_decoding_chain_spec = Декодування специфікації мережі...
loading_configuration_step_decoded_chain_spec = Специфікацію мережі успішно декодовано
loading_networking_stack_title = Ініціалізація мережевого стеку
loading_networking_stack_step_checking_node_path = Перевірка шляху до вузла...
loading_networking_stack_step_creating_node_path = Створення шляху до вузла...
loading_networking_stack_step_node_path_ready = Шлях до вузла готовий
loading_networking_stack_step_preparing = Підготовка мережевого стеку...
loading_networking_stack_step_reading_keypair = Зчитування мережевої пари ключів...
loading_networking_stack_step_generating_keypair = Генерація мережевої пари ключів...
loading_networking_stack_step_writing_keypair_to_disk = Записування мережевої пари на диск...
loading_networking_stack_step_instantiating = Створення мережевого стеку...
loading_networking_stack_step_created_successfully = Стек мережі успішно створено
loading_consensus_node_title = Ініціалізація вузла консенсусу
loading_consensus_node_step_creating = Створення вузла консенсусу...
loading_consensus_node_step_created_successfully = Вузол консенсусу успішно створено
loading_farmer_title = Створення фармера
loading_farmer_step_initializing = Створення ферми {$index}/{$farms_total}...
loading_farmer_step_created_successfully = Фармера успішно створено
loading_wiping_farmer_data_title = Видалення даних фармера
loading_wiping_farmer_data_step_wiping_farm = Видалення ферми {$index}/{$farms_total} at {$path}...
loading_wiping_farmer_data_step_success = Всі ферми успішно видалені
loading_wiping_node_data_title = Видалення даних вузла
loading_wiping_node_data_step_wiping_node = Видалення вузла в {$path}...
loading_wiping_node_data_step_success = Дані вузла успішно видалені

configuration_title = Налаштування
reconfiguration_title = Переналаштування
configuration_node_path = Шлях до вузла
configuration_node_path_placeholder = Приклад: {$path}
configuration_node_path_tooltip = Шлях де будуть зберігатися файли вузла. Рекомендується виділити принаймні 100 ГіБ простору, бажано використовувати SSD гарної якості.
configuration_node_path_button_select = Обрати
configuration_node_path_error_doesnt_exist_or_write_permissions = Папка не існує або користувач не має прав на запис
configuration_node_migrate_button = Migrate...
# configuration_node_migrate_button = Міграція...
configuration_node_migrate_tooltip = Migrate or reset node database
# configuration_node_migrate_tooltip = Міграція або скидання бази даних вузла
configuration_node_size = Node size: {$size}
# configuration_node_size = Розмір вузла: {$size}
configuration_node_volume_free_space = Free space: {$size}
# configuration_node_volume_free_space = Вільний простір: {$size}
configuration_reward_address = Адрес для отримання винагород
configuration_reward_address_placeholder = Приклад: {$address}
configuration_reward_address_tooltip = Використовуйте Subwallet або розширення polkadot{"{"}.js{"}"} або будь-який інший гаманець Substrate, сумісний з Substrate (адреса у форматі SS58)
configuration_reward_address_button_create_wallet = Створити гаманець
configuration_reward_address_error_evm_address = Це має бути адреса у форматі Substrate (SS58) (підходить будь-яка мережа), а не адреса EVM
configuration_farm = Шлях до ферми {$index} та її розмір
configuration_farm_path_placeholder = Приклад: {$path}
configuration_farm_path_tooltip = Шлях, де будуть зберігатися файли ферми. Будь-який SSD підійде, висока витривалість не є обов'язковою.
configuration_farm_path_button_select = Обрати
configuration_farm_path_error_doesnt_exist_or_write_permissions = Папка не існує або користувач не має прав на запис
configuration_farm_size_kind_fixed = Фіксований розмір
configuration_farm_size_kind_free_percentage = % вільного простору
configuration_farm_fixed_size_placeholder = Приклад: 4T, 2.5TB, 500GiB, і тд.
configuration_farm_fixed_size_tooltip = Розмір ферми в будь-яких одиницях, яким ви віддаєте перевагу, будь-яка кількість простору понад 2 ГБ підійде
configuration_farm_free_percentage_size_placeholder = Приклад: 100%, 1.1%, і тд.
configuration_farm_free_percentage_size_tooltip = Відсоток вільного дискового простору, який займатиме ця ферма. Будь-яке значення понад 0% підійде, але на диску повинно залишатися принаймні 2 ГБ вільного місця, щоб уникнути помилок.
configuration_farm_delete = Видалити ферму
configuration_advanced = Розширені конфігурації
configuration_advanced_farmer = Конфігурації Фармера
configuration_advanced_farmer_reduce_plotting_cpu_load = Зменшити навантаження на процесор при плотингу
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = Початковий плотинг за замовчуванням використовує всі ядра процесора, тоді як з цією опцією він почне використовувати половину ядер як під час реплотингу, що покращить реагування системи для інших завдань
configuration_advanced_network = Налаштування мережі
configuration_advanced_network_default_port_number_tooltip = Порт за замовчуванням: {$port}
configuration_advanced_network_substrate_port = Substrate (вузол) P2P порт (TCP):
configuration_advanced_network_subspace_port = Subspace (DSN) P2P порт (TCP):
configuration_advanced_network_faster_networking = Швидша мережа:
configuration_advanced_network_faster_networking_tooltip = За замовчуванням мережа оптимізована для споживчих маршрутизаторів, але якщо у вас є більш потужна конфігурація, швидша мережа може покращити швидкість синхронізації та інші процеси
configuration_button_add_farm = Додати ферму
configuration_button_help = Допомога
configuration_button_cancel = Скасувати
configuration_button_back = Повернутись
configuration_button_save = Зберегти
configuration_button_start = Розпочати
configuration_dialog_button_select = Обрати
configuration_dialog_button_cancel = Скасувати

node_migration_button_cancel = Cancel
# node_migration_button_cancel = Скасувати
node_migration_button_reset = Reset Node
# node_migration_button_reset = Скинути вузол
node_migration_button_start = Start Migration
# node_migration_button_start = Почати міграцію
node_migration_destination_free_space = Free space: {$size}
# node_migration_destination_free_space = Вільний простір: {$size}
node_migration_destination_label = New node location:
# node_migration_destination_label = Нове розташування вузла:
node_migration_destination_placeholder = Select destination folder
# node_migration_destination_placeholder = Виберіть папку призначення
node_migration_dialog_title = Migrate Node Database
# node_migration_dialog_title = Міграція бази даних вузла
node_migration_insufficient_space_warning = Warning: Not enough free space at destination
# node_migration_insufficient_space_warning = Попередження: недостатньо вільного місця в місці призначення
node_migration_mode_fresh_sync = Fresh sync to new location
# node_migration_mode_fresh_sync = Нова синхронізація в новому місці
node_migration_mode_fresh_sync_explanation = Syncs a fresh database from the network at the new location. Often faster than migrating, especially if your node is out of sync, and requires less destination space.
# node_migration_mode_fresh_sync_explanation = Синхронізує нову базу даних з мережі в новому місці. Часто швидше, ніж міграція, особливо якщо ваш вузол розсинхронізований, і потребує менше місця на цільовому диску.
node_migration_mode_migrate = Migrate database
# node_migration_mode_migrate = Мігрувати базу даних
node_migration_mode_migrate_explanation = Moves the existing database to the new location. Requires enough destination space for the current database.
# node_migration_mode_migrate_explanation = Переміщує існуючу базу даних в нове місце. Потребує достатньо місця в місці призначення для поточної бази даних.
node_migration_mode_reset = Reset and resync in place
# node_migration_mode_reset = Скинути і пересинхронізувати на місці
node_migration_mode_reset_explanation = Resets your node by wiping the database and syncing fresh from the network. Use this if your node database is corrupted or significantly out of sync.
# node_migration_mode_reset_explanation = Скидає ваш вузол, видаляючи базу даних і синхронізуючи заново з мережі. Використовуйте, якщо база даних вашого вузла пошкоджена або значно розсинхронізована.
node_migration_non_node_data_warning = Note: Non-node data detected in this directory and will not be migrated
# node_migration_non_node_data_warning = Примітка: У цій директорії виявлено дані, що не належать до ноди, і вони не будуть перенесені
node_migration_source_label = Current location:
# node_migration_source_label = Поточне розташування:
node_migration_status_completed = Migration completed successfully!
# node_migration_status_completed = Міграцію успішно завершено!
node_migration_status_copying = Migrating database: {$percentage}%
# node_migration_status_copying = Міграція бази даних: {$percentage}%
node_migration_status_deleting_source = Removing previous database...
# node_migration_status_deleting_source = Видалення попередньої бази даних...
node_migration_status_failed = Migration failed: {$error}
# node_migration_status_failed = Міграція не вдалася: {$error}
node_migration_status_restarting = Restarting Space Acres...
# node_migration_status_restarting = Перезапуск Space Acres...
node_migration_status_shutting_down = Shutting down node...
# node_migration_status_shutting_down = Зупинка вузла...
node_migration_status_updating_config = Updating configuration...
# node_migration_status_updating_config = Оновлення конфігурації...
node_migration_status_verifying = Verifying database...
# node_migration_status_verifying = Перевірка бази даних...
node_migration_title = Migrating Node Database
# node_migration_title = Міграція бази даних вузла

running_title = Запущено
running_node_title = {$chain_name} вузол консенсусу
running_node_title_tooltip = Натисніть щоб відкрити в файловому менеджері
running_node_free_disk_space_tooltip = Вільний дисковий простір: {$size} remaining
running_node_connections_tooltip = {$connected_peers}/{$expected_peers} пірів підключено, натисніть для деталей про необхідні P2P порти
running_node_status_connecting = Підключення до мережі, кращий блок #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} блоки/с
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} блоки/с (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} годин залишилось)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} блоки/с (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} хвилин залишилось)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} блоки/с (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} секунд залишилось)
running_node_status_syncing =
    {$sync_kind ->
        [dsn] Синхронізація з DSN
        [regular] Звичайна синхронізація
        *[unknown] Невідомий тип синхронізації {$sync_kind}
    } #{$best_block_number}/{$target_block}{$sync_speed}
running_node_status_synced = Синхронізовано, кращий блок #{$best_block_number}
running_farmer_title = Фармер
running_farmer_button_expand_details = Показати деталі про кожну ферму
running_farmer_button_pause_plotting = Призупення плотингу/реплотингу, зверніть увагу, що розпочате кодування секторів не буде перервано
running_farmer_button_resume_plotting = Продовжити плоттинг
running_farmer_account_balance_tooltip = Загальний баланс рахунку та монет, зароблених з моменту запуску програми, натисніть, щоб побачити деталі в Astral
running_farmer_piece_cache_sync = Синхронізація фрагментів кешу {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Наступна винагорода: {$eta_string ->
        [any_time_now] у будь-який момент
        [less_than_an_hour] менше години
        [today] сьогодні
        [this_week] цього тижня
        [more_than_a_week] більше тижня
        *[unknown] невідомо
    }
running_farmer_farm_tooltip = Натисніть щоб відкрити в файловому менеджері
running_farmer_farm_reward_signatures_tooltip = {$successful_signatures}/{$total_signatures} Успішні підписи винагороди, перегляньте деталі ферми, щоб побачити більше інформації
running_farmer_farm_auditing_performance_tooltip = Аудит ефективності: середній час {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}с, ліміт часу {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}с
running_farmer_farm_proving_performance_tooltip = Підтвердження ефективності: середній час {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}с, ліміт часу {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}с
running_farmer_farm_non_fatal_error_tooltip = При фармінгу сталася помилка яка була усунена. Перегляньте журнали для отримання додаткової інформації: {$error}
running_farmer_farm_crashed = Ферма аварійно завершила роботу: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} хв/сектор, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} сектори/г)
running_farmer_farm_plotting_initial =
    {$pausing_state ->
        [pausing] Призупинення початкового плотингу
        [paused] Початковий плотинг призупинений
        *[no] Початковий плотинг
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] фармить
        *[no] не фармить
    }
running_farmer_farm_replotting =
    {$pausing_state ->
        [pausing] Призупинення початкового плотингу
        [paused] Початковий плотинг призупинений
        *[default] Реплотинг
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] фармить
        *[no] не фармить
    }
running_farmer_farm_farming = Фармінг
running_farmer_farm_waiting_for_node_to_sync = Очікування вузла для синхронізації
running_farmer_farm_sector = Сектор {$sector_index}
running_farmer_farm_sector_up_to_date = Сектор {$sector_index}: актуальна версія
running_farmer_farm_sector_waiting_to_be_plotted = Сектор {$sector_index}: очікування плотингу
running_farmer_farm_sector_about_to_expire = Сектор {$sector_index}: наближається до закінчення терміну дії, очікує реплотингу
running_farmer_farm_sector_expired = Сектор {$sector_index}: Термін дії закінчився, очікує реплотинг
running_farmer_farm_sector_downloading = Сектор {$sector_index}: завантаження
running_farmer_farm_sector_encoding = Сектор {$sector_index}: кодується
running_farmer_farm_sector_writing = Сектор {$sector_index}: записується

shutting_down_title = Вимкнення
shutting_down_description = Це може зайняти кілька секунд або кілька хвилин, залежно від того, що робить програма

stopped_title = Зупинено
stopped_message = Зупинено 🛑
stopped_message_with_error = Зупинено з помилкою: {$error}
stopped_button_show_logs = Показати журнал
stopped_button_help_from_community = Допомога від спільноти

error_title = Помилка
error_message = Помилка: {$error}
error_message_failed_to_send_config_to_backend = Не вдалося надіслати конфігурацію: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Не вдалось призупинити плотинг: {$error}
error_button_help_from_community = Допомога від спільноти
# error_button_help_from_community = Допомога від спільноти
error_button_reset_node = Reset node
# error_button_reset_node = Скинути вузол
error_button_reset_node_tooltip = Wipe node data and sync fresh from the network
# error_button_reset_node_tooltip = Видалити дані вузла та синхронізувати заново з мережі
error_button_show_logs = Показати журнал
# error_button_show_logs = Показати журнал

new_version_available = Версія {$version} доступна 🎉
new_version_available_button_open = Перейти до релізів

main_menu_show_logs = Показати журнал у файловому менеджері
main_menu_change_configuration = Змінити конфігурацію
main_menu_share_feedback = Поділитись відгуком
main_menu_about = Про програму
main_menu_exit = Вийти

status_bar_message_configuration_is_invalid = Конфігурація недійсна: {$error}
status_bar_message_restart_is_needed_for_configuration = Для того щоб зміни конфігурації вступили в силу, потрібен перезапуск програми
status_bar_message_failed_to_save_configuration = Не вдалося зберегти зміни конфігурації: {$error}
status_bar_message_restarted_after_crash = Space Acres автоматично перезапустилася після неочікуваної помилки, перевірте журнали програми та системи для отримання деталей
status_bar_button_migrate = Migrate
# status_bar_button_migrate = Перенести
status_bar_button_ok = Ок
status_bar_button_restart = Перезапустити

about_system_information =
    Каталог конфігурації: {$config_directory}
    Каталог даних (включаючи журнали): {$data_directory}

tray_icon_open = Відкрити
tray_icon_quit = Вийти

notification_app_minimized_to_tray = Space Acres було згорнуто
    .body = Ви можете знову відкрити програму або повністю вийти використовуючи значок в меню
notification_stopped_with_error = Space Acres зупинилася з помилкою
    .body = Сталася помилка яка вимагає втручання користувача для її вирішення
notification_farm_error = Одна з ферм зазнала невдачі в Space Acres
    .body = Сталася помилка яка вимагає втручання користувача для її вирішення
notification_node_low_disk_space = Low Node Disk Space
    .body = Node volume has only {$free_space} remaining
# notification_node_low_disk_space = Мало місця на диску вузла
#     .body = На томі вузла залишилось лише {$free_space}
notification_missed_reward = Підписання винагороди не вдалося 😞
    .body = Це прикро, але найближчим часом буде інша можливість
notification_signed_reward_successfully = Успішно підписано нову винагороду 🥳
    .body = Дякую за забезпечення безпеки мережі 🙌

warning_low_disk_space = Low disk space on node volume
# warning_low_disk_space = Мало місця на диску вузла
warning_low_disk_space_detail = Only {$free_space} remaining. Consider migrating to a larger drive.
# warning_low_disk_space_detail = Залишилось лише {$free_space}. Розгляньте можливість міграції на більший диск.
