package com.invenbase.app;

import android.util.Log;

import androidx.annotation.NonNull;

import com.invenbase.app.utils.PushManager;
import com.invenbase.app.notifications.NotificationSyncScheduler;
import com.invenbase.app.notifications.NotificationSyncStore;

import java.util.List;

import ru.rustore.sdk.pushclient.messaging.exception.RuStorePushClientException;
import ru.rustore.sdk.pushclient.messaging.model.RemoteMessage;
import ru.rustore.sdk.pushclient.messaging.service.RuStoreMessagingService;

public class MyRuStoreMessagingService extends RuStoreMessagingService {
    @Override
    public void onNewToken(@NonNull String token) {
        PushManager.getInstance(getApplicationContext())
                .registerToken(PushManager.PROVIDER_RUSTORE, token);
    }

    @Override
    public void onMessageReceived(@NonNull RemoteMessage message) {
        // Notification payload отображается самим RuStore SDK. Данные нужны для навигации
        // и последующей синхронизации полного уведомления по notification_id.
        Log.d("RuStorePush", "Message received, notification_id="
                + message.getData().get("notification_id"));
        NotificationSyncStore.markHandled(
                getApplicationContext(),
                message.getData().get("notification_id")
        );
    }

    @Override
    public void onDeletedMessages() {
        Log.w("RuStorePush", "Messages were deleted before delivery; notification sync required");
        NotificationSyncScheduler.enqueueImmediate(getApplicationContext());
    }

    @Override
    public void onError(@NonNull List<? extends RuStorePushClientException> errors) {
        for (RuStorePushClientException error : errors) {
            Log.w("RuStorePush", "RuStore Push SDK error", error);
        }
    }
}
