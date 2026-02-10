package com.kvantoriym.client;

import android.annotation.SuppressLint;
import android.os.Build;
import android.os.Bundle;
import android.util.Log;
import android.view.View;
import android.view.Window;
import android.view.WindowManager;
import android.webkit.ConsoleMessage;
import android.webkit.WebChromeClient;
import android.webkit.WebResourceError;
import android.webkit.WebResourceRequest;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

public class MainActivity extends AppCompatActivity {
    private static final String TAG = "MainActivity";
    private WebView webView;

    @SuppressLint("SetJavaScriptEnabled")
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        
        // Убираем системные панели (status bar) - делаем прозрачным
        Window window = getWindow();
        WindowCompat.setDecorFitsSystemWindows(window, false);
        
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
            WindowInsetsControllerCompat controller = WindowCompat.getInsetsController(window, window.getDecorView());
            if (controller != null) {
                controller.hide(WindowInsetsCompat.Type.statusBars());
            }
        } else if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.LOLLIPOP) {
            window.addFlags(WindowManager.LayoutParams.FLAG_DRAWS_SYSTEM_BAR_BACKGROUNDS);
            window.setStatusBarColor(android.graphics.Color.TRANSPARENT);
            window.getDecorView().setSystemUiVisibility(
                View.SYSTEM_UI_FLAG_LAYOUT_STABLE | View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN
            );
        }
        
        setContentView(R.layout.activity_main);

        webView = findViewById(R.id.webview);
        
        WebSettings webSettings = webView.getSettings();
        webSettings.setJavaScriptEnabled(true);
        webSettings.setDomStorageEnabled(true);
        webSettings.setDatabaseEnabled(true);
        webSettings.setMediaPlaybackRequiresUserGesture(false);
        webSettings.setAllowFileAccess(true);
        webSettings.setAllowContentAccess(true);
        webSettings.setAllowFileAccessFromFileURLs(true);
        webSettings.setAllowUniversalAccessFromFileURLs(true);
        webSettings.setGeolocationEnabled(true);
        webSettings.setCacheMode(WebSettings.LOAD_DEFAULT);
        
        // Включаем поддержку камеры для QR-сканера
        webSettings.setMixedContentMode(WebSettings.MIXED_CONTENT_ALWAYS_ALLOW);
        
        // Добавляем WebChromeClient для отладки и обработки консольных сообщений
        webView.setWebChromeClient(new WebChromeClient() {
            @Override
            public boolean onConsoleMessage(ConsoleMessage consoleMessage) {
                Log.d(TAG, "Console: " + consoleMessage.message() + " -- From line "
                        + consoleMessage.lineNumber() + " of "
                        + consoleMessage.sourceId());
                return true;
            }
        });
        
        webView.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView view, WebResourceRequest request) {
                String url = request.getUrl().toString();
                Log.d(TAG, "shouldOverrideUrlLoading: " + url);
                return false; // Позволяем WebView обрабатывать URL самостоятельно
            }
            
            @Override
            public android.webkit.WebResourceResponse shouldInterceptRequest(WebView view, 
                    WebResourceRequest request) {
                String url = request.getUrl().toString();
                Log.d(TAG, "Requesting resource: " + url);
                return super.shouldInterceptRequest(view, request);
            }
            
            @Override
            public void onPageStarted(WebView view, String url, android.graphics.Bitmap favicon) {
                super.onPageStarted(view, url, favicon);
                Log.d(TAG, "Page started loading: " + url);
                
                // Инжектируем переменные ДО загрузки скриптов
                // Используем 10.0.2.2 для эмулятора (это специальный IP для доступа к localhost хоста)
                view.evaluateJavascript(
                    "(function() {" +
                    "  window.ANDROID_WEBVIEW = true;" +
                    "  window.ANDROID_API_URL = 'http://10.0.2.2:8080/api';" +
                    "  console.log('Android WebView variables injected (early)');" +
                    "  console.log('API URL set to:', window.ANDROID_API_URL);" +
                    "})();",
                    null
                );
            }
            
            @Override
            public void onPageFinished(WebView view, String url) {
                super.onPageFinished(view, url);
                Log.d(TAG, "Page finished loading: " + url);
                
                // Убеждаемся, что переменные установлены (на случай, если onPageStarted не сработал)
                view.evaluateJavascript(
                    "(function() {" +
                    "  if (!window.ANDROID_API_URL) {" +
                    "    window.ANDROID_WEBVIEW = true;" +
                    "    window.ANDROID_API_URL = 'http://10.0.2.2:8080/api';" +
                    "    console.log('Android WebView variables injected (late)');" +
                    "    console.log('API URL set to:', window.ANDROID_API_URL);" +
                    "  }" +
                    "  return 'Android variables check complete';" +
                    "})();",
                    null
                );
                
                // Проверяем, загрузились ли скрипты и инициализировалось ли приложение
                view.evaluateJavascript(
                    "(function() {" +
                    "  console.log('=== Debug Info ===');" +
                    "  console.log('Root element:', document.getElementById('root'));" +
                    "  console.log('Scripts loaded:', document.querySelectorAll('script').length);" +
                    "  console.log('All scripts:', Array.from(document.querySelectorAll('script')).map(s => s.src || 'inline').join(', '));" +
                    "  var errors = [];" +
                    "  window.addEventListener('error', function(e) { errors.push(e.message); });" +
                    "  setTimeout(function() {" +
                    "    console.log('Errors after 2s:', errors);" +
                    "    console.log('Root content:', document.getElementById('root').innerHTML);" +
                    "  }, 2000);" +
                    "  return 'Debug script injected';" +
                    "})();",
                    new android.webkit.ValueCallback<String>() {
                        @Override
                        public void onReceiveValue(String value) {
                            Log.d(TAG, "Debug script result: " + value);
                        }
                    }
                );
            }
            
            @Override
            public void onReceivedError(WebView view, WebResourceRequest request, WebResourceError error) {
                super.onReceivedError(view, request, error);
                String url = request.getUrl().toString();
                String errorDesc = error.getDescription().toString();
                int errorCode = error.getErrorCode();
                Log.e(TAG, String.format("Error loading resource [%d]: %s - URL: %s", 
                    errorCode, errorDesc, url));
                
                // Если это критическая ошибка загрузки главной страницы, попробуем dev-сервер
                if (url.contains("index.html") && errorCode == -2) {
                    Log.w(TAG, "Failed to load local file, trying dev server...");
                    // Можно попробовать загрузить с dev-сервера
                    // view.loadUrl("http://10.0.2.2:5173");
                }
            }
            
            @Override
            public void onReceivedHttpError(WebView view, WebResourceRequest request, 
                    android.webkit.WebResourceResponse errorResponse) {
                super.onReceivedHttpError(view, request, errorResponse);
                Log.e(TAG, "HTTP Error loading: " + request.getUrl() + 
                    " - Status: " + errorResponse.getStatusCode());
            }
        });

        // Загружаем локальный HTML файл из assets
        // Для разработки можно использовать: webView.loadUrl("http://10.0.2.2:5173");
        // Для продакшена используйте локальные файлы:
        String url = "file:///android_asset/index.html";
        Log.d(TAG, "Loading URL: " + url);
        webView.loadUrl(url);
    }

    @Override
    public void onBackPressed() {
        if (webView.canGoBack()) {
            webView.goBack();
        } else {
            super.onBackPressed();
        }
    }
}

