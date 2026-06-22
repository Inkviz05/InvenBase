package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

/** Ответ сервера при ошибке (400, 404 и т.д.): {"error":"...", "message":"..."} */
public class ApiError {
    @SerializedName("message")
    private String message;

    // Метод getMessage: возвращает нужное значение для текущего контекста.
    public String getMessage() {
        return message;
    }
}
