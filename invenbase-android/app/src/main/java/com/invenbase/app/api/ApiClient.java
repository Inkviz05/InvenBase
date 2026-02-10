package com.invenbase.app.api;

import android.content.Context;
import android.content.SharedPreferences;

import com.invenbase.app.Config;

import okhttp3.OkHttpClient;
import okhttp3.logging.HttpLoggingInterceptor;
import retrofit2.Retrofit;
import retrofit2.converter.gson.GsonConverterFactory;

public class ApiClient {
    private static ApiClient instance;
    private ApiService apiService;
    private Context context;

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
        loggingInterceptor.setLevel(HttpLoggingInterceptor.Level.BODY);

        OkHttpClient.Builder httpClient = new OkHttpClient.Builder();
        httpClient.addInterceptor(loggingInterceptor);
        
        // Добавляем токен авторизации к каждому запросу
        httpClient.addInterceptor(chain -> {
            SharedPreferences prefs = context.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
            String token = prefs.getString(Config.PREF_TOKEN, null);
            
            if (token != null && !chain.request().url().encodedPath().contains("/auth/login")) {
                return chain.proceed(chain.request().newBuilder()
                    .header("Authorization", "Bearer " + token)
                    .build());
            }
            return chain.proceed(chain.request());
        });

        // Получаем базовый URL из настроек или используем дефолтный
        SharedPreferences prefs = context.getSharedPreferences(Config.PREF_NAME, Context.MODE_PRIVATE);
        String apiUrl = prefs.getString(Config.PREF_API_URL, Config.DEFAULT_API_URL);
        String baseUrl = apiUrl;
        if (baseUrl == null || baseUrl.trim().isEmpty()) {
            baseUrl = Config.DEFAULT_API_URL;
        }
        baseUrl = baseUrl.trim();
        // Retrofit требует завершающий слэш
        if (!baseUrl.endsWith("/")) {
            baseUrl = baseUrl + "/";
        }

        Retrofit retrofit = new Retrofit.Builder()
            .baseUrl(baseUrl)
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
        createApiService(); // Пересоздаем сервис с новым URL
    }
}
