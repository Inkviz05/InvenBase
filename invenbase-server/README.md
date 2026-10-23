# InvenBase backend

REST API на Rust 2021, Actix Web и PostgreSQL. Доступ к БД реализован через SQLx; migrations встроены в бинарник и применяются при старте.

## Запуск

Нужны Rust/Cargo и доступный PostgreSQL:

```powershell
Copy-Item .env.example .env
# создайте базу из DATABASE_URL и отредактируйте .env
cargo run
```

Проверка процесса:

```text
GET http://127.0.0.1:8080/health
```

`/health` не проверяет БД. Отдельного readiness endpoint в коде нет.

## Конфигурация

| Переменная | Обязательность и значение |
| --- | --- |
| `DATABASE_URL` | Не обязательна кодом; default `postgresql://postgres:postgres@localhost:5432/kvantoriym` |
| `DATABASE_MAX_CONNECTIONS` | Default `10`, целое число больше нуля |
| `DATABASE_ACQUIRE_TIMEOUT_SECONDS` | Default `5`, целое число больше нуля |
| `HOST` | Default `127.0.0.1` |
| `PORT` | Default `8080` |
| `JWT_SECRET` | Обязательна, минимум 32 символа |
| `JWT_EXPIRATION` | Default `86400` секунд |
| `CORS_ALLOWED_ORIGINS` | Список через запятую; default localhost/127.0.0.1:3000 |
| `CREATE_DEFAULT_ADMIN` | Default `false` |
| `DEFAULT_ADMIN_PASSWORD` | Обязательна при bootstrap, минимум 12 символов |
| `DEFAULT_ADMIN_USERNAME` | Default `admin` |
| `DEFAULT_ADMIN_EMAIL` | Default `admin@kvantoriym.local` |
| `DEFAULT_ADMIN_FULL_NAME` | Default `Administrator` |
| `FCM_PROJECT_ID` | Опциональный project ID для FCM HTTP v1 |
| `FCM_SERVICE_ACCOUNT_PATH` | Опциональный путь к service account JSON для HTTP v1 |
| `FCM_SERVER_KEY` | Опциональный legacy fallback; legacy FCM отключён Google |
| `RUSTORE_PROJECT_ID` | Опциональный ID проекта RuStore Push |
| `RUSTORE_SERVICE_TOKEN` | Опциональный сервисный токен RuStore Push; хранить только в секретах окружения |
| `RUST_LOG` | Фильтр логов; backend по умолчанию использует `info` |

Пустые строки FCM сейчас не нормализуются в `None`; лучше удалять незаданные FCM-переменные из `.env`, а не оставлять `KEY=`.

### Первый администратор

При `CREATE_DEFAULT_ADMIN=true` пользователь создаётся только в том случае, если username ещё отсутствует. Изменение `DEFAULT_ADMIN_PASSWORD` не сбрасывает пароль существующего пользователя. После первого успешного запуска установите `CREATE_DEFAULT_ADMIN=false`.

## HTTP API

Все перечисленные маршруты, кроме login, требуют Bearer JWT. Роли реально проверяются middleware и handlers.

