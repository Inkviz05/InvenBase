package com.invenbase.app.utils;

import android.content.Context;
import android.util.Log;

import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;

import java.util.HashMap;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

/**
 * Отвечает за регистрацию push-токенов разных providers на backend-е.
 * Ожидается, что сервер реализует POST /api/push/register-token
 * и свяжет токен с текущим пользователем по токену авторизации.
 */
public class PushManager {

    public static final String PROVIDER_RUSTORE = "rustore";
    public static final String PROVIDER_HUAWEI = "huawei";
    private static final String PREF_TOKEN_PREFIX = "push_token_";

    private static PushManager instance;
    private final SecurePreferences securePrefs;
    private final ApiService apiService;
    private final AuthManager authManager;
    private final String deviceId;

    // Конструктор PushManager: инициализирует объект и его зависимости.
    private PushManager(Context context) {
        Context appContext = context.getApplicationContext();
        this.securePrefs = new SecurePreferences(appContext);
        this.apiService = ApiClient.getInstance(appContext).getApiService();
        this.authManager = AuthManager.getInstance(appContext);
        this.deviceId = DeviceIdentity.get(appContext);
        migrateLegacyPushToken(appContext, PROVIDER_RUSTORE);
        migrateLegacyPushToken(appContext, PROVIDER_HUAWEI);
        appContext.getSharedPreferences(com.invenbase.app.Config.PREF_NAME, Context.MODE_PRIVATE)
                .edit()
                .remove("fcm_token")
                .remove(PREF_TOKEN_PREFIX + "fcm")
                .commit();
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
    public void registerToken(String provider, String token) {
        if (token == null || token.isEmpty()) {
            return;
        }
        if (!PROVIDER_RUSTORE.equals(provider) && !PROVIDER_HUAWEI.equals(provider)) {
            Log.w("Push", "Unsupported push provider: " + provider);
            return;
        }
        securePrefs.putString(PREF_TOKEN_PREFIX + provider, token);

        if (!authManager.isAuthenticated()) {
            // Пользователь пока не залогинен — отправим позже.
            Log.d("Push", "User not authenticated yet, will send " + provider + " token after login");
            return;
        }

        sendTokenToServer(provider, token);
    }

    /**
     * Вызывается после успешного логина, чтобы убедиться, что токен отправлен на сервер.
     */
    public void sendStoredTokenIfNeeded() {
        if (!authManager.isAuthenticated()) {
            return;
        }
        for (String provider : new String[]{PROVIDER_RUSTORE, PROVIDER_HUAWEI}) {
            String token = securePrefs.getString(PREF_TOKEN_PREFIX + provider);
            if (token != null) {
                sendTokenToServer(provider, token);
            }
        }
    }

    private void migrateLegacyPushToken(Context appContext, String provider) {
        android.content.SharedPreferences legacyPrefs = appContext.getSharedPreferences(
                com.invenbase.app.Config.PREF_NAME,
                Context.MODE_PRIVATE
        );
        String key = PREF_TOKEN_PREFIX + provider;
        if (!securePrefs.contains(key)) {
            String legacyToken = legacyPrefs.getString(key, null);
            if (legacyToken != null) {
                securePrefs.putString(key, legacyToken);
            }
        }
        legacyPrefs.edit().remove(key).commit();
    }

    // Метод sendTokenToServer: выполняет основную бизнес- или UI-логику данного участка кода.
    private void sendTokenToServer(String provider, String token) {
        Map<String, String> body = new HashMap<>();
        body.put("token", token);
        body.put("platform", "android");
        body.put("provider", provider);
        body.put("device_id", deviceId);

        Log.d("Push", "Sending " + provider + " push token to server");
        
        apiService.registerPushToken(body).enqueue(new Callback<Void>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Void> call, Response<Void> response) {
                if (response.isSuccessful()) {
                    Log.d("Push", provider + " token registered on server (status: " + response.code() + ")");
                } else {
                    Log.e("Push", "Failed to register " + provider + " token. Code=" + response.code());
                    if (response.errorBody() != null) {
                        try {
                            String errorBody = response.errorBody().string();
                            Log.e("Push", "Error response: " + errorBody);
                        } catch (Exception e) {
                            Log.e("Push", "Could not read error body", e);
                        }
                    }
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Void> call, Throwable t) {
                Log.e("Push", "Error registering " + provider + " token", t);
            }
        });
    }
}

