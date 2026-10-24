package com.invenbase.app.api;

import android.content.Context;

import com.invenbase.app.BuildConfig;
import com.invenbase.app.Config;
import com.invenbase.app.utils.SecurePreferences;

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
        context.getApplicationContext()
                .getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE)
                .edit()
                .remove("api_url")
                .apply();
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
            if (response.code() == 401 && !chain.request().url().encodedPath().contains("/auth/login")) {
                clearAuthState();
            }
            return response;
        });

        Retrofit retrofit = new Retrofit.Builder()
            .baseUrl(normalizeBaseUrl(BuildConfig.API_BASE_URL))
            .client(httpClient.build())
            .addConverterFactory(GsonConverterFactory.create())
            .build();

        apiService = retrofit.create(ApiService.class);
    }

    public ApiService getApiService() {
        return apiService;
    }

    private String normalizeBaseUrl(String baseUrl) {
        return baseUrl.endsWith("/") ? baseUrl : baseUrl + "/";
    }

    private void clearAuthState() {
        securePreferences.remove(Config.PREF_TOKEN);
        securePreferences.remove(Config.PREF_USER);
    }
}
