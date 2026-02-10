package com.invenbase.app.models;

public class StatItem {
    private String title;
    private String value;

    public StatItem(String title, String value) {
        this.title = title;
        this.value = value;
    }

    public String getTitle() {
        return title;
    }

    public String getValue() {
        return value;
    }
}
