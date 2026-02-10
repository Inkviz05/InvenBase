package com.invenbase.app;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.Toast;


import com.google.firebase.messaging.FirebaseMessaging;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.LoginRequest;
import com.invenbase.app.models.LoginResponse;
import com.invenbase.app.utils.AuthManager;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class LoginActivity extends BaseActivity {
    private EditText editUsername;
    private EditText editPassword;
    private Button buttonLogin;
    private ProgressBar progressBar;
    private ApiService apiService;
    private AuthManager authManager;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_login);

        authManager = AuthManager.getInstance(this);

        // Если уже авторизован, переходим в главное приложение
        if (authManager.isAuthenticated()) {
            startActivity(new Intent(this, MainActivity.class));
            finish();
            return;
        }

        // ApiService создаём при первом нажатии «Войти», чтобы не блокировать главный поток при старте
        apiService = null;

        editUsername = findViewById(R.id.edit_username);
        editPassword = findViewById(R.id.edit_password);
        buttonLogin = findViewById(R.id.button_login);
        progressBar = findViewById(R.id.progress_bar);

        buttonLogin.setOnClickListener(v -> performLogin());
    }

    private ApiService getApiService() {
        if (apiService == null) {
            apiService = ApiClient.getInstance(this).getApiService();
        }
        return apiService;
    }

    private void performLogin() {
        String username = editUsername.getText().toString().trim();
        String password = editPassword.getText().toString();

        if (username.isEmpty() || password.isEmpty()) {
            Toast.makeText(this, R.string.login_error, Toast.LENGTH_SHORT).show();
            return;
        }

        buttonLogin.setEnabled(false);
        progressBar.setVisibility(View.VISIBLE);

        LoginRequest request = new LoginRequest(username, password);
        Call<LoginResponse> call = getApiService().login(request);

        call.enqueue(new Callback<LoginResponse>() {
            @Override
            public void onResponse(Call<LoginResponse> call, Response<LoginResponse> response) {
                buttonLogin.setEnabled(true);
                progressBar.setVisibility(View.GONE);

                if (response.isSuccessful() && response.body() != null) {
                    LoginResponse loginResponse = response.body();
                    authManager.saveToken(loginResponse.getToken());
                    authManager.saveUser(loginResponse.getUser());

                    // Отправляем FCM-токен на сервер, если он уже есть
                    android.util.Log.d("FCM", "Login successful, checking for stored token...");
                    com.invenbase.app.utils.PushManager.getInstance(LoginActivity.this)
                            .sendStoredTokenIfNeeded();

                    // На всякий случай запрашиваем актуальный токен
                    android.util.Log.d("FCM", "Requesting FCM token from Firebase...");
                    FirebaseMessaging.getInstance().getToken()
                            .addOnCompleteListener(task -> {
                                if (task.isSuccessful() && task.getResult() != null) {
                                    String token = task.getResult();
                                    android.util.Log.d("FCM", "FCM token received: " + token.substring(0, Math.min(30, token.length())) + "...");
                                    com.invenbase.app.utils.PushManager.getInstance(LoginActivity.this)
                                            .registerToken(token);
                                } else {
                                    android.util.Log.e("FCM", "Failed to get FCM token: " + (task.getException() != null ? task.getException().getMessage() : "unknown error"));
                                }
                            });

                    Toast.makeText(LoginActivity.this, R.string.success, Toast.LENGTH_SHORT).show();
                    startActivity(new Intent(LoginActivity.this, MainActivity.class));
                    finish();
                } else {
                    Toast.makeText(LoginActivity.this, R.string.login_error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<LoginResponse> call, Throwable t) {
                buttonLogin.setEnabled(true);
                progressBar.setVisibility(View.GONE);
                Toast.makeText(LoginActivity.this, R.string.error + ": " + t.getMessage(), Toast.LENGTH_SHORT).show();
            }
        });
    }
}
