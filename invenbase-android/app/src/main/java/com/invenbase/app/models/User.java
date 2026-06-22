package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

// ООП (инкапсуляция): модель пользователя скрывает поля и даёт контролируемый доступ через методы.
public class User {
    @SerializedName("id")
    private String id;
    
    @SerializedName("username")
    private String username;
    
    @SerializedName("full_name")
    private String fullName;
    
    @SerializedName("email")
    private String email;
    
    @SerializedName("role")
    private String role;

    // ООП: конструктор по умолчанию нужен для сериализации/десериализации (например, Gson).
    public User() {}

    // ООП: параметризованный конструктор создаёт полностью инициализированный объект User.
    public User(String id, String username, String fullName, String email, String role) {
        this.id = id;
        this.username = username;
        this.fullName = fullName;
        this.email = email;
        this.role = role;
    }

    // Метод getId: возвращает нужное значение для текущего контекста.
    public String getId() {
        return id;
    }

    // Метод setId: устанавливает или обновляет значение данных.
    public void setId(String id) {
        this.id = id;
    }

    // Метод getUsername: возвращает нужное значение для текущего контекста.
    public String getUsername() {
        return username;
    }

    // Метод setUsername: устанавливает или обновляет значение данных.
    public void setUsername(String username) {
        this.username = username;
    }

    // Метод getFullName: возвращает нужное значение для текущего контекста.
    public String getFullName() {
        return fullName;
    }

    // Метод setFullName: устанавливает или обновляет значение данных.
    public void setFullName(String fullName) {
        this.fullName = fullName;
    }

    // Метод getEmail: возвращает нужное значение для текущего контекста.
    public String getEmail() {
        return email;
    }

    // Метод setEmail: устанавливает или обновляет значение данных.
    public void setEmail(String email) {
        this.email = email;
    }

    // Метод getRole: возвращает нужное значение для текущего контекста.
    public String getRole() {
        return role;
    }

    // Метод setRole: устанавливает или обновляет значение данных.
    public void setRole(String role) {
        this.role = role;
    }
    
    // ООП (инкапсуляция правил): бизнес-проверка роли вынесена в модель, а не дублируется в UI-слое.
    public boolean isAdmin() {
        return "admin".equals(role);
    }

    // ООП (инкапсуляция правил): централизованная проверка роли "responsible".
    public boolean isResponsible() {
        return "responsible".equals(role);
    }
}
