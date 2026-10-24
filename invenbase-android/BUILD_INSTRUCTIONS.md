# Сборка InvenBase Android

## Требования

- Android SDK 34.
- JDK 17 или 21.
- Интернет при первой Gradle sync для загрузки зависимостей.
- ID проекта Push из RuStore Console, если требуется RuStore transport.

JDK 25 для текущего Gradle/Android toolchain не рекомендуется.

RuStore project ID передаётся без сохранения в git:

```powershell
$env:RUSTORE_PROJECT_ID="your-rustore-project-id"
.\gradlew.bat assembleDebug
```

Либо задайте `RUSTORE_PROJECT_ID` в пользовательском `~/.gradle/gradle.properties`. При пустом значении RuStore SDK не инициализируется; Huawei и WorkManager-синхронизация настраиваются независимо.

API endpoint также задаётся на этапе сборки:

```powershell
$env:DEBUG_API_URL="http://10.0.2.2:8080"
$env:RELEASE_API_URL="https://invenbase.example.com"
```

Defaults — `http://10.0.2.2:8080` для debug и `https://invenbase.inkviz05.online` для release. `RELEASE_API_URL` обязан использовать HTTPS. На чистой установке приложение всё равно запрашивает сервер и одноразовый setup-код; build-time URL используется как подсказка debug-сборки и для миграции уже авторизованных установок старой версии.

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
$env:ANDROID_KEYSTORE_PATH="D:\secure\invenbase-release.keystore"
$env:ANDROID_KEYSTORE_PASSWORD="your-store-password"
$env:ANDROID_KEY_ALIAS="invenbase"
$env:ANDROID_KEY_PASSWORD="your-key-password"
.\gradlew.bat assembleRelease bundleRelease
```

Signing включается, только если заданы все четыре переменные. При частичной конфигурации сборка завершается ошибкой; ключ и пароли не сохраняются в Git или Gradle-файлах. Release включает R8 (`minifyEnabled`) и удаление неиспользуемых ресурсов. Результаты:

```text
app/build/outputs/apk/release/app-release.apk
app/build/outputs/bundle/release/app-release.aab
```

Для CI используются secrets `ANDROID_KEYSTORE_BASE64`, `ANDROID_KEYSTORE_PASSWORD`, `ANDROID_KEY_ALIAS`, `ANDROID_KEY_PASSWORD`. При их наличии workflow декодирует keystore во временный каталог и публикует подписанные APK/AAB; без secrets release всё равно собирается как unsigned проверка R8. Keystore необходимо хранить в резервной копии: без прежнего ключа нельзя выпустить обновление уже установленного приложения.

## Backend

Retrofit добавляет пути `/api/...` сам. Debug разрешает HTTP для emulator/local backend; release запрещает cleartext.
