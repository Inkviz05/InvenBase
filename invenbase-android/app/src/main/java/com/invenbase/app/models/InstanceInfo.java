package com.invenbase.app.models;

import com.google.gson.annotations.SerializedName;

public final class InstanceInfo {
    @SerializedName("product")
    private String product;
    @SerializedName("instance_id")
    private String instanceId;
    @SerializedName("instance_name")
    private String instanceName;
    @SerializedName("api_protocol_version")
    private int apiProtocolVersion;
    @SerializedName("api_base_path")
    private String apiBasePath;

    public String getProduct() { return product; }
    public String getInstanceId() { return instanceId; }
    public String getInstanceName() { return instanceName; }
    public int getApiProtocolVersion() { return apiProtocolVersion; }
    public String getApiBasePath() { return apiBasePath; }
}