- `POST /api/auth/login`
- Users: `GET /api/users/me`; admin-only `GET|POST /api/users`; authenticated `GET|PUT /api/users/{id}`; admin-only `DELETE /api/users/{id}`.
- Equipment: `GET|POST /api/equipment`, `GET|PUT|DELETE /api/equipment/{id}`, `GET /api/equipment/qr/{qr_code}`, `GET /api/equipment/{id}/movements`, `POST /api/equipment/{id}/move`. Изменения требуют `admin` или `responsible`.
- Categories: `GET|POST /api/categories`, `GET|PUT|DELETE /api/categories/{id}`; изменения требуют `admin` или `responsible`.
- Groups: `GET|POST /api/groups`, `GET|PUT|DELETE /api/groups/{id}`; изменения требуют `admin` или `responsible`.
- Squads: `GET|POST /api/squads`, `GET|PUT|DELETE /api/squads/{id}`, `GET /api/squads/{id}/equipment`; изменения требуют `admin` или `responsible`.
- Bookings: `GET|POST /api/bookings`, `POST /api/bookings/bulk`, `GET|PUT|DELETE /api/bookings/{id}`, `POST /api/bookings/{id}/approve|reject|cancel|return`. Approve/reject/return требуют `admin` или `responsible`; `DELETE` оставлен как legacy cancel.
- Permissions: `GET|POST /api/permissions`, `POST /api/permissions/{id}/revoke`.
- Notifications: `GET|POST /api/notifications`, `GET /api/notifications/count`, `GET /api/notifications/unread-count`, `POST /api/notifications/{id}/read`, `POST /api/notifications/read-all`.
- Reports: `GET /api/reports/equipment`, `/bookings`, `/bookings/detailed`, `/audit`; требуют `admin` или `responsible`.
- Logs: `GET /api/logs`; требует `admin` или `responsible`.
- QR: `GET /api/qr/{id}` требует `admin` или `responsible`; `GET /api/qr/{id}/data` требует авторизацию.
- Push: `POST /api/push/register-token` регистрирует или обновляет конкретное устройство; `POST /api/push/unregister-device` деактивирует устройство при logout.
- Support: `GET|POST /api/support/requests`, admin-only `PUT|DELETE /api/support/requests/{id}`, authenticated `POST /api/support/requests/{id}/messages`.

Отдельной OpenAPI/Swagger-спецификации нет.

## Встроенные страницы

Backend отдаёт legacy HTML из `static/` по `/` и `/admin`. Это не React web-клиент. В Docker Compose React build отдаётся nginx, а `/api` и `/health` проксируются к backend.

## Фоновые задачи и push

При старте запускаются проверка истёкших бронирований и worker очереди push-доставки. In-app уведомление и задание в `notification_outbox` создаются одной транзакцией, а payload связывает их через `notification_id`. Затем push доставляется на все активные FCM-устройства; временные ошибки повторяются с exponential backoff, итог и последняя ошибка сохраняются в PostgreSQL.

Предпочтительная FCM-конфигурация — `FCM_PROJECT_ID` + `FCM_SERVICE_ACCOUNT_PATH`; код самостоятельно получает OAuth token и вызывает FCM HTTP v1. `FCM_SERVER_KEY` поддерживается только как legacy fallback. Для RuStore задаются `RUSTORE_PROJECT_ID` и `RUSTORE_SERVICE_TOKEN`; backend вызывает официальный RuStore Push API и деактивирует токены с ответом `NOT_FOUND`. Реестр `user_devices` поддерживает несколько устройств и параллельную доставку через providers `fcm` и `rustore`; Huawei пока только допустим в схеме регистрации, transport ещё не реализован.

## Тесты

Быстрые library tests без БД:

```powershell
cargo test --locked --lib
```

Полный набор, включая `tests/booking_lifecycle.rs` и `tests/database_schema.rs`, требует `TEST_DATABASE_URL`:

```powershell
$env:TEST_DATABASE_URL = "postgresql://postgres:password@localhost:5432/kvantoriym_test"
cargo test --locked
```

Из корня репозитория доступен `scripts/test-backend.ps1`. Он использует `TEST_DATABASE_URL`, а при её отсутствии — `DATABASE_URL` из `.env`; это может затронуть основную локальную базу тестовыми записями.

## Известные ограничения

- Migrations запускаются приложением, отдельного migrate/rollback CLI нет.
- `/ready` отсутствует.
- HTTP integration tests маршрутов и middleware отсутствуют.
- Часть SQL и бизнес-логики всё ещё находится в handlers.
- `sqlx-postgres 0.7.4` выдаёт future-incompatibility warning на актуальном Rust, но текущую сборку не блокирует.
