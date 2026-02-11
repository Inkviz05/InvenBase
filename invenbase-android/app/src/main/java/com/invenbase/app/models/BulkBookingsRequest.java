package com.invenbase.app.models;

import java.util.List;

/** Тело запроса POST /api/bookings/bulk. */
public class BulkBookingsRequest {
    private List<CreateBookingItem> bookings;

    public BulkBookingsRequest(List<CreateBookingItem> bookings) {
        this.bookings = bookings;
    }

    public List<CreateBookingItem> getBookings() {
        return bookings;
    }
}
