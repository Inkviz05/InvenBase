# Тестирование проекта

## Backend

Основная команда для локального запуска backend-тестов из корня проекта:

```powershell
.\scripts\test-backend.ps1
```

Скрипт делает следующее:

- читает `DATABASE_URL` из `invenbase-server/.env`;
- если `TEST_DATABASE_URL` не задан, подставляет в него `DATABASE_URL`;
- запускает `cargo test` в `invenbase-server`;
- прогоняет unit-тесты, интеграционные тесты логики бронирований и тесты схемы PostgreSQL.

Сейчас можно использовать основную локальную базу, потому что production-окружения ещё нет. Важно учитывать, что DB-тесты применяют SQLx migrations и создают временные тестовые записи.

Если нужна отдельная база для тестов, задайте `TEST_DATABASE_URL` перед запуском:

```powershell
$env:TEST_DATABASE_URL = "postgresql://postgres:postgres@localhost:5432/kvantoriym_test"
.\scripts\test-backend.ps1
```

Для вывода `println!`/`eprintln!` из Rust-тестов:

```powershell
.\scripts\test-backend.ps1 -NoCapture
```

## Что проверяют DB-тесты

DB-тесты находятся в `invenbase-server/tests/database_schema.rs`.

Они проверяют:

- создание ключевых таблиц миграциями;
- наличие критичных индексов;
- доменные `CHECK` constraints для ролей, статусов и бизнес-состояний;
- foreign key constraints между основными сущностями;
- отказ БД принимать некорректные значения;
- запрет бронирования без оборудования или группы оборудования.

## Попадают ли тесты в production

Нет. Rust-тесты не входят в обычный production build:

- unit-тесты под `#[cfg(test)]` компилируются только для `cargo test`;
- интеграционные тесты из `invenbase-server/tests/` запускаются только через `cargo test`;
- `cargo build --release` собирает production-бинарник без test harness.

Для Docker/CI правильная схема такая:

1. В build/CI-этапе поднять PostgreSQL.
2. Запустить `.\scripts\test-backend.ps1` или аналогичную Linux-команду.
3. Если тесты прошли, собрать `cargo build --release`.
4. В runtime-образ копировать только release-бинарник и необходимые runtime-файлы.

## Web

У web-клиента подключены Vitest, React Testing Library и jsdom.

Запуск тестов:

```powershell
cd invenbase-client\web
npm run test:run
```

Интерактивный watch-режим:

```powershell
cd invenbase-client\web
npm run test
```

Что уже покрыто:

- `src/config.js`: базовый API URL в dev/test-режиме и доменные константы;
- `src/api/client.js`: base URL, bearer token, исключение для `/auth/login`, очистка auth-состояния на `401`;
- `src/components/Layout.jsx`: тема по умолчанию, восстановление сохранённой темы, переключение на `neon-purple` через меню пользователя.

Следующий практичный шаг:

- покрыть `AuthContext` и формы логина;
- добавить тесты для списка оборудования и формы создания бронирования;
- затем добавить e2e smoke-тесты для входа, списка оборудования и создания бронирования.

## Web dependency audit

Команда для проверки зависимостей:

```powershell
cd invenbase-client\web
npm audit
```

После безопасного `npm audit fix` без `--force` и замены SheetJS:

- `xlsx` удалён из зависимостей;
- экспорт `.xlsx` сохранён через локальный writer `src/utils/xlsxExport.js`;
- `npm audit` должен возвращать `found 0 vulnerabilities`.

Практичный следующий шаг: добавить browser smoke-тест на скачивание отчётов, чтобы проверить кнопку экспорта в реальном UI.
