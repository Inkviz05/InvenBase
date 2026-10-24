# Docker Compose deployment

Основной `docker-compose.yml` поднимает:

- `postgres:16-alpine` с named volume `postgres_data`;
- одноразовый `migrate`, который применяет SQLx migrations и при необходимости создаёт первого администратора;
- multi-stage release image Rust backend;
- production React build, который отдаёт nginx.

Публичен только web/nginx на `WEB_PORT` (default `80`). nginx проксирует `/api`, `/health` и `/ready` в backend. Backend и PostgreSQL host-порты не публикуются.

## Запуск

```powershell
Copy-Item .env.docker.example .env
notepad .env
docker compose up --build
```

Обязательно задайте собственные значения:

- `POSTGRES_PASSWORD`;
- `JWT_SECRET` (минимум 32 символа);
- `DEFAULT_ADMIN_PASSWORD` при `CREATE_DEFAULT_ADMIN=true` (минимум 12 символов).

Открыть:

- UI: `http://localhost` или `http://localhost:<WEB_PORT>`;
- API: `/api` на том же origin;
- liveness: `/health` на том же origin;
- readiness с запросом к PostgreSQL: `/ready`.

## Первый admin

При первом запуске:

```env
CREATE_DEFAULT_ADMIN=true
```

После создания пользователя установите `false`. Bootstrap не обновляет пароль уже существующего username.

## Управление

```powershell
docker compose ps
docker compose logs -f backend
docker compose down
```

Backend в Compose запускается с `RUN_MIGRATIONS_ON_STARTUP=false` и ждёт успешного завершения `migrate`. При обычном локальном `cargo run` автамиграции по умолчанию остаются включёнными; их можно отключить через `RUN_MIGRATIONS_ON_STARTUP=false`.

Отдельный локальный запуск миграций:

```powershell
cd invenbase-server
cargo run --locked --bin migrate
```

Удаление БД вместе с volume — необратимая для этого volume операция:

```powershell
docker compose down -v
```

## Конфигурация

Пул backend настраивается через `DATABASE_MAX_CONNECTIONS` и `DATABASE_ACQUIRE_TIMEOUT_SECONDS`; defaults — `10` и `5` секунд.

Web image собирается с `VITE_API_URL=/api`. nginx имеет SPA fallback на `index.html`, прокси API/liveness/readiness; directory listing backend отключён.

## Push в Docker

Compose передаёт FCM env, но не монтирует service account JSON. Для FCM HTTP v1 deployment должен отдельно смонтировать файл внутрь backend container и указать container path в `FCM_SERVICE_ACCOUNT_PATH`. Не добавляйте JSON в image или git.

## Smoke test

```powershell
cd invenbase-client\web
npm ci
npx playwright install chromium
cd ..\..
.\scripts\test-docker-compose-smoke.ps1
```

Подробности: [REAL_SMOKE_TESTING.md](REAL_SMOKE_TESTING.md).

## Backup и восстановление

Скриптам нужны PostgreSQL client tools (`pg_dump`, `pg_restore`, `psql`, `createdb`) той же или более новой major-версии, чем сервер.

Создание backup в custom-формате, проверка архива и хранение последних 14 дней:

```powershell
$env:DATABASE_URL = "postgresql://postgres:password@localhost:5432/kvantoriym"
.\scripts\backup-postgres.ps1
```

По умолчанию файлы создаются в игнорируемом Git каталоге `backups/`. Можно задать `-OutputPath` и `-RetentionDays`. Пароль передаётся дочерним PostgreSQL-процессам через `PGPASSWORD` и не включается в их аргументы.

Безопасная проверка восстановления в отдельную базу:

```powershell
.\scripts\restore-postgres.ps1 `
  -BackupPath .\backups\invenbase-YYYYMMDD-HHMMSS.dump `
  -TargetDatabaseUrl "postgresql://postgres:password@localhost:5432/invenbase_restore_test" `
  -ConfirmRestore
```

Скрипт по умолчанию разрешает только имена баз, содержащие `test`, `restore` или `staging`. Для иной базы требуется явный `-AllowNonTestDatabase`; это не отменяет необходимость остановить запись данных и подтвердить выбранную точку восстановления.

Production-порядок обновления:

1. Остановить или заблокировать операции записи.
2. Создать backup и дождаться успешной проверки `pg_restore --list`.
3. Запустить `docker compose up --build`; `migrate` завершится до старта backend.
4. Проверить `/ready` и smoke-сценарий.
5. При несовместимой миграции остановить приложение, восстановить backup в проверенную базу и только затем вернуть старую версию приложения.

CI после backend integration tests создаёт настоящий `pg_dump`, восстанавливает его в чистую базу и повторно запускает schema/readiness tests против восстановленной копии.

## Чего эта конфигурация не предоставляет

- TLS/HTTPS и автоматические сертификаты.
- Docker healthcheck backend использует `/ready` с запросом к БД; `/health` остаётся независимой liveness-проверкой процесса.
- Автоматический rollback migrations: для необратимых изменений используется восстановление проверенного backup.
- Resource limits и внешнюю observability.

Перед публичным production-развёртыванием эти пункты необходимо решить в инфраструктуре.
