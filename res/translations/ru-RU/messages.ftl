welcome_title = Добро пожаловать!
welcome_message =
    Space Acres это самодостаточное приложение с графическим интерфейсом для фарминга в Autonomys Network.

    Прежде чем продолжить, вам понадобится 3 вещи:
    ✔ Адрес кошелька, на который вы будете получать вознаграждения (используйте Subwallet, расширение polkadot{"{"}.js{"}"} или любой другой кошелек, совместимый с Substrate)
    ✔ 100Гб свободного места на твердотельном накопителе SSD хорошего качества для хранения данных блокчейна
    ✔ Любой твердотельный накопитель SSD (или несколько) с максимально возможным объемом памяти для фарминга. Чем больше объем - тем выше награда.
welcome_button_continue = Продолжить

upgrade_title = Обновление
upgrade_message =
    Спасибо, что снова выбрали Space Acres!

    Сеть, которую вы использовали до обновления, больше не совместима с этой версией Space Acres. Вероятно, это случилось потому, что вы участвовали в предыдущей версии Autonomys.

    Не волнуйтесь! Вы можете перейти на поддерживаемую в настоящее время сеть одним нажатием кнопки!
upgrade_button_upgrade = Обновить до {$chain_name}

loading_title = Загрузка
loading_configuration_title = Загрузка конфигурации
loading_configuration_step_loading = Загрузка конфигурации...
loading_configuration_step_reading = Чтение конфигурации...
loading_configuration_step_configuration_exists = Конфигурация найдена...
loading_configuration_step_configuration_not_found = Конфигурация отсутствует..
loading_configuration_step_configuration_checking = Проверка конфигурации...
loading_configuration_step_configuration_valid = Конфигурация прошла проверку без ошибок
loading_configuration_step_decoding_chain_spec = Обработка спецификации блокчейна...
loading_configuration_step_decoded_chain_spec = Спецификация блокчейна готова
loading_networking_stack_title = Инициализация сетевого стека
loading_networking_stack_step_checking_node_path = Проверка пути к папке с данными блокчейна...
loading_networking_stack_step_creating_node_path = Создание папки с данными блокчейна...
loading_networking_stack_step_node_path_ready = Папка с данными блокчейна подготовлена
loading_networking_stack_step_preparing = Подготовка сетевого стека...
loading_networking_stack_step_reading_keypair = Чтение сетевой ключевой пары...
loading_networking_stack_step_generating_keypair = Генерация сетевой ключевой пары...
loading_networking_stack_step_writing_keypair_to_disk = Запись сетевой ключевой пары на диск...
loading_networking_stack_step_instantiating = Инициализация сетевого стека...
loading_networking_stack_step_created_successfully = Сетевой стек успешно создан
loading_consensus_node_title = Инициализация узла консенсуса
loading_consensus_node_step_creating = Создание узла консенсуса...
loading_consensus_node_step_created_successfully = Узел консенсуса успешно создана
loading_farmer_title = Создание фермы
loading_farmer_step_initializing = Инициализация фермы {$index}/{$farms_total}...
loading_farmer_step_created_successfully = Ферма создана успешно
loading_wiping_farmer_data_title = Удаление данных фермы
loading_wiping_farmer_data_step_wiping_farm = Удаление фермы {$index}/{$farms_total} в {$path}...
loading_wiping_farmer_data_step_success = Все фермы удалены успешно
loading_wiping_node_data_title = Удаление данных блокчейна
loading_wiping_node_data_step_wiping_node = Удаление данных блокчейна в {$path}...
loading_wiping_node_data_step_success = Данные блокчейна успешно удалены

