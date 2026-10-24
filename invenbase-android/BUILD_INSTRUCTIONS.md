# Сборка InvenBase Android

## Требования

- Android SDK 34.
- JDK 17 или 21.
- Интернет при первой Gradle sync для загрузки зависимостей.
- `app/google-services.json` от Firebase-проекта `com.invenbase.app`, только если нужен FCM; файл хранится локально и исключён из Git.
- ID проекта Push из RuStore Console, если требуется RuStore transport.

JDK 25 для текущего Gradle/Android toolchain не рекомендуется.

Без `app/google-services.json` debug/release варианты собираются с отключённым FCM. Положите собственный файл в `app/`, чтобы Google Services plugin и FCM включились автоматически.

RuStore project ID передаётся без сохранения в git:

```powershell
$env:RUSTORE_PROJECT_ID="your-rustore-project-id"
.\gradlew.bat assembleDebug
```

Либо задайте `RUSTORE_PROJECT_ID` в пользовательском `~/.gradle/gradle.properties`. При пустом значении RuStore SDK не инициализируется, FCM продолжает работать.

## Debug APK

```powershell
cd D:\InvenBase\invenbase-android
.\gradlew.bat assembleDebug
```

Результат:

```text
app/build/outputs/apk/debug/app-debug.apk
```

Установка на подключённое устройство:

```powershell
.\gradlew.bat installDebug
```

## Release variant

```powershell
.\gradlew.bat assembleRelease
```

В репозитории не настроены signing credentials. `release` также собирается с `minifyEnabled false`; это не готовая инструкция публикации в магазин.

## Backend

Текущий default base URL — `https://invenbase.inkviz05.online`, задаётся в `app/src/main/java/com/invenbase/app/Config.java`. Retrofit добавляет пути `/api/...` сам.

Manifest запрещает cleartext HTTP. Для локального backend потребуется либо HTTPS, либо отдельная debug network-security конфигурация, которой сейчас в репозитории нет.
