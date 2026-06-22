package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public class Booking {
    @SerializedName("id")
    private String id;
    
    @SerializedName("user_id")
    private String userId;
    
    @SerializedName("username")
    private String username;
    
    @SerializedName("equipment_id")
    private String equipmentId;
    
    @SerializedName("equipment_name")
    private String equipmentName;
    
    @SerializedName("group_id")
    private String groupId;
    
    @SerializedName("group_name")
    private String groupName;
    
    @SerializedName("quantity")
    private int quantity;
    
    @SerializedName("start_date")
    private String startDate;
    
    @SerializedName("end_date")
    private String endDate;
    
    @SerializedName("status")
    private String status;
    
    @SerializedName("purpose")
    private String purpose;
    
    @SerializedName("permission_type")
    private String permissionType;
    
    @SerializedName("created_at")
    private String createdAt;
    
    @SerializedName("updated_at")
    private String updatedAt;

    // Конструктор Booking: инициализирует объект и его зависимости.
    public Booking() {}

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

    // Метод getUsername: возвращает нужное значение для текущего контекста.
    public String getUsername() {
        return username;
    }

    // Метод setUsername: устанавливает или обновляет значение данных.
    public void setUsername(String username) {
        this.username = username;
    }

    // Метод getEquipmentId: возвращает нужное значение для текущего контекста.
    public String getEquipmentId() {
        return equipmentId;
    }

    // Метод setEquipmentId: устанавливает или обновляет значение данных.
    public void setEquipmentId(String equipmentId) {
        this.equipmentId = equipmentId;
    }

    // Метод getEquipmentName: возвращает нужное значение для текущего контекста.
    public String getEquipmentName() {
        return equipmentName;
    }

    // Метод setEquipmentName: устанавливает или обновляет значение данных.
    public void setEquipmentName(String equipmentName) {
        this.equipmentName = equipmentName;
    }

    // Метод getGroupId: возвращает нужное значение для текущего контекста.
    public String getGroupId() {
        return groupId;
    }

    // Метод setGroupId: устанавливает или обновляет значение данных.
    public void setGroupId(String groupId) {
        this.groupId = groupId;
    }

    // Метод getGroupName: возвращает нужное значение для текущего контекста.
    public String getGroupName() {
        return groupName;
    }

    // Метод setGroupName: устанавливает или обновляет значение данных.
    public void setGroupName(String groupName) {
        this.groupName = groupName;
    }

    // Метод getQuantity: возвращает нужное значение для текущего контекста.
    public int getQuantity() {
        return quantity;
    }

    // Метод setQuantity: устанавливает или обновляет значение данных.
    public void setQuantity(int quantity) {
        this.quantity = quantity;
    }

    // Метод getStartDate: возвращает нужное значение для текущего контекста.
    public String getStartDate() {
        return startDate;
    }

    // Метод setStartDate: устанавливает или обновляет значение данных.
    public void setStartDate(String startDate) {
        this.startDate = startDate;
    }

    // Метод getEndDate: возвращает нужное значение для текущего контекста.
    public String getEndDate() {
        return endDate;
    }

    // Метод setEndDate: устанавливает или обновляет значение данных.
    public void setEndDate(String endDate) {
        this.endDate = endDate;
    }

    // Метод getStatus: возвращает нужное значение для текущего контекста.
    public String getStatus() {
        return status;
    }

    // Метод setStatus: устанавливает или обновляет значение данных.
    public void setStatus(String status) {
        this.status = status;
    }

    // Метод getPurpose: возвращает нужное значение для текущего контекста.
    public String getPurpose() {
        return purpose;
    }

    // Метод setPurpose: устанавливает или обновляет значение данных.
    public void setPurpose(String purpose) {
        this.purpose = purpose;
    }

    // Метод getPermissionType: возвращает нужное значение для текущего контекста.
    public String getPermissionType() {
        return permissionType;
    }

    // Метод setPermissionType: устанавливает или обновляет значение данных.
    public void setPermissionType(String permissionType) {
        this.permissionType = permissionType;
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
