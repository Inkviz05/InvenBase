# Тестирование InvenBase

## Backend

### Library tests без PostgreSQL

```powershell
cd invenbase-server
cargo test --locked --lib
```

Проверяются JWT/auth helpers, модели и часть правил бронирования. Эту команду выполняет GitHub Actions quality job; полный набор отдельно запускается в backend integration job.

### Полный набор с PostgreSQL

Integration tests находятся в:

- `invenbase-server/tests/booking_lifecycle.rs`;
- `invenbase-server/tests/database_schema.rs`;
- `invenbase-server/tests/push_device_api.rs`.

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

DB tests проверяют migrations, ключевые таблицы/индексы/constraints/foreign keys и lifecycle бронирования, включая approve, reject, cancel, awaiting return, return, нехватку остатка и недоступное оборудование. HTTP-тесты проверяют успешный и неуспешный login, регистрацию и обновление устройства, несколько устройств одной учётной записи, перенос push-токена при смене пользователя, деактивацию текущего устройства и сценарий login → два устройства → notification/outbox для обоих устройств.

Остальные HTTP routes и middleware через `actix_web::test` пока не покрыты.

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

Зависимости JUnit/Espresso подключены, но тестовых Java/Kotlin исходников в репозитории нет. CI собирает debug APK на JDK 21 и публикует его как artifact.

## Что реально делает CI

Workflow `.github/workflows/docker-smoke.yml` запускается для push в `main`/`master` и pull request.

Quality job:

- `npm ci`;
- `npm audit --audit-level=high`;
- `npm run test:run` и `npm run build`;
- `cargo fmt --check` и `cargo test --locked --lib`;
- Android `:app:assembleDebug` на JDK 21 с публикацией debug APK;
- запрет `console.log` в web source и `show_files_listing` в backend source.

Compose smoke job:

- устанавливает Chromium;
- собирает полный Compose stack;
- ждёт `/ready` через nginx, включая проверку PostgreSQL;
- выполняет login → создание equipment → создание booking;
- на failure загружает Playwright artifacts.

Backend integration job поднимает отдельный чистый PostgreSQL 16, задаёт `TEST_DATABASE_URL`, применяет migrations и выполняет полный `cargo test --locked`, включая тесты из `invenbase-server/tests/`.

После тестов CI создаёт custom-format backup через `pg_dump`, проверяет его, восстанавливает в новую базу `invenbase_restore_ci` и повторно запускает schema/readiness tests против восстановленной копии.

CI не выполняет Android unit/instrumentation tests, lint или secret scan.
