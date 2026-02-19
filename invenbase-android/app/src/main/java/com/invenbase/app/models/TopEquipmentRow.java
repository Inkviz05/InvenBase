package com.invenbase.app.models;

public class TopEquipmentRow {
    public final int index;
    public final String equipmentName;
    public final String categoryName;
    public final int bookingsCount;
    public final String percent;
    public final int totalQuantity;

    public TopEquipmentRow(int index, String equipmentName, String categoryName, int bookingsCount, String percent, int totalQuantity) {
        this.index = index;
        this.equipmentName = equipmentName != null ? equipmentName : "—";
        this.categoryName = categoryName != null ? categoryName : "—";
        this.bookingsCount = bookingsCount;
        this.percent = percent != null ? percent : "0.0";
        this.totalQuantity = totalQuantity;
    }
}
