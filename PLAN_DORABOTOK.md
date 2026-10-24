# План доработок InvenBase

Актуализировано по исходному коду 24 октября 2026 года. Это план, а не описание уже существующих возможностей.

## Что уже сделано

- Удалён старый Android WebView-клиент; активны React web и native Android Java.
- Секреты вынесены в env, `JWT_SECRET` и bootstrap-пароль валидируются.
- Добавлены SQLx migrations, ограничения и внешние ключи PostgreSQL.
- Основная логика жизненного цикла бронирования вынесена в services и покрыта unit/DB integration tests.
- Добавлены Dockerfiles, Compose, nginx reverse proxy, healthchecks и browser smoke полного стека.
- Добавлен GitHub Actions workflow: Android debug build, web audit/tests/build, `cargo fmt`, backend library tests, полный backend integration suite на чистом PostgreSQL 16 и Compose smoke.
- Убраны production `console.log`, directory listing отключён, параметры пула БД вынесены в env.
- Web использует локальный XLSX writer вместо зависимости `xlsx`.
- Завершён первый этап multi-device push: provider-aware `user_devices`, стабильный `device_id`, active/last-seen lifecycle, register/unregister и рабочие RuStore/Huawei transports. Firebase/FCM удалён.
- Добавлена transactional outbox-очередь push: in-app уведомление и задание доставки создаются в одной транзакции, payload содержит `notification_id`; worker поддерживает атомарный claim, retries с exponential backoff, восстановление зависших задач и журнал последней ошибки/итогового статуса.

## Следующие задачи

### 1. Backend API и безопасность

1. [Выполнено] Добавлены HTTP integration tests через `actix_web::test` для login, ролей, users, equipment и полного lifecycle support; тесты выполняются на отдельной PostgreSQL через `TEST_DATABASE_URL`.
2. [Выполнено] JSON-ошибки унифицированы полями `code`, `message`, `details`, `request_id`; добавлен `X-Request-ID`, внутренние диагностические сообщения не выдаются клиенту, `401` и `403` разделены.
3. [Выполнено] Добавлены OpenAPI 3.1 `/openapi.json`, Swagger UI `/docs`, все маршруты, Bearer JWT, единая error schema, предметные request/response schemas, path/query-параметры, JSON и бинарные ответы. Contract tests проверяют ключевые операции и разрешение всех `$ref`. Генерация клиентских типов вынесена в отдельный следующий этап.
4. [Выполнено] Добавлен `/ready` с `SELECT 1` в PostgreSQL и ответом `503` при недоступной БД; `/health` оставлен простой liveness-проверкой. Docker и Compose smoke ожидают readiness.
5. [Выполнено] Пустые значения RuStore/Huawei env нормализуются как `None`; startup-логи не раскрывают значения, пути или наличие секретов, а push-логи не содержат payload, текст уведомления и фрагменты device token.

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
3. [Выполнено] Добавить API регистрации, обновления и удаления устройства. Android logout деактивирует текущее устройство для RuStore и Huawei.
4. [Выполнено] Введены интерфейс `PushTransport`, `RuStoreTransport` и `HuaweiTransport`; Android получает и регистрирует tokens обоих providers. Firebase/FCM удалён из Android, backend, конфигурации и зависимостей.
5. [Выполнено на backend] Событие рассылается на все активные устройства пользователя соответствующих providers; RuStore `NOT_FOUND` и Huawei `80300007` деактивируют token.
6. [Выполнено] Добавлена PostgreSQL transactional outbox: in-app запись и outbox создаются одной транзакцией, payload включает `notification_id`, worker выполняет retries с exponential backoff и сохраняет статус/последнюю ошибку. Полное содержимое доступно из авторизованного notifications API.
7. Добавить SSE для мгновенного обновления открытых web/Android-клиентов. WebSocket использовать только если появится реальный двусторонний сценарий, например live-чат поддержки.
8. [Выполнено] Android WorkManager каждые 15 минут синхронизирует непрочитанные уведомления при наличии сети; immediate sync запускается после login и RuStore `onDeletedMessages`. Локальный реестр `notification_id` предотвращает повторный показ уже обработанных push.
9. Не использовать foreground service для постоянного соединения без отдельной продуктовой необходимости: он расходует батарею и требует постоянного системного уведомления.
10. [Частично выполнено] Миграции, multi-device/deactivation lifecycle, атомарное создание notification/outbox и outbox claim/retry/complete покрыты PostgreSQL integration tests. HTTP lifecycle login → два устройства → notification/outbox, а также register/update/unregister покрыт через `actix_web::test` и выполняется в CI; ещё нужны network contract tests transports.
11. [Отложено] Провести end-to-end проверку уведомлений на реальных Android-устройствах: доставка на несколько устройств одной учётной записи, восстановление после потери сети и `onDeletedMessages`, а также прекращение доставки после logout. Вернуться к проверке, когда будет готово реальное устройство и пользователь решит продолжить ручное тестирование.

