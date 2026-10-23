# Docker Compose deployment

Основной `docker-compose.yml` поднимает:

- `postgres:16-alpine` с named volume `postgres_data`;
- multi-stage release image Rust backend;
- production React build, который отдаёт nginx.

Публичен только web/nginx на `WEB_PORT` (default `80`). nginx проксирует `/api` и `/health` в backend. Backend и PostgreSQL host-порты не публикуются.

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
- health: `/health` на том же origin.

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

Удаление БД вместе с volume — необратимая для этого volume операция:

```powershell
docker compose down -v
```

## Конфигурация

Пул backend настраивается через `DATABASE_MAX_CONNECTIONS` и `DATABASE_ACQUIRE_TIMEOUT_SECONDS`; defaults — `10` и `5` секунд.

Web image собирается с `VITE_API_URL=/api`. nginx имеет SPA fallback на `index.html`, прокси API и healthcheck; directory listing backend отключён.

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

## Чего эта конфигурация не предоставляет

- TLS/HTTPS и автоматические сертификаты.
- Readiness check с запросом к БД; используется статический `/health`.
- Backup/restore automation.
- Отдельный migrate/rollback job.
- Resource limits и внешнюю observability.

Перед публичным production-развёртыванием эти пункты необходимо решить в инфраструктуре.
