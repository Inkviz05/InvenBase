package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public class Category {
    @SerializedName("id")
    private String id;
    
    @SerializedName("name")
    private String name;
    
    @SerializedName("description")
    private String description;

    @SerializedName("squad_id")
    private String squadId;
    
    @SerializedName("created_at")
    private String createdAt;
    
    @SerializedName("updated_at")
    private String updatedAt;

    // Конструктор Category: инициализирует объект и его зависимости.
    public Category() {}

    // Конструктор Category: инициализирует объект и его зависимости.
    public Category(String id, String name, String description) {
        this.id = id;
        this.name = name;
        this.description = description;
    }

    // Метод getId: возвращает нужное значение для текущего контекста.
    public String getId() {
        return id;
    }

    // Метод setId: устанавливает или обновляет значение данных.
    public void setId(String id) {
        this.id = id;
    }

    // Метод getName: возвращает нужное значение для текущего контекста.
    public String getName() {
        return name;
    }

    // Метод setName: устанавливает или обновляет значение данных.
    public void setName(String name) {
        this.name = name;
    }

    // Метод getDescription: возвращает нужное значение для текущего контекста.
    public String getDescription() {
        return description;
    }

    // Метод setDescription: устанавливает или обновляет значение данных.
    public void setDescription(String description) {
        this.description = description;
    }

    // Метод getSquadId: возвращает нужное значение для текущего контекста.
    public String getSquadId() {
        return squadId;
    }

    // Метод setSquadId: устанавливает или обновляет значение данных.
    public void setSquadId(String squadId) {
        this.squadId = squadId;
    }

    // Метод getCreatedAt: возвращает нужное значение для текущего контекста.
    public String getCreatedAt() {
        return createdAt;
    }

    // Метод setCreatedAt: устанавливает или обновляет значение данных.
    public void setCreatedAt(String createdAt) {
        this.createdAt = createdAt;
    }

    // Метод getUpdatedAt: возвращает нужное значение для текущего контекста.
    public String getUpdatedAt() {
        return updatedAt;
    }

    // Метод setUpdatedAt: устанавливает или обновляет значение данных.
    public void setUpdatedAt(String updatedAt) {
        this.updatedAt = updatedAt;
    }
}
