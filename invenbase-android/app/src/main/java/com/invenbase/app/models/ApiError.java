package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

/** Ответ сервера при ошибке (400, 404 и т.д.): {"error":"...", "message":"..."} */
public class ApiError {
    @SerializedName("message")
    private String message;

    public String getMessage() {
        return message;
    }
}
