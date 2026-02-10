package com.invenbase.app.models;

/**
 * Элемент статистики по категории для экрана отчётов (как в веб-версии).
 */
public class ReportCategoryItem {
    private final String categoryName;
    private final long total;
    private final long available;
    private final long booked;

    public ReportCategoryItem(String categoryName, long total, long available, long booked) {
        this.categoryName = categoryName != null ? categoryName : "";
        this.total = total;
        this.available = available;
        this.booked = booked;
    }

    public String getCategoryName() {
        return categoryName;
    }

    public long getTotal() {
        return total;
    }

    public long getAvailable() {
        return available;
    }

    public long getBooked() {
        return booked;
    }
}
