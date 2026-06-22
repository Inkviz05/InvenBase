package com.invenbase.app.utils;

import android.content.Context;
import android.content.SharedPreferences;
import android.util.Log;

import com.invenbase.app.Config;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;

import java.util.HashMap;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

/**
 * Отвечает за регистрацию FCM-токена на backend-е.
 * Ожидается, что сервер реализует POST /api/push/register-token
 * и свяжет токен с текущим пользователем по токену авторизации.
 */
public class PushManager {

    private static final String PREF_FCM_TOKEN = "fcm_token";

    private static PushManager instance;
    private final SharedPreferences prefs;
    private final ApiService apiService;
    private final AuthManager authManager;

    // Конструктор PushManager: инициализирует объект и его зависимости.
    private PushManager(Context context) {
        Context appContext = context.getApplicationContext();
        this.prefs = appContext.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
        this.apiService = ApiClient.getInstance(appContext).getApiService();
        this.authManager = AuthManager.getInstance(appContext);
    }

    // Метод getInstance: возвращает нужное значение для текущего контекста.
    public static synchronized PushManager getInstance(Context context) {
        if (instance == null) {
            instance = new PushManager(context);
        }
        return instance;
    }

    /**
     * Сохраняем токен локально и пытаемся отправить его на сервер.
     * Если пользователь еще не авторизован, токен будет отправлен при следующем успешном входе.
     */
    public void registerToken(String token) {
        if (token == null || token.isEmpty()) {
            return;
        }
        Log.d("FCM", "PushManager.registerToken: " + token);
        prefs.edit().putString(PREF_FCM_TOKEN, token).apply();

        if (!authManager.isAuthenticated()) {
            // Пользователь пока не залогинен — отправим позже.
            Log.d("FCM", "User not authenticated yet, will send token after login");
            return;
        }

        sendTokenToServer(token);
    }

    /**
     * Вызывается после успешного логина, чтобы убедиться, что токен отправлен на сервер.
     */
    public void sendStoredTokenIfNeeded() {
        String token = prefs.getString(PREF_FCM_TOKEN, null);
        if (token != null && authManager.isAuthenticated()) {
            sendTokenToServer(token);
        }
    }

    // Метод sendTokenToServer: выполняет основную бизнес- или UI-логику данного участка кода.
    private void sendTokenToServer(String token) {
        Map<String, String> body = new HashMap<>();
        body.put("token", token);
        body.put("platform", "android");

        Log.d("FCM", "Sending push token to server: " + token.substring(0, Math.min(20, token.length())) + "...");
        Log.d("FCM", "API endpoint: POST /api/push/register-token");
        
        apiService.registerPushToken(body).enqueue(new Callback<Void>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Void> call, Response<Void> response) {
                if (response.isSuccessful()) {
                    Log.d("FCM", "Push token registered successfully on server (status: " + response.code() + ")");
                } else {
                    Log.e("FCM", "Failed to register push token. Code=" + response.code());
                    if (response.errorBody() != null) {
                        try {
                            String errorBody = response.errorBody().string();
                            Log.e("FCM", "Error response: " + errorBody);
                        } catch (Exception e) {
                            Log.e("FCM", "Could not read error body", e);
                        }
                    }
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Void> call, Throwable t) {
                Log.e("FCM", "Error registering push token: " + t.getMessage(), t);
                Log.e("FCM", "Request URL: " + (call.request() != null ? call.request().url() : "null"));
            }
        });
    }
}

