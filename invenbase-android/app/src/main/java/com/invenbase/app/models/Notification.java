package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public class Notification {
    @SerializedName("id")
    private String id;
    
    @SerializedName("user_id")
    private String userId;
    
    @SerializedName("title")
    private String title;
    
    @SerializedName("message")
    private String message;
    
    @SerializedName("is_read")
    private boolean isRead;
    
    @SerializedName("booking_id")
    private String bookingId;
    
    @SerializedName("created_at")
    private String createdAt;

    // Конструктор Notification: инициализирует объект и его зависимости.
    public Notification() {}

    // Метод getId: возвращает нужное значение для текущего контекста.
    public String getId() {
        return id;
    }

    // Метод setId: устанавливает или обновляет значение данных.
    public void setId(String id) {
        this.id = id;
    }

    // Метод getUserId: возвращает нужное значение для текущего контекста.
    public String getUserId() {
        return userId;
    }

    // Метод setUserId: устанавливает или обновляет значение данных.
    public void setUserId(String userId) {
        this.userId = userId;
    }

    // Метод getTitle: возвращает нужное значение для текущего контекста.
    public String getTitle() {
        return title;
    }

    // Метод setTitle: устанавливает или обновляет значение данных.
    public void setTitle(String title) {
        this.title = title;
    }

    // Метод getMessage: возвращает нужное значение для текущего контекста.
    public String getMessage() {
        return message;
    }

    // Метод setMessage: устанавливает или обновляет значение данных.
    public void setMessage(String message) {
        this.message = message;
    }

    // Метод isRead: проверяет условие и возвращает логический результат.
    public boolean isRead() {
        return isRead;
    }

    // Метод setRead: устанавливает или обновляет значение данных.
    public void setRead(boolean read) {
        isRead = read;
    }

    // Метод getBookingId: возвращает нужное значение для текущего контекста.
    public String getBookingId() {
        return bookingId;
    }

    // Метод setBookingId: устанавливает или обновляет значение данных.
    public void setBookingId(String bookingId) {
        this.bookingId = bookingId;
    }

    // Метод getCreatedAt: возвращает нужное значение для текущего контекста.
    public String getCreatedAt() {
        return createdAt;
    }

    // Метод setCreatedAt: устанавливает или обновляет значение данных.
    public void setCreatedAt(String createdAt) {
        this.createdAt = createdAt;
    }
}
