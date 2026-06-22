package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public class LoginResponse {
    @SerializedName("token")
    private String token;
    
    @SerializedName("user")
    private User user;

    // Конструктор LoginResponse: инициализирует объект и его зависимости.
    public LoginResponse() {}

    // Метод getToken: возвращает нужное значение для текущего контекста.
    public String getToken() {
        return token;
    }

    // Метод setToken: устанавливает или обновляет значение данных.
    public void setToken(String token) {
        this.token = token;
    }

    // Метод getUser: возвращает нужное значение для текущего контекста.
    public User getUser() {
        return user;
    }

    // Метод setUser: устанавливает или обновляет значение данных.
    public void setUser(User user) {
        this.user = user;
    }
}
