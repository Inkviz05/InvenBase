# InvenBase Android

Активный native Android-клиент на Java.

## Технологии и параметры сборки

- Android Gradle Plugin 8.1.0, Gradle Wrapper 8.7.
- `compileSdk 34`, `targetSdk 34`, `minSdk 24`.
- Java source/target compatibility 8; для запуска Gradle рекомендуется JDK 17 или 21.
- Retrofit/Gson/OkHttp, Material/AndroidX, ZXing, Glide и Firebase Messaging.
- Package/application ID: `com.invenbase.app`.

## Сборка

```powershell
.\gradlew.bat assembleDebug
```

APK: `app/build/outputs/apk/debug/app-debug.apk`.

Release variant существует, но signing config в проекте не задан, а `minifyEnabled=false`. Готовой production release-процедуры документация не заявляет.

## API URL

Константа находится в `app/src/main/java/com/invenbase/app/Config.java`:

```java
public static final String DEFAULT_API_URL = "https://invenbase.inkviz05.online";
```

`ApiClient` читает переопределение из `SharedPreferences` по ключу `api_url`. Retrofit-методы содержат абсолютные пути `/api/...`, поэтому base URL указывается без `/api`.

В manifest задано `android:usesCleartextTraffic="false"`. Поэтому обычный `http://10.0.2.2:8080` или LAN HTTP URL не заработает без изменения network security для debug-сборки. Для текущей конфигурации нужен HTTPS endpoint.

## Реализованные экраны

- Login и основной экран с fragments dashboard, equipment, bookings, scanner и admin.
- Equipment detail/create/edit и перемещения.
- Корзина и одиночное/пакетное бронирование.
- Categories, users, squads, reports и logs.
- Notifications, support и squad detail.

Доступность функций дополнительно ограничивается backend-ролями `admin`, `responsible`, `user`.

## Авторизация и локальные данные

JWT, сериализованный пользователь и API URL сохраняются в обычных `SharedPreferences`. `EncryptedSharedPreferences` в проекте не используется — это известное ограничение безопасности.

## Push

Приложение:

- подключает Firebase Messaging;
- подписывается на topic `all` при старте;
- обрабатывает `onNewToken` и `onMessageReceived`;
- после авторизации регистрирует token через `POST /api/push/register-token`.

Для сборки Firebase нужен `app/google-services.json`; он не должен попадать в git. Подробности: [FIREBASE_SETUP.md](FIREBASE_SETUP.md) и [PUSH_NOTIFICATIONS_SERVER.md](PUSH_NOTIFICATIONS_SERVER.md).

## Тесты и ограничения

Gradle содержит зависимости JUnit и Espresso, но каталогов с test/androidTest исходниками в репозитории сейчас нет. Android build также не включён в GitHub Actions.

Архитектура в основном Activity/Fragment + прямые Retrofit-вызовы; полноценного repository/ViewModel слоя нет.
