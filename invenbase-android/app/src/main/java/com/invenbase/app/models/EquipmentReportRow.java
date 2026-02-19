package com.invenbase.app.models;

public class EquipmentReportRow {
    public final int index;
    public final String name;
    public final String category;
    public final int total;
    public final int available;
    public final int booked;
    public final String status;

    public EquipmentReportRow(int index, String name, String category, int total, int available, int booked, String status) {
        this.index = index;
        this.name = name != null ? name : "—";
        this.category = category != null ? category : "—";
        this.total = total;
        this.available = available;
        this.booked = booked;
        this.status = status != null ? status : "—";
    }
}
