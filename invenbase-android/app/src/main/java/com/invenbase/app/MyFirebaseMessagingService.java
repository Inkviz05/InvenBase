package com.invenbase.app;

import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;
import android.media.RingtoneManager;
import android.os.Build;
import android.widget.RemoteViews;

import androidx.core.app.NotificationCompat;
import androidx.core.content.ContextCompat;

import com.google.firebase.messaging.FirebaseMessagingService;
import com.google.firebase.messaging.RemoteMessage;

public class MyFirebaseMessagingService extends FirebaseMessagingService {

    private static final String CHANNEL_ID = "invenbase_notifications";

    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    public void onCreate() {
        super.onCreate();
        createNotificationChannel();
    }

    @Override
    // Метод onMessageReceived: обрабатывает соответствующее событие приложения.
    public void onMessageReceived(RemoteMessage remoteMessage) {
        android.util.Log.d("FCM", "=== onMessageReceived called ===");
        android.util.Log.d("FCM", "Message ID: " + remoteMessage.getMessageId());
        android.util.Log.d("FCM", "From: " + remoteMessage.getFrom());
        android.util.Log.d("FCM", "Notification: " + (remoteMessage.getNotification() != null ? "present" : "null"));
        android.util.Log.d("FCM", "Data: " + remoteMessage.getData());
        
        // Логируем все детали для отладки
        if (remoteMessage.getNotification() != null) {
            android.util.Log.d("FCM", "Notification title: " + remoteMessage.getNotification().getTitle());
            android.util.Log.d("FCM", "Notification body: " + remoteMessage.getNotification().getBody());
        }
        if (!remoteMessage.getData().isEmpty()) {
            android.util.Log.d("FCM", "Data payload size: " + remoteMessage.getData().size());
            for (java.util.Map.Entry<String, String> entry : remoteMessage.getData().entrySet()) {
                android.util.Log.d("FCM", "Data[" + entry.getKey() + "] = " + entry.getValue());
            }
        }
        
        String title = null;
        String body = null;
        
        // Приоритет: notification payload, затем data payload
        if (remoteMessage.getNotification() != null) {
            title = remoteMessage.getNotification().getTitle();
            body = remoteMessage.getNotification().getBody();
        }
        
        // Если нет notification payload, используем data
        java.util.Map<String, String> data = remoteMessage.getData();
        if (title == null && data.containsKey("title")) {
            title = data.get("title");
        }
        if (body == null && data.containsKey("body")) {
            body = data.get("body");
        }
        
        // Если всё ещё нет заголовка, используем дефолтный
        if (title == null || title.isEmpty()) {
            title = "InvenBase";
        }
        
        // Если есть body или data, показываем уведомление
        if (body != null && !body.isEmpty()) {
            sendNotification(title, body, data);
        } else if (!data.isEmpty()) {
            // Даже если нет body, но есть data - показываем уведомление
            sendNotification(title, "Новое уведомление", data);
        }
    }

    @Override
    // Метод onNewToken: обрабатывает соответствующее событие приложения.
    public void onNewToken(String token) {
        android.util.Log.d("FCM", "=== onNewToken called ===");
        android.util.Log.d("FCM", "Refreshed token: " + (token != null ? token.substring(0, Math.min(30, token.length())) + "..." : "null"));
        // Сохраняем и отправляем токен на сервер
        if (token != null && !token.isEmpty()) {
            com.invenbase.app.utils.PushManager.getInstance(getApplicationContext())
                    .registerToken(token);
        } else {
            android.util.Log.e("FCM", "Token is null or empty!");
        }
    }

