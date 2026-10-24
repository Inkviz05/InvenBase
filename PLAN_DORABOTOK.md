# План доработок InvenBase

Актуализировано по исходному коду 23 октября 2026 года. Это план, а не описание уже существующих возможностей.

## Что уже сделано

- Удалён старый Android WebView-клиент; активны React web и native Android Java.
- Секреты вынесены в env, `JWT_SECRET` и bootstrap-пароль валидируются.
- Добавлены SQLx migrations, ограничения и внешние ключи PostgreSQL.
- Основная логика жизненного цикла бронирования вынесена в services и покрыта unit/DB integration tests.
- Добавлены Dockerfiles, Compose, nginx reverse proxy, healthchecks и browser smoke полного стека.
- Добавлен GitHub Actions workflow: Android debug build, web audit/tests/build, `cargo fmt`, backend library tests, полный backend integration suite на чистом PostgreSQL 16 и Compose smoke.
- Убраны production `console.log`, directory listing отключён, параметры пула БД вынесены в env.
- Web использует локальный XLSX writer вместо зависимости `xlsx`.
- Завершён первый этап multi-device push: provider-aware `user_devices`, стабильный `device_id`, active/last-seen lifecycle, register/unregister, деактивация невалидных FCM-токенов и рабочий `FcmTransport`.
- Добавлена transactional outbox-очередь push: in-app уведомление и задание доставки создаются в одной транзакции, payload содержит `notification_id`; worker поддерживает атомарный claim, retries с exponential backoff, восстановление зависших задач и журнал последней ошибки/итогового статуса.

## Следующие задачи

### 1. Backend API и безопасность

1. Добавить HTTP integration tests через `actix_web::test` для login, ролей, users, equipment и support.
2. Унифицировать JSON-ошибки полями `code`, `message`, `details`; сейчас публичной спецификации нет.
3. Добавить OpenAPI и проверку контрактов для React и Android.
4. [Выполнено] Добавлен `/ready` с `SELECT 1` в PostgreSQL и ответом `503` при недоступной БД; `/health` оставлен простой liveness-проверкой. Docker и Compose smoke ожидают readiness.
5. Убрать подробное диагностическое логирование FCM-конфигурации и нормализовать пустые FCM env как `None`.

### 2. База данных и эксплуатация

1. Не использовать основную БД для integration tests: создавать отдельную БД и задавать `TEST_DATABASE_URL`.
2. [Выполнено] Добавлены отдельный migrate binary/Compose job, управляемые автамиграции для локальной разработки, PowerShell backup/restore с проверками и CI restore drill на чистой базе. Автоматического rollback migrations нет: recovery выполняется восстановлением проверенного backup.
3. Проверить исторические данные перед валидацией оставшихся `NOT VALID` constraints.
4. [Выполнено] Добавлен CI job с отдельным чистым PostgreSQL 16 для полного `cargo test --locked`, включая migrations и HTTP/DB integration tests.

### 3. Архитектура backend

1. Продолжить перенос SQL и бизнес-правил из handlers в service/repository слой.
2. Добавить транзакции для оставшихся операций, изменяющих несколько сущностей.
3. Перейти от `log` к структурированному `tracing`, добавить request/correlation ID.
4. Добавить метрики ошибок БД, push и времени ответа API.

### 4. Уведомления и доставка на устройства

Целевая архитектура не должна зависеть от Java/Kotlin/Flutter или конкретного push-провайдера.

