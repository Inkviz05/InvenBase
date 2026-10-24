package com.invenbase.app.api;

import android.content.Context;

import com.invenbase.app.BuildConfig;
import com.invenbase.app.Config;
import com.invenbase.app.utils.SecurePreferences;
import com.invenbase.app.utils.ServerConfiguration;

import java.util.concurrent.TimeUnit;

import okhttp3.OkHttpClient;
import okhttp3.Response;
import okhttp3.logging.HttpLoggingInterceptor;
import retrofit2.Retrofit;
import retrofit2.converter.gson.GsonConverterFactory;

public class ApiClient {
    private static ApiClient instance;

    private final SecurePreferences securePreferences;
    private ApiService apiService;

    private ApiClient(Context context) {
        this.securePreferences = new SecurePreferences(context.getApplicationContext());
        String baseUrl = new ServerConfiguration(context).getBaseUrl();
        if (baseUrl == null) {
            throw new IllegalStateException("InvenBase server is not configured");
        }
        createApiService(baseUrl);
    }

    public static synchronized ApiClient getInstance(Context context) {
        if (instance == null) {
            instance = new ApiClient(context);
        }
        return instance;
    }

    public static synchronized void reset() {
        instance = null;
    }

    public static ApiService createUnauthenticatedService(String baseUrl) {
        Retrofit retrofit = new Retrofit.Builder()
            .baseUrl(normalizeBaseUrl(ServerConfiguration.validateAndNormalizeUrl(baseUrl)))
            .client(new OkHttpClient.Builder()
                .connectTimeout(15, TimeUnit.SECONDS)
                .readTimeout(30, TimeUnit.SECONDS)
                .writeTimeout(30, TimeUnit.SECONDS)
                .build())
            .addConverterFactory(GsonConverterFactory.create())
            .build();
        return retrofit.create(ApiService.class);
    }

    private void createApiService(String baseUrl) {
        HttpLoggingInterceptor loggingInterceptor = new HttpLoggingInterceptor();
        loggingInterceptor.setLevel(BuildConfig.DEBUG
            ? HttpLoggingInterceptor.Level.BODY
            : HttpLoggingInterceptor.Level.NONE);

        OkHttpClient.Builder httpClient = new OkHttpClient.Builder()
            .connectTimeout(15, TimeUnit.SECONDS)
            .readTimeout(30, TimeUnit.SECONDS)
            .writeTimeout(30, TimeUnit.SECONDS);

        httpClient.addInterceptor(loggingInterceptor);
        httpClient.addInterceptor(chain -> {
            String token = securePreferences.getString(Config.PREF_TOKEN);

            if (token == null || chain.request().url().encodedPath().contains("/auth/login")) {
                return chain.proceed(chain.request());
            }

            return chain.proceed(chain.request().newBuilder()
                .header("Authorization", "Bearer " + token)
                .build());
        });

        httpClient.addInterceptor(chain -> {
            Response response = chain.proceed(chain.request());
            String path = chain.request().url().encodedPath();
            if (response.code() == 401
                    && !path.contains("/auth/login")
                    && !path.contains("/instance/pair")) {
                clearAuthState();
            }
            return response;
        });

        Retrofit retrofit = new Retrofit.Builder()
            .baseUrl(normalizeBaseUrl(baseUrl))
            .client(httpClient.build())
            .addConverterFactory(GsonConverterFactory.create())
            .build();

        apiService = retrofit.create(ApiService.class);
    }

    public ApiService getApiService() {
        return apiService;
    }

    private static String normalizeBaseUrl(String baseUrl) {
        return baseUrl.endsWith("/") ? baseUrl : baseUrl + "/";
    }

    private void clearAuthState() {
        securePreferences.remove(Config.PREF_TOKEN);
        securePreferences.remove(Config.PREF_USER);
    }
}