configuration_title = Конфигурация
reconfiguration_title = Реконфигурация
configuration_node_path = Путь к папке блокчейна
configuration_node_path_placeholder = Пример: {$path}
configuration_node_path_tooltip = Абсолютный путь к папке, в которой будут храниться данные блокчейна. Выделите для этого не менее 100Гб свободного места, рекомендуется использовать SSD хорошего качества
configuration_node_path_button_select = Выбрать
configuration_node_path_error_doesnt_exist_or_write_permissions = Папка не существует или у пользователя отсутствуют права на запись
configuration_node_migrate_button = Мигрировать...
configuration_node_migrate_tooltip = Миграция или сброс базы данных узла
configuration_node_size = Размер узла: {$size}
configuration_node_volume_free_space = Свободное место: {$size}
configuration_reward_address = Адрес для получения вознаграждений
configuration_reward_address_placeholder = Пример: {$address}
configuration_reward_address_tooltip = Используйте Subwallet, расширение polkadot{"{"}.js{"}"} или любой другой кошелек, совместимый с Substrate chain (адрес в формате SS58).
configuration_reward_address_button_create_wallet = Создать кошелек
configuration_reward_address_error_evm_address = Это должен быть адрес в формате Substrate (SS58) (любая сеть), а не EVM адрес
configuration_farm = Путь к ферме {$index} и её размер
configuration_farm_path_placeholder = Пример: {$path}
configuration_farm_path_tooltip = Абсолютный путь к папке, в которой будут храниться данные фермы. Подойдет любой SSD-накопитель, высокая производительность не требуется
configuration_farm_path_button_select = Выбрать
configuration_farm_path_error_doesnt_exist_or_write_permissions = Папка не существует или у пользователя отсутствуют права на запись
configuration_farm_size_kind_fixed = Фиксированный размер
configuration_farm_size_kind_free_percentage = % свободного места
configuration_farm_fixed_size_placeholder = Пример: 4T, 2.5TB, 500GiB, и т.д.
configuration_farm_fixed_size_tooltip = Размер фермы в зависимости от того, какие единицы измерения вы предпочитаете. Подойдет любой размер свыше 2Гб
configuration_farm_free_percentage_size_placeholder = Пример: 100%, 1.1%, и т.д.
configuration_farm_free_percentage_size_tooltip = Процент свободного места на диске, занимаемого фермой. Будет работать от 0%, но на диске должно оставаться не менее 2Гб свободного места для исключения ошибок.
configuration_farm_delete = Удалить эту ферму
configuration_advanced = Расширенная конфигурация
configuration_advanced_farmer = Конфигурация фермы
configuration_advanced_farmer_reduce_plotting_cpu_load = Уменьшить нагрузку на процессор при плоттинге
configuration_advanced_farmer_reduce_plotting_cpu_load_tooltip = Первичный плоттинг использует все ядра процессора по умолчанию. При использовании этой опции, для плоттинга используется половина ядер процессора, что позволит использовать компьютер для выполнения других задач
configuration_advanced_network = Конфигурация сети
configuration_advanced_network_default_port_number_tooltip = Номер порта по умолчанию - {$port}
configuration_advanced_network_substrate_port = Substrate (блокчейн) P2P порт (TCP):
configuration_advanced_network_subspace_port = Subspace (DSN) P2P порт (TCP):
configuration_advanced_network_faster_networking = Быстрая сеть:
configuration_advanced_network_faster_networking_tooltip = По умолчанию сетевые настройки оптимизированы для домашних маршрутизаторов. Если у вас более мощное оборудование, данная опция может улучшить скорость синхронизации и другие процессы
configuration_button_add_farm = Добавить ферму
configuration_button_help = Помощь
configuration_button_cancel = Отмена
configuration_button_back = Назад
configuration_button_save = Сохранить
configuration_button_start = Старт
configuration_dialog_button_select = Выбрать
configuration_dialog_button_cancel = Отмена

node_migration_button_cancel = Отмена
node_migration_button_reset = Сбросить узел
node_migration_button_start = Начать миграцию
node_migration_destination_free_space = Свободное место: {$size}
node_migration_destination_label = Новое расположение узла:
node_migration_destination_placeholder = Выберите папку назначения
node_migration_dialog_title = Миграция базы данных узла
node_migration_insufficient_space_warning = Предупреждение: недостаточно свободного места в месте назначения
node_migration_mode_fresh_sync = Новая синхронизация в новом месте
node_migration_mode_fresh_sync_explanation = Синхронизирует новую базу данных из сети в новом месте. Часто быстрее, чем миграция, особенно если ваш узел рассинхронизирован, и требует меньше места на целевом диске.
node_migration_mode_migrate = Миграция базы данных
node_migration_mode_migrate_explanation = Перемещает существующую базу данных в новое место. Требует достаточно места в месте назначения для текущей базы данных.
node_migration_mode_reset = Сбросить и пересинхронизировать на месте
node_migration_mode_reset_explanation = Сбрасывает ваш узел, удаляя базу данных и синхронизируя заново из сети. Используйте, если база данных вашего узла повреждена или значительно рассинхронизирована.
node_migration_non_node_data_warning = Примечание: В этой директории обнаружены данные, не относящиеся к ноде, и они не будут перенесены
node_migration_source_label = Текущее расположение:
node_migration_status_completed = Миграция успешно завершена!
node_migration_status_copying = Миграция базы данных: {$percentage}%
node_migration_status_deleting_source = Удаление предыдущей базы данных...
node_migration_status_failed = Миграция не удалась: {$error}
node_migration_status_restarting = Перезапуск Space Acres...
node_migration_status_shutting_down = Остановка узла...
node_migration_status_updating_config = Обновление конфигурации...
node_migration_status_verifying = Проверка базы данных...
node_migration_title = Миграция базы данных узла

