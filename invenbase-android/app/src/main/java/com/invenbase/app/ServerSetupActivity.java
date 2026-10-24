package com.invenbase.app;

import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.Toast;

import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.InstanceInfo;
import com.invenbase.app.models.PairInstanceRequest;
import com.invenbase.app.models.PairInstanceResponse;
import com.invenbase.app.utils.ApiErrorParser;
import com.invenbase.app.utils.ServerConfiguration;
import com.invenbase.app.utils.SetupLink;
import com.invenbase.app.utils.AuthManager;
import com.journeyapps.barcodescanner.ScanContract;
import com.journeyapps.barcodescanner.ScanOptions;

import androidx.activity.result.ActivityResultLauncher;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public final class ServerSetupActivity extends BaseActivity {
    private EditText serverUrl;
    private EditText setupCode;
    private Button connectButton;
    private ProgressBar progressBar;
    private final ActivityResultLauncher<ScanOptions> qrScanner =
            registerForActivityResult(new ScanContract(), result -> {
                if (result.getContents() == null) return;
                if (!applySetupLink(result.getContents())) {
                    Toast.makeText(this, R.string.invalid_server_setup_qr, Toast.LENGTH_LONG).show();
                }
            });

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_server_setup);

        serverUrl = findViewById(R.id.edit_server_url);
        setupCode = findViewById(R.id.edit_setup_code);
        connectButton = findViewById(R.id.button_connect_server);
        progressBar = findViewById(R.id.server_setup_progress);

        if (BuildConfig.DEBUG) {
            serverUrl.setText(BuildConfig.API_BASE_URL);
        }
        connectButton.setOnClickListener(view -> connect());
        findViewById(R.id.button_scan_server_qr).setOnClickListener(view -> {
            ScanOptions options = new ScanOptions();
            options.setDesiredBarcodeFormats(ScanOptions.QR_CODE);
            options.setPrompt(getString(R.string.scan_server_setup_qr));
            options.setBeepEnabled(true);
            options.setOrientationLocked(false);
            qrScanner.launch(options);
        });
        if (getIntent() != null && getIntent().getData() != null) {
            if (!applySetupLink(getIntent().getData().toString())) {
                Toast.makeText(this, R.string.invalid_server_setup_link, Toast.LENGTH_LONG).show();
            }
        }
    }

    private void connect() {
        final String normalizedUrl;
        try {
            normalizedUrl = ServerConfiguration.validateAndNormalizeUrl(
                    serverUrl.getText().toString());
        } catch (IllegalArgumentException error) {
            serverUrl.setError(error.getMessage());
            return;
        }
        String code = setupCode.getText().toString().trim();
        if (code.isEmpty()) {
            setupCode.setError(getString(R.string.server_setup_code_required));
            return;
        }

        setLoading(true);
        ApiService service = ApiClient.createUnauthenticatedService(normalizedUrl);
        service.getInstanceInfo().enqueue(new Callback<InstanceInfo>() {
            @Override
            public void onResponse(Call<InstanceInfo> call, Response<InstanceInfo> response) {
                if (!response.isSuccessful() || !isCompatible(response.body())) {
                    setLoading(false);
                    String message = response.isSuccessful()
                            ? getString(R.string.server_incompatible)
                            : ApiErrorParser.fromResponse(ServerSetupActivity.this, response);
                    Toast.makeText(ServerSetupActivity.this, message, Toast.LENGTH_LONG).show();
                    return;
                }
                pair(service, normalizedUrl, code, response.body());
            }

            @Override
            public void onFailure(Call<InstanceInfo> call, Throwable throwable) {
                showFailure(throwable);
            }
        });
    }

    private void pair(ApiService service, String baseUrl, String code, InstanceInfo discovered) {
        service.pairInstance(new PairInstanceRequest(code)).enqueue(new Callback<PairInstanceResponse>() {
            @Override
            public void onResponse(Call<PairInstanceResponse> call,
                                   Response<PairInstanceResponse> response) {
                setLoading(false);
                PairInstanceResponse body = response.body();
                if (!response.isSuccessful() || body == null || !isCompatible(body.getInstance())
                        || !discovered.getInstanceId().equals(body.getInstance().getInstanceId())) {
                    String message = response.isSuccessful()
                            ? getString(R.string.server_incompatible)
                            : ApiErrorParser.fromResponse(ServerSetupActivity.this, response);
                    Toast.makeText(ServerSetupActivity.this, message, Toast.LENGTH_LONG).show();
                    return;
                }

                AuthManager.getInstance(ServerSetupActivity.this).logout();
                new ServerConfiguration(ServerSetupActivity.this).save(baseUrl, body.getInstance());
                Toast.makeText(ServerSetupActivity.this,
                        getString(R.string.server_connected, body.getInstance().getInstanceName()),
                        Toast.LENGTH_LONG).show();
                Intent login = new Intent(ServerSetupActivity.this, LoginActivity.class);
                login.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK | Intent.FLAG_ACTIVITY_CLEAR_TASK);
                startActivity(login);
            }

            @Override
            public void onFailure(Call<PairInstanceResponse> call, Throwable throwable) {
                showFailure(throwable);
            }
        });
    }

    private boolean isCompatible(InstanceInfo instance) {
        return instance != null
                && "InvenBase".equals(instance.getProduct())
                && instance.getInstanceId() != null
                && instance.getInstanceName() != null
                && "/api".equals(instance.getApiBasePath())
                && instance.getApiProtocolVersion()
                    == ServerConfiguration.SUPPORTED_API_PROTOCOL_VERSION;
    }

    private void showFailure(Throwable throwable) {
        setLoading(false);
        Toast.makeText(this, ApiErrorParser.fromThrowable(this, throwable), Toast.LENGTH_LONG).show();
    }

    private void setLoading(boolean loading) {
        connectButton.setEnabled(!loading);
        serverUrl.setEnabled(!loading);
        setupCode.setEnabled(!loading);
        progressBar.setVisibility(loading ? View.VISIBLE : View.GONE);
    }

    private boolean applySetupLink(String value) {
        SetupLink link = SetupLink.parse(value);
        if (link == null) return false;
        serverUrl.setText(link.getServerUrl());
        setupCode.setText(link.getSetupCode());
        return true;
    }
}
