package com.invenbase.app.notifications;

import android.content.Context;
import android.content.SharedPreferences;

import com.invenbase.app.Config;

import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

public final class NotificationSyncStore {
    private static final String PREF_HANDLED_IDS = "handled_notification_ids";
    private static final int MAX_IDS = 500;

    private NotificationSyncStore() {
    }

    public static synchronized boolean isHandled(Context context, String notificationId) {
        return preferences(context).getStringSet(PREF_HANDLED_IDS, new LinkedHashSet<>())
                .contains(notificationId);
    }

    public static synchronized void markHandled(Context context, String notificationId) {
        if (notificationId == null || notificationId.isEmpty()) {
            return;
        }
        Set<String> stored = preferences(context)
                .getStringSet(PREF_HANDLED_IDS, new LinkedHashSet<>());
        LinkedHashSet<String> updated = new LinkedHashSet<>(stored);
        updated.add(notificationId);
        if (updated.size() > MAX_IDS) {
            List<String> ids = new ArrayList<>(updated);
            updated = new LinkedHashSet<>(ids.subList(ids.size() - MAX_IDS, ids.size()));
        }
        preferences(context).edit().putStringSet(PREF_HANDLED_IDS, updated).apply();
    }

    private static SharedPreferences preferences(Context context) {
        return context.getApplicationContext()
                .getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
    }
}
