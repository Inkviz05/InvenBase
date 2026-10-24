package com.invenbase.app.utils;

import android.content.Context;
import android.net.Uri;

import com.invenbase.app.BuildConfig;
import com.invenbase.app.Config;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.models.InstanceInfo;

import java.util.Locale;

/** Stores the selected self-hosted InvenBase instance in Keystore-backed preferences. */
public final class ServerConfiguration {
    public static final int SUPPORTED_API_PROTOCOL_VERSION = 1;

    private final Context appContext;
    private final SecurePreferences preferences;

    public ServerConfiguration(Context context) {
        appContext = context.getApplicationContext();
        preferences = new SecurePreferences(appContext);
    }

    public boolean isConfigured() {
        return getBaseUrl() != null && preferences.getString(Config.PREF_INSTANCE_ID) != null;
    }

    public String getBaseUrl() {
        return preferences.getString(Config.PREF_SERVER_URL);
    }

    public String getInstanceName() {
        return preferences.getString(Config.PREF_INSTANCE_NAME);
    }

    public void save(String baseUrl, InstanceInfo instance) {
        String normalized = validateAndNormalizeUrl(baseUrl);
        preferences.putString(Config.PREF_SERVER_URL, normalized);
        preferences.putString(Config.PREF_INSTANCE_ID, instance.getInstanceId());
        preferences.putString(Config.PREF_INSTANCE_NAME, instance.getInstanceName());
        ApiClient.reset();
    }

    public void clear() {
        preferences.remove(Config.PREF_SERVER_URL);
        preferences.remove(Config.PREF_INSTANCE_ID);
        preferences.remove(Config.PREF_INSTANCE_NAME);
        ApiClient.reset();
    }

    /** Keeps already signed-in users on the server used by builds predating onboarding. */
    public void migrateLegacyAuthenticatedInstall(AuthManager authManager) {
        if (!isConfigured() && authManager.isAuthenticated()) {
            preferences.putString(Config.PREF_SERVER_URL, validateAndNormalizeUrl(BuildConfig.API_BASE_URL));
            preferences.putString(Config.PREF_INSTANCE_ID, "legacy-unverified");
            preferences.putString(Config.PREF_INSTANCE_NAME, "InvenBase");
        }
    }

    public static String validateAndNormalizeUrl(String value) {
        if (value == null || value.trim().isEmpty()) {
            throw new IllegalArgumentException("Server URL is required");
        }
        Uri uri = Uri.parse(value.trim());
        String scheme = uri.getScheme() == null ? "" : uri.getScheme().toLowerCase(Locale.ROOT);
        if (uri.getHost() == null || !("https".equals(scheme) || "http".equals(scheme))) {
            throw new IllegalArgumentException("Enter an absolute HTTP(S) server URL");
        }
        if (!BuildConfig.DEBUG && !"https".equals(scheme)) {
            throw new IllegalArgumentException("Release builds require an HTTPS server");
        }
        String path = uri.getPath();
        if (path != null && !path.isEmpty() && !"/".equals(path)) {
            throw new IllegalArgumentException("Enter the server address without /api or another path");
        }
        if (uri.getQuery() != null || uri.getFragment() != null || uri.getUserInfo() != null) {
            throw new IllegalArgumentException("Server URL must not contain credentials, query, or fragment");
        }
        Uri.Builder builder = new Uri.Builder()
                .scheme(scheme)
                .encodedAuthority(uri.getEncodedAuthority());
        return builder.build().toString();
    }
}
