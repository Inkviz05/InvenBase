package com.invenbase.app.utils;

import android.net.Uri;

public final class SetupLink {
    public static final String SCHEME = "invenbase";
    public static final String HOST = "pair";

    private final String serverUrl;
    private final String setupCode;

    private SetupLink(String serverUrl, String setupCode) {
        this.serverUrl = serverUrl;
        this.setupCode = setupCode;
    }

    public String getServerUrl() { return serverUrl; }
    public String getSetupCode() { return setupCode; }

    public static SetupLink parse(String value) {
        if (value == null) return null;
        Uri uri = Uri.parse(value.trim());
        if (!SCHEME.equalsIgnoreCase(uri.getScheme())
                || !HOST.equalsIgnoreCase(uri.getHost())) {
            return null;
        }
        String server = uri.getQueryParameter("server");
        String code = uri.getQueryParameter("code");
        if (server == null || server.trim().isEmpty() || code == null || code.trim().isEmpty()) {
            return null;
        }
        try {
            server = ServerConfiguration.validateAndNormalizeUrl(server);
        } catch (IllegalArgumentException ignored) {
            return null;
        }
        return new SetupLink(server, code.trim());
    }
}
