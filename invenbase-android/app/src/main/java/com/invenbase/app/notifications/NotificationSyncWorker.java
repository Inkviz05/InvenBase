package com.invenbase.app.notifications;

import android.content.Context;

import androidx.annotation.NonNull;
import androidx.work.Worker;
import androidx.work.WorkerParameters;

import com.invenbase.app.api.ApiClient;
import com.invenbase.app.utils.AuthManager;

import java.io.IOException;
import java.util.List;
import java.util.Map;

import retrofit2.Response;

public class NotificationSyncWorker extends Worker {
    public NotificationSyncWorker(@NonNull Context context, @NonNull WorkerParameters params) {
        super(context, params);
    }

    @NonNull
    @Override
    public Result doWork() {
        Context context = getApplicationContext();
        if (!AuthManager.getInstance(context).isAuthenticated()) {
            return Result.success();
        }

        try {
            Response<List<Map<String, Object>>> response = ApiClient.getInstance(context)
                    .getApiService()
                    .getNotifications()
                    .execute();
            if (response.code() == 401 || response.code() == 403) {
                return Result.success();
            }
            if (!response.isSuccessful() || response.body() == null) {
                return response.code() >= 500 ? Result.retry() : Result.failure();
            }

            for (Map<String, Object> item : response.body()) {
                String id = stringValue(item.get("id"));
                boolean isRead = Boolean.TRUE.equals(item.get("is_read"));
                if (id == null || isRead || NotificationSyncStore.isHandled(context, id)) {
                    continue;
                }
                boolean shown = LocalNotificationHelper.show(
                        context,
                        id,
                        stringValue(item.get("title")),
                        stringValue(item.get("message"))
                );
                if (shown) {
                    NotificationSyncStore.markHandled(context, id);
                }
            }
            return Result.success();
        } catch (IOException error) {
            return Result.retry();
        }
    }

    private static String stringValue(Object value) {
        return value == null ? null : String.valueOf(value);
    }
}