1. Ввести единое серверное событие `NotificationEvent` и сохранять каждое уведомление в PostgreSQL как источник истины.
2. [Выполнено] Расширить модель устройств: `user_id`, стабильный `device_id`, `provider`, `push_token`, `platform`, `active`, `last_seen_at`. Одна учётная запись поддерживает несколько устройств и провайдеров на уровне схемы.
3. [Выполнено] Добавить API регистрации, обновления и удаления устройства. Android logout деактивирует текущее устройство для FCM, RuStore и Huawei.
4. [Выполнено] Введены интерфейс `PushTransport`, `FcmTransport`, `RuStoreTransport` и `HuaweiTransport`; Android получает и регистрирует токены всех трёх providers.
5. [Выполнено на backend] Событие рассылается на все активные устройства пользователя соответствующих providers; FCM `UNREGISTERED`, RuStore `NOT_FOUND` и Huawei `80300007` деактивируют токен.
6. [Выполнено] Добавлена PostgreSQL transactional outbox: in-app запись и outbox создаются одной транзакцией, payload включает `notification_id`, worker выполняет retries с exponential backoff и сохраняет статус/последнюю ошибку. Полное содержимое доступно из авторизованного notifications API.
7. Добавить SSE для мгновенного обновления открытых web/Android-клиентов. WebSocket использовать только если появится реальный двусторонний сценарий, например live-чат поддержки.
8. [Выполнено] Android WorkManager каждые 15 минут синхронизирует непрочитанные уведомления при наличии сети; immediate sync запускается после login и `onDeletedMessages` FCM/RuStore. Локальный реестр `notification_id` предотвращает повторный показ уже обработанных push.
9. Не использовать foreground service для постоянного соединения без отдельной продуктовой необходимости: он расходует батарею и требует постоянного системного уведомления.
10. [Частично выполнено] Миграции, multi-device/deactivation lifecycle, атомарное создание notification/outbox и outbox claim/retry/complete покрыты PostgreSQL integration tests, backoff и нормализация FCM env — unit tests. HTTP lifecycle login → два устройства → notification/outbox, а также register/update/unregister покрыт через `actix_web::test` и выполняется в CI; ещё нужны contract tests новых transports.
11. [Отложено] Провести end-to-end проверку уведомлений на реальных Android-устройствах: доставка на несколько устройств одной учётной записи, восстановление после потери сети и `onDeletedMessages`, а также прекращение доставки после logout. Вернуться к проверке, когда будет готово реальное устройство и пользователь решит продолжить ручное тестирование.

### 5. Web

1. Добавить route-level role guards; сейчас маршруты защищены фактом входа, а окончательная авторизация выполняется backend и частично UI.
2. Расширить Vitest-покрытие reports, support, equipment detail/edit и действий с бронированием.
3. Расширить Playwright flow на approve/reject/cancel/return и экспорт отчётов.
4. Разбить крупные страницы и добавить lazy loading; отдельного lint script сейчас нет.

### 6. Native Android

1. Убрать токены и пользовательские данные из обычных `SharedPreferences` в защищённое хранилище.
2. Вынести API URL в build types/flavors вместо одной константы и пользовательской настройки.
3. Ввести repository/ViewModel слой и сократить API/UI-логику в Activity/Fragment.
4. Добавить unit и instrumentation tests; сейчас Gradle содержит зависимости JUnit/Espresso, но тестовых исходников в проекте нет.
5. Проверить release signing, включить и протестировать R8/ProGuard.
6. [Частично выполнено] Android RuStore Push SDK 7.5.0 и Huawei Push SDK подключены: получение/обновление token, регистрация provider/device и деактивация при logout. Huawei включается при наличии `app/agconnect-services.json`. WorkManager-сверка пропущенных уведомлений реализована. Остаются проверка RuStore/Huawei на реальных устройствах и SSE.

### 7. Delivery

1. [Выполнено] Добавить Android debug build в CI на JDK 21 с Gradle cache и публикацией APK-артефакта.
2. Добавить secret scanning и проверку случайно staged `.env`/service-account файлов.
3. Определить production TLS и reverse proxy перед публичным развёртыванием; текущий Compose — production-shaped локальный стек без TLS.

## Не планируется без отдельного решения

- Переход на микросервисы.
- Переписывание backend на другом языке.
- Полная замена native Android на Flutter/React Native.
- Большой UI-редизайн до фиксации API-контрактов и тестового покрытия.
