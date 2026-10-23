# Тестирование InvenBase

## Backend

### Library tests без PostgreSQL

```powershell
cd invenbase-server
cargo test --locked --lib
```

Проверяются JWT/auth helpers, модели и часть правил бронирования. Именно эту команду выполняет текущий GitHub Actions quality job.

### Полный набор с PostgreSQL

Integration tests находятся в:

- `invenbase-server/tests/booking_lifecycle.rs`;
- `invenbase-server/tests/database_schema.rs`.

Они требуют `TEST_DATABASE_URL`, применяют migrations и создают/удаляют тестовые записи:

```powershell
$env:TEST_DATABASE_URL = "postgresql://postgres:password@localhost:5432/kvantoriym_test"
cd invenbase-server
cargo test --locked
```

Либо из корня:

```powershell
.\scripts\test-backend.ps1
.\scripts\test-backend.ps1 -NoCapture
```

Если `TEST_DATABASE_URL` не задана, скрипт берёт `DATABASE_URL` из `invenbase-server/.env`. Это допускается кодом, но небезопасно для важных локальных данных; предпочтительна отдельная тестовая БД.

DB tests проверяют migrations, ключевые таблицы/индексы/constraints/foreign keys и lifecycle бронирования, включая approve, reject, cancel, awaiting return, return, нехватку остатка и недоступное оборудование.

HTTP routes и middleware через `actix_web::test` сейчас не покрыты.

## Web unit/component tests

```powershell
cd invenbase-client\web
npm ci
npm run test:run
```

Watch mode:

```powershell
npm run test
```

Покрыты config, API client, AuthContext, login, layout/theme, equipment list, booking create/list и XLSX writer. Reports, support и полный набор booking actions отдельными component tests не покрыты.

## Web Playwright

Mock smoke:

```powershell
cd invenbase-client\web
npx playwright install chromium
npm run test:e2e:smoke
```

Real backend/PostgreSQL и Docker Compose сценарии описаны в [REAL_SMOKE_TESTING.md](REAL_SMOKE_TESTING.md).

## Dependency и build checks

```powershell
cd invenbase-client\web
npm audit --audit-level=high
npm run build

cd ..\..\invenbase-server
cargo fmt --check
cargo build --release --locked
```

Отдельных ESLint/Prettier scripts в web нет.

## Android

```powershell
cd invenbase-android
.\gradlew.bat assembleDebug
```

Зависимости JUnit/Espresso подключены, но тестовых Java/Kotlin исходников в репозитории нет. CI Android не собирает.

## Что реально делает CI

Workflow `.github/workflows/docker-smoke.yml` запускается для push в `main`/`master` и pull request.

Quality job:

- `npm ci`;
- `npm audit --audit-level=high`;
- `npm run test:run` и `npm run build`;
- `cargo fmt --check` и `cargo test --locked --lib`;
- запрет `console.log` в web source и `show_files_listing` в backend source.

Compose smoke job:

- устанавливает Chromium;
- собирает полный Compose stack;
- ждёт `/health` через nginx;
- выполняет login → создание equipment → создание booking;
- на failure загружает Playwright artifacts.

CI не выполняет полный backend DB test suite, Android build, lint или secret scan.
