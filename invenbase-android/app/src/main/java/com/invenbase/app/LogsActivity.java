package com.invenbase.app;

import android.os.Bundle;
import android.view.View;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.adapters.LogsAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.utils.AuthManager;

import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class LogsActivity extends BaseActivity {

    private ApiService apiService;
    private AuthManager authManager;
    private RecyclerView recyclerView;
    private ProgressBar progressBar;
    private TextView emptyView;
    private LogsAdapter adapter;

    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_logs);

        apiService = ApiClient.getInstance(this).getApiService();
        authManager = AuthManager.getInstance(this);

        if (!authManager.isAdmin() && !authManager.isResponsible()) {
            Toast.makeText(this, R.string.access_denied, Toast.LENGTH_LONG).show();
            finish();
            return;
        }

        recyclerView = findViewById(R.id.recycler_logs);
        progressBar = findViewById(R.id.progress_bar);
        emptyView = findViewById(R.id.text_empty);

        recyclerView.setLayoutManager(new LinearLayoutManager(this));
        adapter = new LogsAdapter();
        recyclerView.setAdapter(adapter);

        setTitle(R.string.logs);
        loadLogs();
    }

    // Метод loadLogs: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadLogs() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getLogs().enqueue(new Callback<List<Map<String, Object>>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<List<Map<String, Object>>> call, Response<List<Map<String, Object>>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful() && response.body() != null) {
                    List<Map<String, Object>> list = response.body();
                    adapter.setItems(list);
                    emptyView.setVisibility(list.isEmpty() ? View.VISIBLE : View.GONE);
                } else {
                    Toast.makeText(LogsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<List<Map<String, Object>>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(LogsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }
}
