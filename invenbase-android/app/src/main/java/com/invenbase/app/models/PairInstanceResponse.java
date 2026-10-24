package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public final class PairInstanceResponse {
    @SerializedName("instance")
    private InstanceInfo instance;

    public InstanceInfo getInstance() { return instance; }
}
