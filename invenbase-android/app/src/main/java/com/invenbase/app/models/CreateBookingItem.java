package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

/** Один элемент для массового создания бронирований (POST /api/bookings/bulk). */
public class CreateBookingItem {
    @SerializedName("equipment_id")
    private String equipmentId;

    @SerializedName("quantity")
    private int quantity;

    @SerializedName("start_date")
    private String startDate;

    @SerializedName("end_date")
    private String endDate;

    @SerializedName("purpose")
    private String purpose;

    @SerializedName("permission_type")
    private String permissionType;

    public CreateBookingItem(String equipmentId, int quantity, String startDate, String endDate, String purpose, String permissionType) {
        this.equipmentId = equipmentId;
        this.quantity = quantity;
        this.startDate = startDate;
        this.endDate = endDate;
        this.purpose = purpose;
        this.permissionType = permissionType;
    }
}
