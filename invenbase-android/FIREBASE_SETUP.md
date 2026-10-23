# Firebase Cloud Messaging для Android

## Что уже реализовано

- В корневом `build.gradle` подключён Google Services plugin.
- В `app/build.gradle` подключены Firebase BOM, Messaging и Analytics.
- `InvenBaseApplication` подписывается на topic `all`.
- `MyFirebaseMessagingService` принимает сообщения и обновление токена.
- `PushManager` регистрирует token на backend после авторизации.

## Настройка Android

1. Создайте или выберите Firebase project.
2. Добавьте Android app с package name `com.invenbase.app`.
3. Скачайте `google-services.json`.
4. Поместите файл в `invenbase-android/app/google-services.json`.
5. Решите политику хранения явно: текущий корневой `.gitignore` не исключает `google-services.json`. Service account JSON и приватные ключи коммитить нельзя.
6. Выполните Gradle sync и соберите debug APK.

Регистрация токена требует авторизованного пользователя, потому что `POST /api/push/register-token` защищён Bearer JWT.

## Backend для отправки

Рекомендуемый и реализованный вариант — FCM HTTP v1:

```env
FCM_PROJECT_ID=your-firebase-project-id
FCM_SERVICE_ACCOUNT_PATH=C:\secure\firebase-service-account.json
```

Service account JSON храните вне git и вне Docker image. Backend читает файл, создаёт OAuth JWT, получает access token и вызывает `https://fcm.googleapis.com/v1/projects/{project}/messages:send`.

`FCM_SERVER_KEY` присутствует в коде только как legacy fallback. Legacy endpoint `fcm/send` отключён Google, поэтому на него нельзя рассчитывать как на рабочую production-конфигурацию.

Если FCM вообще не настроен, текущий backend не сохраняет token устройства и возвращает ошибку регистрации.

## Проверка

1. Авторизуйтесь в Android-приложении.
2. Убедитесь по backend-логу, что `POST /api/push/register-token` успешен.
3. Создайте событие, для которого backend вызывает push: изменение статуса бронирования или сообщение/изменение заявки поддержки.
4. Проверьте доставку на устройстве и запись in-app уведомления.

Отправка из Firebase Console в topic `all` проверяет только Firebase/Android, но не backend-регистрацию token и не серверную HTTP v1 реализацию.
