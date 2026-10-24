package com.invenbase.app.utils;

import android.content.Context;
import android.content.SharedPreferences;
import android.util.Log;

import com.google.gson.Gson;
import com.invenbase.app.Config;
import com.invenbase.app.models.User;
import com.invenbase.app.api.ApiClient;

import java.util.HashMap;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class AuthManager {
    private static AuthManager instance;
    private final SharedPreferences legacyPrefs;
    private final SecurePreferences securePrefs;
    private final Gson gson;
    private final Context appContext;

    // Конструктор AuthManager: инициализирует объект и его зависимости.
    private AuthManager(Context context) {
        appContext = context.getApplicationContext();
        legacyPrefs = appContext.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
        securePrefs = new SecurePreferences(appContext);
        gson = new Gson();
        migrateLegacySession();
    }

    // Метод getInstance: возвращает нужное значение для текущего контекста.
    public static synchronized AuthManager getInstance(Context context) {
        if (instance == null) {
            instance = new AuthManager(context.getApplicationContext());
        }
        return instance;
    }

    // Метод saveToken: выполняет основную бизнес- или UI-логику данного участка кода.
    public void saveToken(String token) {
        securePrefs.putString(Config.PREF_TOKEN, token);
    }

    // Метод getToken: возвращает нужное значение для текущего контекста.
    public String getToken() {
        return securePrefs.getString(Config.PREF_TOKEN);
    }

    // Метод saveUser: выполняет основную бизнес- или UI-логику данного участка кода.
    public void saveUser(User user) {
        String userJson = gson.toJson(user);
        securePrefs.putString(Config.PREF_USER, userJson);
    }

    // Метод getUser: возвращает нужное значение для текущего контекста.
    public User getUser() {
        String userJson = securePrefs.getString(Config.PREF_USER);
        if (userJson == null) {
            return null;
        }
        return gson.fromJson(userJson, User.class);
    }

    // Метод isAuthenticated: проверяет условие и возвращает логический результат.
    public boolean isAuthenticated() {
        return getToken() != null && getUser() != null;
    }

    // Метод logout: выполняет основную бизнес- или UI-логику данного участка кода.
    public void logout() {
        String token = getToken();
        if (token != null) {
            String deviceId = DeviceIdentity.get(appContext);
            unregisterPushDevice(token, deviceId, PushManager.PROVIDER_RUSTORE);
            unregisterPushDevice(token, deviceId, PushManager.PROVIDER_HUAWEI);
        }

        securePrefs.remove(Config.PREF_TOKEN);
        securePrefs.remove(Config.PREF_USER);
    }

    private void migrateLegacySession() {
        migrateLegacyValue(Config.PREF_TOKEN);
        migrateLegacyValue(Config.PREF_USER);
    }

    private void migrateLegacyValue(String key) {
        if (!securePrefs.contains(key)) {
            String legacyValue = legacyPrefs.getString(key, null);
            if (legacyValue != null) {
                securePrefs.putString(key, legacyValue);
            }
        }
        legacyPrefs.edit().remove(key).commit();
    }

    private void unregisterPushDevice(String authToken, String deviceId, String provider) {
        Map<String, String> body = new HashMap<>();
        body.put("device_id", deviceId);
        body.put("provider", provider);
        ApiClient.getInstance(appContext)
                .getApiService()
                .unregisterPushDevice("Bearer " + authToken, body)
                .enqueue(new Callback<Void>() {
                    @Override
                    public void onResponse(Call<Void> call, Response<Void> response) {
                        if (!response.isSuccessful()) {
                            Log.w("Auth", provider + " device unregister returned " + response.code());
                        }
                    }

                    @Override
                    public void onFailure(Call<Void> call, Throwable throwable) {
                        Log.w("Auth", provider + " device unregister failed", throwable);
                    }
                });
    }

    // Метод isAdmin: проверяет условие и возвращает логический результат.
    public boolean isAdmin() {
        User user = getUser();
        return user != null && user.isAdmin();
    }

    // Метод isResponsible: проверяет условие и возвращает логический результат.
    public boolean isResponsible() {
        User user = getUser();
        return user != null && user.isResponsible();
    }
}
