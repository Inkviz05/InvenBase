@echo off
REM Скрипт для сборки веб-приложения и копирования в Android проект (Windows)

echo Сборка веб-приложения...
cd web
call npm install
call npm run build

echo Копирование файлов в Android проект...
cd ..
if not exist "android\app\src\main\assets" mkdir "android\app\src\main\assets"
xcopy /E /I /Y "web\dist\*" "android\app\src\main\assets\"

echo Исправление HTML для Android WebView...
powershell -Command "(Get-Content 'android\app\src\main\assets\index.html') -replace 'type=\"module\" crossorigin', '' -replace 'crossorigin', '' | Set-Content 'android\app\src\main\assets\index.html'"
powershell -Command "(Get-Content 'android\app\src\main\assets\index.html') -replace '<base href=\"./\" />', '<base href=\"file:///android_asset/\" />' | Set-Content 'android\app\src\main\assets\index.html'"
REM Инжектируем скрипт для установки Android переменных перед другими скриптами
powershell -Command "$content = Get-Content 'android\app\src\main\assets\index.html' -Raw; $script = '<script>window.ANDROID_WEBVIEW = true; window.ANDROID_API_URL = \"http://10.0.2.2:8080/api\"; console.log(\"Android WebView variables set:\", window.ANDROID_API_URL);</script>'; $content = $content -replace '(<head>)', \"`$1`n    $script\"; Set-Content 'android\app\src\main\assets\index.html' -Value $content -NoNewline"

echo Готово! Теперь откройте Android Studio и соберите APK.
pause

