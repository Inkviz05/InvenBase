package com.invenbase.app.models;

public class CartItem {
    private String equipmentId;
    private String equipmentName;
    private int quantity;

    // Конструктор CartItem: инициализирует объект и его зависимости.
    public CartItem() {}

    // Конструктор CartItem: инициализирует объект и его зависимости.
    public CartItem(String equipmentId, String equipmentName, int quantity) {
        this.equipmentId = equipmentId;
        this.equipmentName = equipmentName;
        this.quantity = quantity;
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

    // Метод getQuantity: возвращает нужное значение для текущего контекста.
    public int getQuantity() {
        return quantity;
    }

    // Метод setQuantity: устанавливает или обновляет значение данных.
    public void setQuantity(int quantity) {
        this.quantity = quantity;
    }
}
