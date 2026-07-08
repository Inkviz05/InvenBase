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

Сейчас у web-клиента нет отдельной тестовой инфраструктуры. Следующий практичный шаг:

- добавить Vitest и React Testing Library;
- покрыть API-клиент, авторизацию, выбор темы и основные формы;
- затем добавить e2e smoke-тесты для входа, списка оборудования и создания бронирования.
