package com.invenbase.app.models;

public class StatItem {
    private String title;
    private String value;

    // Конструктор StatItem: инициализирует объект и его зависимости.
    public StatItem(String title, String value) {
        this.title = title;
        this.value = value;
    }

    // Метод getTitle: возвращает нужное значение для текущего контекста.
    public String getTitle() {
        return title;
    }

    // Метод getValue: возвращает нужное значение для текущего контекста.
    public String getValue() {
        return value;
    }
}
