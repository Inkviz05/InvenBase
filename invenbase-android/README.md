# InvenBase Android

Native Android client for InvenBase. This is the active mobile client; the old WebView Android client under `invenbase-client/android` has been removed from the active architecture.

## Stack

- Java
- Android Gradle Plugin
- Gradle Wrapper
- Retrofit + Gson
- AndroidX + Material Components
- ZXing QR scanner
- Firebase Cloud Messaging

## Requirements

- Android SDK with `compileSdk 34`
- JDK 17 or JDK 21
- Avoid running this project with JDK 25: Gradle/Android tooling can fail during script compilation.

On this workstation the successful build command used JetBrains Runtime 21:

```powershell
$env:JAVA_HOME='C:\Program Files\JetBrains\PyCharm 2025.2.3\jbr'
$env:Path="$env:JAVA_HOME\bin;$env:Path"
.\gradlew.bat assembleDebug
```

## API URL

Configure the backend URL in `app/src/main/java/com/invenbase/app/utils/Config.java`.

- Android emulator: `http://10.0.2.2:8080/api`
- Physical device: `http://YOUR_LAN_IP:8080/api`

The Rust backend must allow the web/mobile origin in `CORS_ALLOWED_ORIGINS` when requests come from a browser context. Native Android Retrofit requests are not restricted by browser CORS.

## Build

```powershell
.\gradlew.bat assembleDebug
```

Debug APK output:

```text
app/build/outputs/apk/debug/app-debug.apk
```

Build outputs are intentionally ignored by git.
