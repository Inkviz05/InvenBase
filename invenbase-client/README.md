# InvenBase Web Client

React/Vite web-клиент для системы учета и бронирования оборудования.

В этой директории больше нет Android WebView-проекта. Android-приложение поддерживается отдельно в `../invenbase-android`.

## Структура

```text
web/
  src/
    api/          API-клиенты
    components/   Общие компоненты
    context/      React context
    pages/        Страницы приложения
    App.jsx
    main.jsx
    config.js
  public/
  package.json
  vite.config.js
```

## Запуск

```bash
cd invenbase-client/web
cp .env.example .env
npm install
npm run dev
```

Dev-сервер доступен на `http://localhost:3000`.

## Конфигурация

`web/.env`:

```env
VITE_API_URL=http://localhost:8080/api
VITE_API_PORT=8080
```

В режиме разработки запросы к `/api` проксируются Vite на backend. В production используется `VITE_API_URL`, а если он не задан, клиент пытается обратиться к API на том же host и порту `VITE_API_PORT`.

## Сборка

```bash
cd invenbase-client/web
npm run build
npm run preview
```

Результат сборки появляется в `web/dist/`.

## Основные модули

- `src/api` - axios-клиенты для backend endpoints.
- `src/context/AuthContext.jsx` - авторизация и текущий пользователь.
- `src/context/CartContext.jsx` - корзина бронирований.
- `src/pages` - страницы оборудования, бронирований, отчетов, уведомлений, пользователей и поддержки.

## Android

WebView APK больше не собирается из web-клиента. Для Android используется native Java-приложение:

```bash
cd ../invenbase-android
./gradlew assembleDebug
```
