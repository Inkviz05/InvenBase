# Реальные browser smoke tests

## Docker Compose stack

Этот сценарий ближе всего к deployment: PostgreSQL 16, release backend image и production React build за nginx.

```powershell
cd invenbase-client\web
npm ci
npx playwright install chromium
cd ..\..
.\scripts\test-docker-compose-smoke.ps1
```

Скрипт использует отдельный Compose project, временные credentials и named volume. Он ждёт публичный `/health`, затем запускает `tests/e2e/real-smoke.spec.js` через nginx. Проверяются login, создание squad/equipment через API, отображение equipment в UI и создание/появление booking.

Параметры:

```powershell
.\scripts\test-docker-compose-smoke.ps1 -SkipBuild
.\scripts\test-docker-compose-smoke.ps1 -KeepDocker
$env:SMOKE_WEB_PORT = "3100"
```

Без `-KeepDocker` containers, network и volume удаляются в `finally`.

## Development stack

```powershell
.\scripts\test-web-real-smoke.ps1
```

Скрипт:

- поднимает `postgres:16-alpine` из `docker-compose.smoke.yml` на host-порту `55432`;
- запускает backend через `cargo run` на `127.0.0.1:18080`;
- запускает Vite на `127.0.0.1:3100`;
- выполняет тот же Playwright real smoke.

Overrides:

```powershell
$env:SMOKE_API_PORT = "18080"
$env:SMOKE_WEB_PORT = "3100"
$env:SMOKE_DATABASE_URL = "postgresql://postgres:postgres@127.0.0.1:55432/kvantoriym_smoke"
$env:REAL_SMOKE_ADMIN_USERNAME = "admin"
$env:REAL_SMOKE_ADMIN_PASSWORD = "smoke-admin-password"
$env:REAL_SMOKE_JWT_SECRET = "real-smoke-jwt-secret-minimum-32-chars"
```

С уже существующей PostgreSQL:

```powershell
$env:SMOKE_DATABASE_URL = "postgresql://postgres:password@127.0.0.1:5432/kvantoriym_test"
.\scripts\test-web-real-smoke.ps1 -SkipDocker
```

Не указывайте production database: smoke создаёт реальные тестовые записи.
