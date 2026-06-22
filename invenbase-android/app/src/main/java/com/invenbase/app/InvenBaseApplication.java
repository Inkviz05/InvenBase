package com.invenbase.app;

import android.app.Application;

import com.google.firebase.FirebaseApp;
import com.google.firebase.messaging.FirebaseMessaging;

public class InvenBaseApplication extends Application {
    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    public void onCreate() {
        super.onCreate();
        FirebaseApp.initializeApp(this);
        
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
}