### 5. Web

1. Добавить route-level role guards; сейчас маршруты защищены фактом входа, а окончательная авторизация выполняется backend и частично UI.
2. Расширить Vitest-покрытие reports, support, equipment detail/edit и действий с бронированием.
3. Расширить Playwright flow на approve/reject/cancel/return и экспорт отчётов.
4. Разбить крупные страницы и добавить lazy loading; отдельного lint script сейчас нет.

### 6. Native Android

1. [Выполнено] JWT, сериализованный пользователь и push-токены шифруются AES-GCM с ключом Android Keystore; plaintext-значения мигрируются и удаляются, зашифрованный файл исключён из backup.
2. [Выполнено] API URL вынесен в `BuildConfig` по build type: локальный emulator URL для debug, HTTPS production URL для release, оба переопределяются Gradle property/env. Пользовательское сохранение endpoint удалено.
3. Ввести repository/ViewModel слой и сократить API/UI-логику в Activity/Fragment.
4. Добавить unit и instrumentation tests; сейчас Gradle содержит зависимости JUnit/Espresso, но тестовых исходников в проекте нет.
5. [Выполнено] Release включает R8/resource shrinking и проверяется в CI; signing читается из `ANDROID_KEYSTORE_*`, а CI при наличии secrets публикует подписанные APK/AAB. Keystore и пароли не хранятся в Git.
6. [Частично выполнено] Android RuStore Push SDK 7.5.0 и Huawei Push SDK подключены: получение/обновление token, регистрация provider/device и деактивация при logout. Huawei включается при наличии `app/agconnect-services.json`. WorkManager-сверка пропущенных уведомлений реализована. Остаются проверка RuStore/Huawei на реальных устройствах и SSE.

### 7. Delivery

1. [Выполнено] Добавить Android debug и unsigned release builds в CI на JDK 21 с Gradle cache и публикацией debug APK-артефакта.
2. [Выполнено] Добавить secret scanning и проверку случайно staged `.env`/service-account/provider config/private key файлов: локальный pre-commit hook и CI guard проверяют имена и содержимое, Gitleaks сканирует историю Git.
3. Определить production TLS и reverse proxy перед публичным развёртыванием; текущий Compose — production-shaped локальный стек без TLS.

### 8. Multi-instance onboarding

1. Сделать один release-клиент для разных школ: при первом запуске показывать экран привязки к серверу вместо зашитого production URL.
2. Поддержать QR/setup-ссылку с HTTPS URL сервера и одноразовым кодом привязки; ручной ввод URL оставить запасным вариантом.
3. [Backend выполнен] Добавлены публичные метаданные instance, admin-only создание одноразового кода и его атомарный однократный обмен. Код без URL требует отдельного центрального каталога серверов и пока не является базовым вариантом.
4. Проверять HTTPS, ожидаемый API protocol/version и идентификатор instance до сохранения; конфигурацию защищать Android Keystore, позволять пользователю явно сбросить привязку.
5. [Частично выполнено] Setup-код имеет TTL 1–60 минут, хранится только как SHA-256 hash, атомарно помечается использованным и может быть отозван администратором. В QR/deep link не должны попадать credentials или долгоживущие secrets. До публичного развёртывания остаётся rate limit для публичного обмена.
6. Определить схему push для единого подписанного приложения. Ключи проектов RuStore/Huawei нельзя раздавать серверам школ: либо проект предоставляет центральный push-relay с отдельной аутентификацией каждого instance, либо автономная установка работает без внешнего push и восстанавливает уведомления через WorkManager при следующем доступном запуске/периодической синхронизации.

## Не планируется без отдельного решения

- Переход на микросервисы.
- Переписывание backend на другом языке.
- Полная замена native Android на Flutter/React Native.
- Большой UI-редизайн до фиксации API-контрактов и тестового покрытия.
