package com.invenbase.app.utils;

import android.content.Context;
import android.content.SharedPreferences;
import android.provider.Settings;

import com.invenbase.app.Config;

import java.util.UUID;

public final class DeviceIdentity {
    private static final String PREF_DEVICE_ID = "push_device_id";

    private DeviceIdentity() {
    }

    public static String get(Context context) {
        Context appContext = context.getApplicationContext();
        String androidId = Settings.Secure.getString(
            appContext.getContentResolver(),
            Settings.Secure.ANDROID_ID
        );
        if (androidId != null && !androidId.trim().isEmpty()) {
            return androidId;
        }

        SharedPreferences prefs = appContext.getSharedPreferences(
            Config.PREF_NAME,
            Context.MODE_PRIVATE
        );
        String stored = prefs.getString(PREF_DEVICE_ID, null);
        if (stored != null) {
            return stored;
        }

        String generated = UUID.randomUUID().toString();
        prefs.edit().putString(PREF_DEVICE_ID, generated).apply();
        return generated;
    }
}
