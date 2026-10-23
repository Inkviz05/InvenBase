package com.invenbase.app;

import android.app.Application;
import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.os.Build;

import com.google.firebase.FirebaseApp;
import com.google.firebase.messaging.FirebaseMessaging;
import com.invenbase.app.utils.PushManager;
import com.invenbase.app.notifications.NotificationSyncScheduler;

import ru.rustore.sdk.pushclient.RuStorePushClient;

public class InvenBaseApplication extends Application {
    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    public void onCreate() {
        super.onCreate();
        createNotificationChannel();
        NotificationSyncScheduler.schedulePeriodic(this);
        FirebaseApp.initializeApp(this);
        if (BuildConfig.HUAWEI_ENABLED) {
            new Thread(() -> {
                try {
                    String appId = com.huawei.agconnect.config.AGConnectServicesConfig
                            .fromContext(this).getString("client/app_id");
                    String token = com.huawei.hms.aaid.HmsInstanceId.getInstance(this)
                            .getToken(appId, "HCM");
                    PushManager.getInstance(this).registerToken(PushManager.PROVIDER_HUAWEI, token);
                } catch (Exception error) {
                    android.util.Log.w("HuaweiPush", "Unable to obtain push token", error);
                }
            }, "huawei-push-token").start();
        }

        if (!BuildConfig.RUSTORE_PROJECT_ID.isEmpty()) {
            RuStorePushClient.INSTANCE.init(this, BuildConfig.RUSTORE_PROJECT_ID);
            RuStorePushClient.INSTANCE.getToken()
                    .addOnSuccessListener(token -> PushManager.getInstance(this)
                            .registerToken(PushManager.PROVIDER_RUSTORE, token))
                    .addOnFailureListener(error -> android.util.Log.w(
                            "RuStorePush", "Unable to obtain RuStore push token", error));
        } else {
            android.util.Log.i("RuStorePush", "RUSTORE_PROJECT_ID is not configured; SDK disabled");
        }
        
        // Подписываемся на топик уведомлений
        FirebaseMessaging.getInstance().subscribeToTopic("all")
                .addOnCompleteListener(task -> {
                    if (task.isSuccessful()) {
                        android.util.Log.d("FCM", "Subscribed to notifications topic");
                    } else {
                        android.util.Log.e("FCM", "Failed to subscribe", task.getException());
                    }
                });
    }

    private void createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel channel = new NotificationChannel(
                    "invenbase_notifications",
                    "InvenBase Notifications",
                    NotificationManager.IMPORTANCE_HIGH
            );
            channel.setDescription("Уведомления о бронированиях и событиях");
            NotificationManager manager = getSystemService(NotificationManager.class);
            if (manager != null) {
                manager.createNotificationChannel(channel);
            }
        }
    }
}
