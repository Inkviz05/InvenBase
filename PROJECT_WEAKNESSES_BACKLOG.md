# Технический бэклог InvenBase

Проверено по репозиторию 24 октября 2026 года. Завершённые старые пункты удалены из списка; история доступна в Git.

## Критично перед production

1. **Нет TLS-конфигурации.** Compose публикует nginx по HTTP. Нужен внешний TLS-терминатор или отдельная production-конфигурация.
2. [Закрыто] Добавлены отдельный migrate binary/Compose job, документированная production-процедура, backup/restore-скрипты и CI restore drill. Необратимые migrations откатываются восстановлением проверенного backup.
3. [Закрыто] Добавлен `/ready` с запросом к PostgreSQL и `503` при недоступной БД; Docker healthcheck использует readiness, `/health` остаётся liveness-проверкой.
4. [Закрыто] Опубликованы OpenAPI 3.1 и Swagger UI, описаны все маршруты, JWT, единый error-контракт, предметные request/response schemas, path/query-параметры, JSON и бинарные ответы. Contract tests проверяют ключевые привязки и разрешение всех `$ref`. Отдельный следующий этап — генерация клиентских типов из контракта.
5. [Закрыто] CI поднимает чистый PostgreSQL 16 и выполняет полный `cargo test --locked`, включая DB/HTTP integration tests.

## Высокий приоритет

1. [Закрыто] Добавлены HTTP integration tests login, `401`/`403`, формата ошибок, request ID, multi-device notification flow, CRUD/role-сценариев users и equipment, а также полного lifecycle support.
2. [Закрыто] Пустые RuStore/Huawei env нормализуются как отсутствующие; из startup/push-логов убраны значения, пути, payload, текст уведомлений и фрагменты device token.
3. [Закрыто] Firebase/FCM полностью удалён из Android и backend: Google Services plugin, SDK, service, transport, OAuth/service-account код, env и зависимости. Миграция удаляет старые FCM device tokens.
4. [Закрыто] Реальные `.env`, mobile provider configs, service-account JSON, приватные ключи и keystore исключены из Git; tracked/staged файлы проверяет PowerShell guard, история Git сканируется Gitleaks в CI. Репозиторный pre-commit hook включается через `git config core.hooksPath .githooks`.
5. [Частично выполнено] Android auth/user/push-token storage защищён AES-GCM и Android Keystore. Остаются API URL по build type и release signing/minification.
6. [Частично выполнено] Android debug build добавлен в CI; unit/instrumentation tests появятся после добавления тестовых исходников.
7. Добавить role guards на маршрутах web, не полагаясь только на скрытие пунктов меню и backend `403`.
8. Продолжить provider-independent архитектуру: RuStore и Huawei реализованы на сервере и Android; остаются проверка transports на реальном устройстве и единое `NotificationEvent`.
9. [Закрыто] Добавлен HTTP integration test register/update/unregister устройства, нескольких устройств одной учётной записи и смены пользователя на одном физическом устройстве.
10. [Закрыто] Transactional outbox, retries/backoff и журнал доставки добавлены; in-app уведомление и outbox создаются одной транзакцией, payload содержит `notification_id`.
11. [Отложено] End-to-end проверка уведомлений на реальных Android-устройствах: multi-device доставка, восстановление WorkManager после потери сети и `onDeletedMessages`, деактивация устройства после logout. Выполнить позже по решению пользователя.
12. [Закрыто] Добавлен CI integration job с чистым PostgreSQL и HTTP-сценарий login → регистрация двух устройств → создание и чтение уведомления → outbox для обоих устройств.

## Средний приоритет

1. Продолжить разделение backend handlers и services/repositories.
2. [Закрыто] Введён документированный формат API errors `code/message/details/request_id`, безопасные internal errors и `X-Request-ID`.
3. Добавить structured logging, request ID и метрики.
4. Расширить web unit/e2e тесты на reports, support и полный lifecycle бронирования.
5. Разбить крупные React-страницы и внедрить lazy loading.
6. Ввести repository/ViewModel слой в Android и покрыть критичные сценарии тестами.
7. Добавить SSE для открытых web/Android-клиентов. WorkManager-фоновая сверка непрочитанных уведомлений уже реализована; foreground service не использовать без отдельной необходимости.

## Низкий приоритет

1. Систематизировать повторяющиеся web-компоненты: статусы, фильтры, таблицы и booking actions.
2. Рассмотреть TypeScript после стабилизации API-контрактов.
3. Проверить RuStore и Huawei transports на реальном устройстве. Huawei внедрён через общий серверный `PushTransport`, Android token lifecycle реализован; FCM fallback удалён.

## Подтверждённые ограничения текущей реализации

- Backend также содержит встроенные страницы `/` и `/admin` из `invenbase-server/static`; production Compose отдаёт React через nginx, поэтому эти страницы не являются основным web UI.
- PostgreSQL не опубликован наружу основным Compose-файлом.
- Android `usesCleartextTraffic=false`; локальный HTTP backend не заработает без debug network-security настройки или HTTPS.
- Android release build имеет `minifyEnabled false`.
- Автоматической ротации JWT/refresh token нет: API выдаёт один JWT с временем жизни из `JWT_EXPIRATION`.
- Отдельного logout endpoint на backend нет; клиенты удаляют локальный токен.
- `user_devices` моделирует provider/device lifecycle и несколько устройств; сервер и Android реализуют RuStore/Huawei. Оба provider ещё не проверены на зарегистрированных сборках реальных устройств.
- Для открытых клиентов нет SSE/WebSocket. Android WorkManager сверяет непрочитанные уведомления и восстанавливает пропущенные push; серверная очередь повторной доставки атомарно связана с in-app уведомлением.

Приоритетный порядок: readiness и production DB-процедура → HTTP/API tests и контракт → multi-device notifications/outbox/transports → CI с PostgreSQL → Android security/build → SSE/WorkManager → архитектурные рефакторинги.
