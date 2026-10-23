package com.invenbase.app;

import android.os.Bundle;
import com.huawei.hms.push.HmsMessageService;
import com.huawei.hms.push.RemoteMessage;
import com.invenbase.app.utils.PushManager;
import com.invenbase.app.notifications.NotificationSyncScheduler;

public class MyHuaweiMessagingService extends HmsMessageService {
    @Override
    public void onNewToken(String token) {
        PushManager.getInstance(this).registerToken(PushManager.PROVIDER_HUAWEI, token);
    }

    @Override
    public void onNewToken(String token, Bundle bundle) {
        onNewToken(token);
    }

    @Override
    public void onMessageReceived(RemoteMessage message) {
        NotificationSyncScheduler.enqueueImmediate(this);
    }
}
