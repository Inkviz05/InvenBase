# InvenBase web

React 18 SPA на Vite. Навигация использует `HashRouter`, поэтому клиентские URL имеют вид `/#/equipment`.

## Запуск

Нужны Node.js и npm. CI использует Node.js 22 для quality job и Node.js 20 для Compose smoke.

```powershell
Copy-Item .env.example .env
npm ci
npm run dev
```

Vite слушает `0.0.0.0:3000`. В dev-режиме `/api` проксируется на `http://127.0.0.1:<VITE_API_PORT>`, по умолчанию на порт `8080`.

Переменные:

- `VITE_API_URL` — production base URL API. В dev код всегда использует `/api`; production fallback без этой переменной строится как `http://<hostname>:<VITE_API_PORT>/api`. Docker build явно задаёт `/api`.
- `VITE_API_PORT` — порт локального backend для Vite proxy, default `8080`.

В production Docker image собирается с `VITE_API_URL=/api`; nginx отдаёт SPA и проксирует `/api` и `/health`.

## Скрипты

```powershell
npm run dev
npm run build
npm run preview
npm run test
npm run test:run
npm run test:e2e
npm run test:e2e:real-smoke
npm run test:e2e:docker-smoke
```

Отдельного lint script в `package.json` нет.

## Реальные маршруты UI

- `/#/login`
- `/#/` — dashboard
- `/#/equipment`, `/equipment/create`, `/equipment/:id`, `/equipment/:id/edit`
- `/#/bookings`, `/bookings/create`, `/bookings/bulk`
- `/#/categories`, `/users`, `/reports`, `/logs`, `/notifications`
- `/#/qr-scanner`, `/support`, `/squads`, `/squads/:id`

`PrivateRoute` проверяет только наличие активной авторизации. Роль влияет на меню и действия, но отдельных route-level role guards нет; окончательные права проверяет backend.

## API и auth

- JWT хранится клиентом в `localStorage` и добавляется как Bearer token.
- При `401` API client очищает локальное auth-состояние, кроме запроса login.
- Отдельного backend logout endpoint нет.
- API-модули покрывают auth, equipment, categories, bookings, users, reports, logs, notifications, QR, support и squads.

## Тесты

Vitest/React Testing Library сейчас покрывают:

- `config.js`;
- API client;
- `AuthContext`;
- login;
- layout и тему;
- equipment list;
- booking create/list;
- локальный XLSX writer.

Playwright содержит:

- mock smoke test;
- real smoke с настоящими backend/PostgreSQL;
- Docker Compose smoke через nginx: login, создание оборудования и бронирования.

Полные команды и требования: `../../TESTING.md` и `../../REAL_SMOKE_TESTING.md`.

## Подтверждённые ограничения

- OpenAPI-generated client отсутствует; endpoints описаны вручную в `src/api`.
- Route-level role guards отсутствуют.
- Не все страницы покрыты unit tests; нет полного e2e lifecycle approve/reject/cancel/return.
- TypeScript и ESLint в проекте не настроены.
