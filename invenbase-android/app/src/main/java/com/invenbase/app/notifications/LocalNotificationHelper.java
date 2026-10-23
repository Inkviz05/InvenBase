package com.invenbase.app.notifications;

import android.app.NotificationManager;
import android.Manifest;
import android.content.pm.PackageManager;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;

import androidx.core.app.NotificationCompat;
import androidx.core.app.NotificationManagerCompat;
import androidx.core.content.ContextCompat;

import com.invenbase.app.NotificationsActivity;
import com.invenbase.app.R;

public final class LocalNotificationHelper {
    private static final String CHANNEL_ID = "invenbase_notifications";

    private LocalNotificationHelper() {
    }

    public static boolean show(Context context, String id, String title, String body) {
        if (android.os.Build.VERSION.SDK_INT >= android.os.Build.VERSION_CODES.TIRAMISU
                && ContextCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS)
                != PackageManager.PERMISSION_GRANTED) {
            return false;
        }
        Intent intent = new Intent(context, NotificationsActivity.class)
                .addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP | Intent.FLAG_ACTIVITY_NEW_TASK)
                .putExtra("notification_id", id);
        PendingIntent pendingIntent = PendingIntent.getActivity(
                context,
                id.hashCode(),
                intent,
                PendingIntent.FLAG_IMMUTABLE | PendingIntent.FLAG_UPDATE_CURRENT
        );
        NotificationCompat.Builder notification = new NotificationCompat.Builder(context, CHANNEL_ID)
                .setSmallIcon(R.drawable.ic_notification)
                .setContentTitle(title == null || title.isEmpty() ? "InvenBase" : title)
                .setContentText(body)
                .setStyle(new NotificationCompat.BigTextStyle().bigText(body))
                .setPriority(NotificationCompat.PRIORITY_HIGH)
                .setAutoCancel(true)
                .setContentIntent(pendingIntent)
                .setCategory(NotificationCompat.CATEGORY_STATUS);

        NotificationManagerCompat.from(context).notify(id.hashCode(), notification.build());
        return true;
    }
}
