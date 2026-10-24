package com.invenbase.app;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.Toast;
import android.widget.TextView;


import ru.rustore.sdk.pushclient.RuStorePushClient;
import com.invenbase.app.notifications.NotificationSyncScheduler;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.LoginRequest;
import com.invenbase.app.models.LoginResponse;
import com.invenbase.app.utils.ApiErrorParser;
import com.invenbase.app.utils.AuthManager;
import com.invenbase.app.utils.ServerConfiguration;

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
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_login);

        authManager = AuthManager.getInstance(this);
        ServerConfiguration serverConfiguration = new ServerConfiguration(this);
        serverConfiguration.migrateLegacyAuthenticatedInstall(authManager);

        if (!serverConfiguration.isConfigured()) {
            startActivity(new Intent(this, ServerSetupActivity.class));
            finish();
            return;
        }

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
        TextView serverName = findViewById(R.id.text_server_name);
        serverName.setText(getString(R.string.connected_server,
                serverConfiguration.getInstanceName()));
        findViewById(R.id.button_change_server).setOnClickListener(view -> {
            authManager.logout();
            serverConfiguration.clear();
            Intent setup = new Intent(this, ServerSetupActivity.class);
            setup.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TASK);
            startActivity(setup);
        });

        buttonLogin.setOnClickListener(v -> performLogin());
    }

    // Метод getApiService: возвращает нужное значение для текущего контекста.
    private ApiService getApiService() {
        if (apiService == null) {
            apiService = ApiClient.getInstance(this).getApiService();
        }
        return apiService;
    }

    // Метод performLogin: выполняет основную бизнес- или UI-логику данного участка кода.
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
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<LoginResponse> call, Response<LoginResponse> response) {
                buttonLogin.setEnabled(true);
                progressBar.setVisibility(View.GONE);

                if (response.isSuccessful() && response.body() != null) {
                    LoginResponse loginResponse = response.body();
                    authManager.saveToken(loginResponse.getToken());
                    authManager.saveUser(loginResponse.getUser());
                    NotificationSyncScheduler.enqueueImmediate(LoginActivity.this);

                    // Отправляем уже полученные RuStore/Huawei токены на сервер.
                    com.invenbase.app.utils.PushManager.getInstance(LoginActivity.this)
                            .sendStoredTokenIfNeeded();

                    if (!BuildConfig.RUSTORE_PROJECT_ID.isEmpty()) {
                        RuStorePushClient.INSTANCE.getToken()
                                .addOnSuccessListener(token ->
                                        com.invenbase.app.utils.PushManager
                                                .getInstance(LoginActivity.this)
                                                .registerToken(
                                                        com.invenbase.app.utils.PushManager.PROVIDER_RUSTORE,
                                                        token))
                                .addOnFailureListener(error -> android.util.Log.w(
                                        "RuStorePush",
                                        "Failed to get RuStore token after login",
                                        error));
                    }

                    Toast.makeText(LoginActivity.this, R.string.success, Toast.LENGTH_SHORT).show();
                    startActivity(new Intent(LoginActivity.this, MainActivity.class));
                    finish();
                } else {
                    Toast.makeText(LoginActivity.this, ApiErrorParser.fromResponse(LoginActivity.this, response), Toast.LENGTH_LONG).show();
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<LoginResponse> call, Throwable t) {
                buttonLogin.setEnabled(true);
                progressBar.setVisibility(View.GONE);
                Toast.makeText(LoginActivity.this, ApiErrorParser.fromThrowable(LoginActivity.this, t), Toast.LENGTH_LONG).show();
            }
        });
    }
}
