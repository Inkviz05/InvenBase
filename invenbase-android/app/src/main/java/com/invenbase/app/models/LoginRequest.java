package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public class LoginRequest {
    @SerializedName("username")
    private String username;
    
    @SerializedName("password")
    private String password;

    // Конструктор LoginRequest: инициализирует объект и его зависимости.
    public LoginRequest(String username, String password) {
        this.username = username;
        this.password = password;
    }

    // Метод getUsername: возвращает нужное значение для текущего контекста.
    public String getUsername() {
        return username;
    }

    // Метод setUsername: устанавливает или обновляет значение данных.
    public void setUsername(String username) {
        this.username = username;
    }

    // Метод getPassword: возвращает нужное значение для текущего контекста.
    public String getPassword() {
        return password;
    }

    // Метод setPassword: устанавливает или обновляет значение данных.
    public void setPassword(String password) {
        this.password = password;
    }
}
