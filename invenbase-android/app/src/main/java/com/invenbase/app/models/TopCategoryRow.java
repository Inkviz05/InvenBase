package com.invenbase.app.models;

public class TopCategoryRow {
    public final int index;
    public final String categoryName;
    public final int bookingsCount;
    public final String percent;

    // Конструктор TopCategoryRow: инициализирует объект и его зависимости.
    public TopCategoryRow(int index, String categoryName, int bookingsCount, String percent) {
        this.index = index;
        this.categoryName = categoryName != null ? categoryName : "—";
        this.bookingsCount = bookingsCount;
        this.percent = percent != null ? percent : "0.0";
    }
}
