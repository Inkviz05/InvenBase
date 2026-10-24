# Push-уведомления: фактическая серверная реализация

## Поток данных

1. Android получает token от RuStore или Huawei.
2. После login `PushManager` отправляет token, стабильный `device_id`, provider и platform `android` на `POST /api/push/register-token` с Bearer JWT.
3. Backend сохраняет устройство независимо от наличия credentials транспорта; повторная регистрация обновляет token, `active` и `last_seen_at`.
4. In-app notification и outbox-задание создаются одной транзакцией; worker доставляет событие на все активные RuStore/Huawei-устройства.
5. Payload содержит `notification_id`, а полное уведомление доступно через `/api/notifications`.
6. WorkManager каждые 15 минут сверяет непрочитанные уведомления и показывает пропущенные сообщения. RuStore `onDeletedMessages` запускает немедленную сверку.

Firebase/FCM из Android и backend удалён. Новая миграция удаляет прежние FCM device tokens и ограничивает `user_devices.provider` значениями `rustore` и `huawei`.

## RuStore transport

```env
RUSTORE_PROJECT_ID=your-rustore-project-id
RUSTORE_SERVICE_TOKEN=your-service-token
```

Backend отправляет tokens provider `rustore` через `https://vkpns.rustore.ru/v1/projects/{project_id}/messages:send`. Ответ `NOT_FOUND` деактивирует token. Android использует RuStore Push SDK 7.5.0, получает token при старте и через `onNewToken`, сохраняет его до авторизации и регистрирует с provider `rustore`. Для инициализации задайте Gradle property или переменную окружения `RUSTORE_PROJECT_ID`; при пустом значении интеграция отключена.

## Huawei transport

```env
HUAWEI_CLIENT_ID=your-client-id
HUAWEI_CLIENT_SECRET=your-client-secret
```

Backend получает OAuth token AppGallery Connect и отправляет сообщения через Huawei Push Kit. Provider code `80300007` деактивирует недействительный token. Android включает AG Connect только при наличии локального `app/agconnect-services.json`.

## Реальные маршруты

- `POST /api/push/register-token` — регистрация token текущего пользователя; поле `provider` обязательно и принимает `rustore` или `huawei`.
- `POST /api/push/unregister-device` — деактивация конкретного provider/device при logout.
- `GET /api/notifications` — in-app уведомления.
- `GET /api/notifications/count` и `/unread-count` — число непрочитанных.
- `POST /api/notifications/{id}/read` и `/read-all` — отметка прочитанного.

Публичного API для произвольной отправки push в проекте нет.

## Ограничения

- RuStore требует отдельный проект для каждой комбинации package name и подписи.
- Получение tokens и доставка RuStore/Huawei на реальных зарегистрированных устройствах пока отложены.
- Отдельных network contract tests provider API пока нет.
- WorkManager использует минимальный системный интервал 15 минут и не является каналом мгновенной доставки.
- Provider credentials должны храниться в secrets deployment-окружения.
