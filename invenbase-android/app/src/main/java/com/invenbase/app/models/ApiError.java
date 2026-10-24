package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;
import com.google.gson.JsonElement;

/** Единый ответ API при ошибке: code, message, details и request_id. */
public class ApiError {
    @SerializedName("code")
    private String code;

    @SerializedName("message")
    private String message;

    @SerializedName("request_id")
    private String requestId;

    @SerializedName("details")
    private JsonElement details;

    public String getCode() {
        return code;
    }

    public String getMessage() {
        return message;
    }

    public String getRequestId() {
        return requestId;
    }

    public JsonElement getDetails() {
        return details;
    }
}
