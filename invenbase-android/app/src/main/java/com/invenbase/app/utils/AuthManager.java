package com.invenbase.app.utils;

import android.content.Context;
import android.content.SharedPreferences;

import com.google.gson.Gson;
import com.invenbase.app.Config;
import com.invenbase.app.models.User;

public class AuthManager {
    private static AuthManager instance;
    private SharedPreferences prefs;
    private Gson gson;

    // Конструктор AuthManager: инициализирует объект и его зависимости.
    private AuthManager(Context context) {
        prefs = context.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
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
        prefs.edit()
            .remove(Config.PREF_TOKEN)
            .remove(Config.PREF_USER)
            .apply();
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