    // Метод createNotificationChannel: выполняет основную бизнес- или UI-логику данного участка кода.
    private void createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            NotificationChannel channel = new NotificationChannel(
                    CHANNEL_ID,
                    "InvenBase Notifications",
                    NotificationManager.IMPORTANCE_HIGH
            );
            channel.setDescription("Уведомления о бронированиях и событиях");
            channel.enableLights(true);
            channel.enableVibration(true);
            channel.setShowBadge(true);
            int accentColor = ContextCompat.getColor(this, R.color.primary);
            channel.setLightColor(accentColor);
            NotificationManager manager = getSystemService(NotificationManager.class);
            if (manager != null) {
                manager.createNotificationChannel(channel);
                android.util.Log.d("FCM", "Notification channel created: " + CHANNEL_ID);
            } else {
                android.util.Log.e("FCM", "Failed to get NotificationManager");
            }
        }
    }

    // Метод sendNotification: выполняет основную бизнес- или UI-логику данного участка кода.
    private void sendNotification(String title, String messageBody, java.util.Map<String, String> data) {
        Intent intent = new Intent(this, MainActivity.class);
        
        // Определяем, куда открывать уведомление на основе data
        if (data != null && !data.isEmpty()) {
            String type = data.get("type");
            String bookingId = data.get("booking_id");
            String equipmentId = data.get("equipment_id");
            
            if ("notification".equals(type) || bookingId != null || equipmentId != null) {
                // Открываем экран уведомлений
                intent = new Intent(this, NotificationsActivity.class);
            } else if (equipmentId != null) {
                // Открываем детали оборудования
                intent = new Intent(this, com.invenbase.app.EquipmentDetailActivity.class);
                intent.putExtra(com.invenbase.app.EquipmentDetailActivity.EXTRA_EQUIPMENT_ID, equipmentId);
            }
        }
        
        intent.addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP | Intent.FLAG_ACTIVITY_NEW_TASK);
        
        // Добавляем data в intent для дальнейшей обработки
        if (data != null) {
            for (java.util.Map.Entry<String, String> entry : data.entrySet()) {
                intent.putExtra(entry.getKey(), entry.getValue());
            }
        }
        
        PendingIntent pendingIntent = PendingIntent.getActivity(
                this, (int) System.currentTimeMillis(), intent,
                PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT
        );

        // Цвет любых системных областей — surface (тёмный), не чёрный
        int notificationColor = ContextCompat.getColor(this, R.color.surface);
        String titleStr = title != null ? title : "InvenBase";

        RemoteViews collapsedView = new RemoteViews(getPackageName(), R.layout.notification_collapsed);
        collapsedView.setTextViewText(R.id.notification_title, titleStr);
        collapsedView.setTextViewText(R.id.notification_body, messageBody);
        collapsedView.setImageViewResource(R.id.notification_logo, R.drawable.app_icon_logo);

        RemoteViews expandedView = new RemoteViews(getPackageName(), R.layout.notification_expanded);
        expandedView.setTextViewText(R.id.notification_title, titleStr);
        expandedView.setTextViewText(R.id.notification_body, messageBody);
        expandedView.setImageViewResource(R.id.notification_logo, R.drawable.app_icon_logo);

        NotificationCompat.Builder notificationBuilder = new NotificationCompat.Builder(this, CHANNEL_ID)
                .setSmallIcon(R.drawable.ic_notification)
                .setContentTitle(titleStr)
                .setContentText(messageBody)
                .setAutoCancel(true)
                .setSound(RingtoneManager.getDefaultUri(RingtoneManager.TYPE_NOTIFICATION))
                .setContentIntent(pendingIntent)
                .setPriority(NotificationCompat.PRIORITY_HIGH)
                .setDefaults(NotificationCompat.DEFAULT_ALL)
                .setColor(notificationColor)
                .setCustomContentView(collapsedView)
                .setCustomBigContentView(expandedView);
        // Не ставим setLargeIcon — логотип уже в кастомных макетах, иначе возможен артефакт

        NotificationManager notificationManager =
                (NotificationManager) getSystemService(Context.NOTIFICATION_SERVICE);

        if (notificationManager != null) {
            int notificationId = (int) System.currentTimeMillis();
            notificationManager.notify(notificationId, notificationBuilder.build());
            android.util.Log.d("FCM", "Notification shown with ID: " + notificationId);
            android.util.Log.d("FCM", "Notification title: " + title);
            android.util.Log.d("FCM", "Notification body: " + messageBody);
        } else {
            android.util.Log.e("FCM", "NotificationManager is null! Cannot show notification.");
        }
    }
}
