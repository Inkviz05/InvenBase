package com.invenbase.app.utils;

import android.content.Context;

import com.google.gson.Gson;
import com.invenbase.app.R;
import com.invenbase.app.models.ApiError;

import java.io.IOException;

import retrofit2.Response;

public final class ApiErrorParser {
    private ApiErrorParser() {
    }

    public static String fromResponse(Context context, Response<?> response) {
        if (response == null) {
            return context.getString(R.string.error);
        }

        if (response.errorBody() != null) {
            try {
                ApiError error = new Gson().fromJson(response.errorBody().string(), ApiError.class);
                if (error != null && error.getMessage() != null && !error.getMessage().trim().isEmpty()) {
                    return error.getMessage().trim();
                }
            } catch (IOException | RuntimeException ignored) {
                // Fall back to HTTP status below.
            }
        }

        String message = response.message();
        if (message != null && !message.trim().isEmpty()) {
            return message.trim();
        }

        return context.getString(R.string.error);
    }

    public static String fromThrowable(Context context, Throwable throwable) {
        if (throwable != null && throwable.getMessage() != null && !throwable.getMessage().trim().isEmpty()) {
            return throwable.getMessage().trim();
        }
        return context.getString(R.string.error);
    }
}
