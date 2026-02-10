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

    private AuthManager(Context context) {
        prefs = context.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
        gson = new Gson();
    }

    public static synchronized AuthManager getInstance(Context context) {
        if (instance == null) {
            instance = new AuthManager(context.getApplicationContext());
        }
        return instance;
    }

    public void saveToken(String token) {
        prefs.edit().putString(Config.PREF_TOKEN, token).apply();
    }

    public String getToken() {
        return prefs.getString(Config.PREF_TOKEN, null);
    }

    public void saveUser(User user) {
        String userJson = gson.toJson(user);
        prefs.edit().putString(Config.PREF_USER, userJson).apply();
    }

    public User getUser() {
        String userJson = prefs.getString(Config.PREF_USER, null);
        if (userJson == null) {
            return null;
        }
        return gson.fromJson(userJson, User.class);
    }

    public boolean isAuthenticated() {
        return getToken() != null && getUser() != null;
    }

    public void logout() {
        prefs.edit()
            .remove(Config.PREF_TOKEN)
            .remove(Config.PREF_USER)
            .apply();
    }

    public boolean isAdmin() {
        User user = getUser();
        return user != null && user.isAdmin();
    }

    public boolean isResponsible() {
        User user = getUser();
        return user != null && user.isResponsible();
    }
}
