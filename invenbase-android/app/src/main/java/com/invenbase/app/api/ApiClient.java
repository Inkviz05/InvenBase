package com.invenbase.app.api;

import android.content.Context;
import android.content.SharedPreferences;

import com.invenbase.app.BuildConfig;
import com.invenbase.app.Config;

import java.util.concurrent.TimeUnit;

import okhttp3.OkHttpClient;
import okhttp3.Response;
import okhttp3.logging.HttpLoggingInterceptor;
import retrofit2.Retrofit;
import retrofit2.converter.gson.GsonConverterFactory;

public class ApiClient {
    private static ApiClient instance;

    private final Context context;
    private ApiService apiService;

    private ApiClient(Context context) {
        this.context = context.getApplicationContext();
        createApiService();
    }

    public static synchronized ApiClient getInstance(Context context) {
        if (instance == null) {
            instance = new ApiClient(context);
        }
        return instance;
    }

    private void createApiService() {
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
            SharedPreferences prefs = context.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
            String token = prefs.getString(Config.PREF_TOKEN, null);

            if (token == null || chain.request().url().encodedPath().contains("/auth/login")) {
                return chain.proceed(chain.request());
            }

            return chain.proceed(chain.request().newBuilder()
                .header("Authorization", "Bearer " + token)
                .build());
        });

        httpClient.addInterceptor(chain -> {
            Response response = chain.proceed(chain.request());
            if (response.code() == 401 && !chain.request().url().encodedPath().contains("/auth/login")) {
                clearAuthState();
            }
            return response;
        });

        Retrofit retrofit = new Retrofit.Builder()
            .baseUrl(normalizeBaseUrl(getConfiguredApiUrl()))
            .client(httpClient.build())
            .addConverterFactory(GsonConverterFactory.create())
            .build();

        apiService = retrofit.create(ApiService.class);
    }

    public ApiService getApiService() {
        return apiService;
    }

    public void updateApiUrl(String newUrl) {
        SharedPreferences prefs = context.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
        prefs.edit().putString(Config.PREF_API_URL, newUrl).apply();
        createApiService();
    }

    private String getConfiguredApiUrl() {
        SharedPreferences prefs = context.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
        String apiUrl = prefs.getString(Config.PREF_API_URL, Config.DEFAULT_API_URL);
        if (apiUrl == null || apiUrl.trim().isEmpty()) {
            return Config.DEFAULT_API_URL;
        }
        return apiUrl.trim();
    }

    private String normalizeBaseUrl(String baseUrl) {
        return baseUrl.endsWith("/") ? baseUrl : baseUrl + "/";
    }

    private void clearAuthState() {
        SharedPreferences prefs = context.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
        prefs.edit()
            .remove(Config.PREF_TOKEN)
            .remove(Config.PREF_USER)
            .apply();
    }
}
