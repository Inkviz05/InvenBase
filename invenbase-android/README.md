# InvenBase Android

Активный native Android-клиент на Java.

## Технологии и параметры сборки

- Android Gradle Plugin 8.1.0, Gradle Wrapper 8.7.
- `compileSdk 34`, `targetSdk 34`, `minSdk 24`.
- Java source/target compatibility 8; для запуска Gradle рекомендуется JDK 17 или 21.
- Retrofit/Gson/OkHttp, Material/AndroidX, ZXing, Glide, RuStore Push SDK 7.5.0 и Huawei Push Kit.
- Package/application ID: `com.invenbase.app`.

## Сборка

```powershell
.\gradlew.bat assembleDebug
```

APK: `app/build/outputs/apk/debug/app-debug.apk`.

Release включает R8/minification и resource shrinking. Signing credentials читаются только из `ANDROID_KEYSTORE_*` env; без них CI собирает unsigned release для проверки, а при наличии GitHub secrets публикует подписанные APK/AAB. Инструкция находится в [BUILD_INSTRUCTIONS.md](BUILD_INSTRUCTIONS.md).

## API URL

URL задаётся на этапе сборки через `BuildConfig.API_BASE_URL`:

```text
debug:   http://10.0.2.2:8080
release: https://invenbase.inkviz05.online
```

Переопределение передаётся Gradle property или одноимённой переменной окружения `DEBUG_API_URL`/`RELEASE_API_URL`. Release URL обязан использовать HTTPS. Сохранённое прежними версиями пользовательское значение `api_url` удаляется и больше не влияет на endpoint. Retrofit-методы содержат абсолютные пути `/api/...`, поэтому base URL указывается без `/api`.

Debug manifest разрешает cleartext для локальной разработки; `10.0.2.2` направляет Android Emulator на localhost компьютера. Release manifest запрещает cleartext.

## Реализованные экраны

- Login и основной экран с fragments dashboard, equipment, bookings, scanner и admin.
- Equipment detail/create/edit и перемещения.
- Корзина и одиночное/пакетное бронирование.
- Categories, users, squads, reports и logs.
- Notifications, support и squad detail.

Доступность функций дополнительно ограничивается backend-ролями `admin`, `responsible`, `user`.

## Авторизация и локальные данные

JWT, сериализованный пользователь и push-токены шифруются AES-GCM; ключ создаётся и хранится в Android Keystore. При обновлении приложения прежние чувствительные значения однократно переносятся из обычных `SharedPreferences` и удаляются оттуда. Зашифрованный preference-файл исключён из Android backup. API URL в локальном хранилище не сохраняется.

## Push

Приложение:

- подключает RuStore Push SDK при заданном build-параметре `RUSTORE_PROJECT_ID`;
- подключает Huawei Push Kit при наличии `app/agconnect-services.json`;
- обрабатывает обновление RuStore/Huawei tokens и входящие сообщения;
- после авторизации регистрирует доступные tokens через `POST /api/push/register-token` с соответствующим provider;
- перед локальным logout вызывает `POST /api/push/unregister-device` для RuStore и Huawei, поэтому отключается только текущее устройство, а остальные устройства учётной записи остаются активными;
- планирует WorkManager-сверку непрочитанных уведомлений каждые 15 минут при наличии сети и немедленно после login/`onDeletedMessages`;
- хранит до 500 обработанных `notification_id`, чтобы не показывать повторно уже полученный push.

Firebase/FCM, Google Services plugin и `google-services.json` в проекте не используются. RuStore project ID задаётся через Gradle property или одноимённую переменную окружения `RUSTORE_PROJECT_ID`. Подробности: [BUILD_INSTRUCTIONS.md](BUILD_INSTRUCTIONS.md) и [PUSH_NOTIFICATIONS_SERVER.md](PUSH_NOTIFICATIONS_SERVER.md).

## Тесты и ограничения

Gradle содержит зависимости JUnit и Espresso, но каталогов с test/androidTest исходниками в репозитории сейчас нет. GitHub Actions всегда собирает debug и R8 release variants, публикуя debug APK; подписанные release APK/AAB публикуются только при наличии signing secrets.

Архитектура в основном Activity/Fragment + прямые Retrofit-вызовы; полноценного repository/ViewModel слоя нет.
## Huawei Push

Поместите конфигурацию своего Android-приложения из AppGallery Connect в
`app/agconnect-services.json`; файл исключён из Git. В консоли должны совпадать
package name и отпечаток сертификата подписи. При наличии файла сборка включает
AG Connect, приложение запрашивает HMS push token в фоновом потоке и регистрирует
его как `huawei`. Обновление токена обрабатывает `MyHuaweiMessagingService`,
logout деактивирует устройство на backend. Входящие data-сообщения запускают
синхронизацию непрочитанных уведомлений. Debug-сборка без конфигурационного файла
AppGallery Connect проверена; получение токена и доставка на реальном устройстве
остаются в отложенном ручном тесте.
