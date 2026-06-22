package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public class Log {
    @SerializedName("id")
    private String id;
    
    @SerializedName("user_id")
    private String userId;
    
    @SerializedName("action")
    private String action;
    
    @SerializedName("entity_type")
    private String entityType;
    
    @SerializedName("entity_id")
    private String entityId;
    
    @SerializedName("details")
    private Object details;
    
    @SerializedName("created_at")
    private String createdAt;

    // Конструктор Log: инициализирует объект и его зависимости.
    public Log() {}

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

    // Метод getAction: возвращает нужное значение для текущего контекста.
    public String getAction() {
        return action;
    }

    // Метод setAction: устанавливает или обновляет значение данных.
    public void setAction(String action) {
        this.action = action;
    }

    // Метод getEntityType: возвращает нужное значение для текущего контекста.
    public String getEntityType() {
        return entityType;
    }

    // Метод setEntityType: устанавливает или обновляет значение данных.
    public void setEntityType(String entityType) {
        this.entityType = entityType;
    }

    // Метод getEntityId: возвращает нужное значение для текущего контекста.
    public String getEntityId() {
        return entityId;
    }

    // Метод setEntityId: устанавливает или обновляет значение данных.
    public void setEntityId(String entityId) {
        this.entityId = entityId;
    }

    // Метод getDetails: возвращает нужное значение для текущего контекста.
    public Object getDetails() {
        return details;
    }

    // Метод setDetails: устанавливает или обновляет значение данных.
    public void setDetails(Object details) {
        this.details = details;
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
