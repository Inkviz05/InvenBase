package com.invenbase.app.models;

/**
 * Элемент статистики по категории для экрана отчётов (как в веб-версии).
 */
public class ReportCategoryItem {
    private final String categoryName;
    private final long total;
    private final long available;
    private final long booked;

    // Конструктор ReportCategoryItem: инициализирует объект и его зависимости.
    public ReportCategoryItem(String categoryName, long total, long available, long booked) {
        this.categoryName = categoryName != null ? categoryName : "";
        this.total = total;
        this.available = available;
        this.booked = booked;
    }

    // Метод getCategoryName: возвращает нужное значение для текущего контекста.
    public String getCategoryName() {
        return categoryName;
    }

    // Метод getTotal: возвращает нужное значение для текущего контекста.
    public long getTotal() {
        return total;
    }

    // Метод getAvailable: возвращает нужное значение для текущего контекста.
    public long getAvailable() {
        return available;
    }

    // Метод getBooked: возвращает нужное значение для текущего контекста.
    public long getBooked() {
        return booked;
    }
}
