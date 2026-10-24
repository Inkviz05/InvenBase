package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public final class PairInstanceRequest {
    @SerializedName("setup_code")
    private final String setupCode;

    public PairInstanceRequest(String setupCode) {
        this.setupCode = setupCode;
    }
}
