package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public class Equipment {
    @SerializedName("id")
    private String id;
    
    @SerializedName("name")
    private String name;
    
    @SerializedName("description")
    private String description;
    
    @SerializedName("category_id")
    private String categoryId;
    
    @SerializedName("category_name")
    private String categoryName;
    
    @SerializedName("quantity")
    private int quantity;
    
    @SerializedName("available_quantity")
    private int availableQuantity;

    @SerializedName("is_unique")
    private boolean isUnique;

    @SerializedName("squad_id")
    private String squadId;

    @SerializedName("squad_name")
    private String squadName;

    @SerializedName("location")
    private String location;
    
    @SerializedName("qr_code")
    private String qrCode;
    
    @SerializedName("status")
    private String status;
    
    @SerializedName("created_at")
    private String createdAt;
    
    @SerializedName("updated_at")
    private String updatedAt;

    // Конструктор Equipment: инициализирует объект и его зависимости.
    public Equipment() {}

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

    // Метод getCategoryId: возвращает нужное значение для текущего контекста.
    public String getCategoryId() {
        return categoryId;
    }

    // Метод setCategoryId: устанавливает или обновляет значение данных.
    public void setCategoryId(String categoryId) {
        this.categoryId = categoryId;
    }

    // Метод getCategoryName: возвращает нужное значение для текущего контекста.
    public String getCategoryName() {
        return categoryName;
    }

    // Метод setCategoryName: устанавливает или обновляет значение данных.
    public void setCategoryName(String categoryName) {
        this.categoryName = categoryName;
    }

    // Метод getQuantity: возвращает нужное значение для текущего контекста.
    public int getQuantity() {
        return quantity;
    }

    // Метод setQuantity: устанавливает или обновляет значение данных.
    public void setQuantity(int quantity) {
        this.quantity = quantity;
    }

    // Метод getAvailableQuantity: возвращает нужное значение для текущего контекста.
    public int getAvailableQuantity() {
        return availableQuantity;
    }

    // Метод setAvailableQuantity: устанавливает или обновляет значение данных.
    public void setAvailableQuantity(int availableQuantity) {
        this.availableQuantity = availableQuantity;
    }

    // Метод isUnique: проверяет условие и возвращает логический результат.
    public boolean isUnique() {
        return isUnique;
    }

    // Метод setUnique: устанавливает или обновляет значение данных.
    public void setUnique(boolean unique) {
        isUnique = unique;
    }

    // Метод getSquadId: возвращает нужное значение для текущего контекста.
    public String getSquadId() {
        return squadId;
    }

    // Метод setSquadId: устанавливает или обновляет значение данных.
    public void setSquadId(String squadId) {
        this.squadId = squadId;
    }

    // Метод getSquadName: возвращает нужное значение для текущего контекста.
    public String getSquadName() {
        return squadName;
    }

    // Метод setSquadName: устанавливает или обновляет значение данных.
    public void setSquadName(String squadName) {
        this.squadName = squadName;
    }

    // Метод getLocation: возвращает нужное значение для текущего контекста.
    public String getLocation() {
        return location;
    }

    // Метод setLocation: устанавливает или обновляет значение данных.
    public void setLocation(String location) {
        this.location = location;
    }

    // Метод getQrCode: возвращает нужное значение для текущего контекста.
    public String getQrCode() {
        return qrCode;
    }

    // Метод setQrCode: устанавливает или обновляет значение данных.
    public void setQrCode(String qrCode) {
        this.qrCode = qrCode;
    }

    // Метод getStatus: возвращает нужное значение для текущего контекста.
    public String getStatus() {
        return status;
    }

    // Метод setStatus: устанавливает или обновляет значение данных.
    public void setStatus(String status) {
        this.status = status;
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
