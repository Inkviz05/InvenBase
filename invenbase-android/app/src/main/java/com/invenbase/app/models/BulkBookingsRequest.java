package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

import java.util.List;

/** Тело запроса POST /api/bookings/bulk. */
public class BulkBookingsRequest {
    @SerializedName("bookings")
    private List<CreateBookingItem> bookings;

    // Конструктор BulkBookingsRequest: инициализирует объект и его зависимости.
    public BulkBookingsRequest(List<CreateBookingItem> bookings) {
        this.bookings = bookings;
    }

    // Метод getBookings: возвращает нужное значение для текущего контекста.
    public List<CreateBookingItem> getBookings() {
        return bookings;
    }
}