running_title = Запущено
running_node_title = Узел блокчейна {$chain_name}
running_node_title_tooltip = Нажмите, чтобы открыть в файловом менеджере
running_node_free_disk_space_tooltip = Осталось свободного места на диске: {$size}
running_node_connections_tooltip = {$connected_peers}/{$expected_peers} подключенных пиров, нажмите для подробностей про нужные P2P порты
running_node_status_connecting = Подключение к сети, лучший блок #{$block_number}
running_node_status_syncing_speed_no_eta = , {NUMBER($blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} блоков/сек
running_node_status_syncing_speed_hours_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} блоков/сек (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} часов осталось)
running_node_status_syncing_speed_minutes_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} блоков/сек (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} минут осталось)
running_node_status_syncing_speed_seconds_eta = , {NUMBER($a_blocks_per_second, minimumFractionDigits: 2, maximumFractionDigits: 2)} блоков/сек (~{NUMBER($b_hours_remaining, minimumFractionDigits: 2, maximumFractionDigits: 2)} секунд осталось)
running_node_status_syncing =
    {$sync_kind ->
        [dsn] Синхронизация из DSN
        [regular] Обычная синхронизация
        *[unknown] Неизвестный тип синхронизации {$sync_kind}
    } #{$best_block_number}/{$target_block}{$sync_speed}
running_node_status_synced = Синхронизовано, лучший блок #{$best_block_number}
running_farmer_title = Ферма
running_farmer_button_expand_details = Показать детали о каждой ферме
running_farmer_button_pause_plotting = Приостановить плоттинг/реплоттинг. Обратите внимание, что текущее кодирование секторов не будет прервано
running_farmer_button_resume_plotting = Продолжить плоттинг
running_farmer_account_balance_tooltip = Общий баланс и монеты, заработанные с момента запуска приложения. Нажмите, чтобы увидеть подробности в Astral
running_farmer_piece_cache_sync = Синхронизация фрагментов кэша {NUMBER($percentage, minimumFractionDigits: 2, maximumFractionDigits: 2)}%
running_farmer_next_reward_estimate =
    Следующее вознаграждение:{$eta_string ->
        [any_time_now] с минуты на минуту
        [less_than_an_hour] меньше часа
        [today] сегодня
        [this_week] эта неделя
        [more_than_a_week] больше недели
        *[unknown] неизвестно
    }
