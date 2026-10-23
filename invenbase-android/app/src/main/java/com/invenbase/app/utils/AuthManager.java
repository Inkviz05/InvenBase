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
    private SharedPreferences prefs;
    private Gson gson;
    private Context appContext;

    // Конструктор AuthManager: инициализирует объект и его зависимости.
    private AuthManager(Context context) {
        appContext = context.getApplicationContext();
        prefs = appContext.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
        gson = new Gson();
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
        prefs.edit().putString(Config.PREF_TOKEN, token).apply();
    }

    // Метод getToken: возвращает нужное значение для текущего контекста.
    public String getToken() {
        return prefs.getString(Config.PREF_TOKEN, null);
    }

    // Метод saveUser: выполняет основную бизнес- или UI-логику данного участка кода.
    public void saveUser(User user) {
        String userJson = gson.toJson(user);
        prefs.edit().putString(Config.PREF_USER, userJson).apply();
    }

    // Метод getUser: возвращает нужное значение для текущего контекста.
    public User getUser() {
        String userJson = prefs.getString(Config.PREF_USER, null);
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
            unregisterPushDevice(token, deviceId, PushManager.PROVIDER_FCM);
            unregisterPushDevice(token, deviceId, PushManager.PROVIDER_RUSTORE);
        }

        prefs.edit()
            .remove(Config.PREF_TOKEN)
            .remove(Config.PREF_USER)
            .apply();
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