running_farmer_farm_tooltip = Нажмите, чтобы открыть в файловом менеджере
running_farmer_farm_reward_signatures_tooltip = {$successful_signatures}/{$total_signatures} успешных подписей вознаграждения. Смотрите детали фарма, чтобы получить подробную информацию
running_farmer_farm_auditing_performance_tooltip = Эффективность аудита: среднее время {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, лимит времени {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_proving_performance_tooltip = Эффективность подтверждения: среднее время {NUMBER($a_average_time, minimumFractionDigits: 2, maximumFractionDigits: 2)}s, лимит времени {NUMBER($b_time_timit, minimumFractionDigits: 2, maximumFractionDigits: 2)}s
running_farmer_farm_non_fatal_error_tooltip = При фарминге произошла ошибка, которая была устранена. Более подробную информацию смотрите в журнале: {$error}
running_farmer_farm_crashed = Фарм сломался: {$error}
running_farmer_farm_plotting_speed =  ({NUMBER($a_sector_time, minimumFractionDigits: 2, maximumFractionDigits: 2)} мин/сектор, {NUMBER($b_sectors_per_hour, minimumFractionDigits: 2, maximumFractionDigits: 2)} сектор/час)
running_farmer_farm_plotting_initial =
    {$pausing_state ->
        [pausing] Приостановка первичного плоттинга
        [paused] Первичный плоттинг приостановлен
        *[no] Первичный плоттинг
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] Фарминг
        *[no] Не фармится
    }
running_farmer_farm_replotting =
    {$pausing_state ->
        [pausing] Приостановка первичного плоттинга
        [paused] Первичный плоттинг приостановлен
        *[default] Реплоттинг
    } {NUMBER($b_progress, minimumFractionDigits: 2, maximumFractionDigits: 2)}%{$plotting_speed}, {$farming ->
        [yes] Фарминг
        *[no] Не фармится
    }
running_farmer_farm_farming = Фарминг
running_farmer_farm_waiting_for_node_to_sync = Ожидание синхронизации узла блокчейна
running_farmer_farm_sector = Сектор {$sector_index}
running_farmer_farm_sector_up_to_date = Сектор {$sector_index}: актуален
running_farmer_farm_sector_waiting_to_be_plotted = Сектор {$sector_index}: ожидает плоттинга
running_farmer_farm_sector_about_to_expire = Сектор {$sector_index}: истекает, ожидает реплоттинга
running_farmer_farm_sector_expired = Сектор {$sector_index}: истек, ожидает реплоттинга
running_farmer_farm_sector_downloading = Сектор {$sector_index}: скачивается
running_farmer_farm_sector_encoding = Сектор {$sector_index}: кодируется
running_farmer_farm_sector_writing = Сектор {$sector_index}: записывается

shutting_down_title = Выключение
shutting_down_description = Это может занять от нескольких секунд до нескольких минут, в зависимости от того, что делает приложение

stopped_title = Остановлен
stopped_message = Остановлен 🛑
stopped_message_with_error = Остановлен с ошибкой: {$error}
stopped_button_show_logs = Показать журнал
stopped_button_help_from_community = Помощь от сообщества

error_title = Ошибка
error_message = Ошибка: {$error}
error_message_failed_to_send_config_to_backend = Не удалось отправить конфигурацию: {$error}
error_message_failed_to_send_pause_plotting_to_backend = Не удалось приостановить плоттинг: {$error}
error_button_help_from_community = Помощь от сообщества
error_button_reset_node = Сбросить узел
error_button_reset_node_tooltip = Удалить данные узла и синхронизировать заново из сети
error_button_show_logs = Показать журнал

new_version_available = Доступна новая версия {$version} 🎉
new_version_available_button_open = Перейти к релизам

main_menu_show_logs = Показать журнал в файловом менеджере
main_menu_change_configuration = Изменить конфигурацию
main_menu_share_feedback = Оставить отзыв
main_menu_about = О программе
main_menu_exit = Выход

status_bar_message_configuration_is_invalid = Неверная конфигурация: {$error}
status_bar_message_restart_is_needed_for_configuration = Перезапустите приложение, чтобы изменения конфигурации вступили в силу
status_bar_message_failed_to_save_configuration = Не удалось сохранить изменения конфигурации: {$error}
status_bar_message_restarted_after_crash = Space Acres автоматически перезапускается после сбоя. Подробности можно найти в приложении и системном журнале
status_bar_message_running_elevated = Space Acres запущен от имени root или администратора, что не рекомендуется. Перезапустите его от имени обычного пользователя
status_bar_button_migrate = Перенести
status_bar_button_ok = Ok
status_bar_button_restart = Перезапустить

about_system_information =
    Директория конфигурации: {$config_directory}
    Директория данных (включая журнал): {$data_directory}

tray_icon_open = Открыть
tray_icon_quit = Выход

notification_app_minimized_to_tray = Space Acres был свернут
    .body = Вы можете открыть его снова или полностью выйти, используя значок в области уведомлений
notification_stopped_with_error = Space Acres остановился с ошибкой
    .body = Произошла ошибка, для устранения которой требуется вмешательство пользователя
notification_farm_error = Одна из ферм сломалась в Space Acres
    .body = Произошла ошибка, для устранения которой требуется вмешательство пользователя
notification_node_low_disk_space = Мало места на диске узла
    .body = На томе узла осталось только {$free_space}
notification_missed_reward = Не удалось подписать вознаграждение 😞
    .body = Это досадно, но скоро представится еще один шанс
notification_signed_reward_successfully = Успешно подписано новое вознаграждение 🥳
    .body = Спасибо за обеспечение безопасности сети 🙌

warning_low_disk_space = Мало места на диске узла
warning_low_disk_space_detail = Осталось только {$free_space}. Рассмотрите возможность миграции на диск большего размера.
