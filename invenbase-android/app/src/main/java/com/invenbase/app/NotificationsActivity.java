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

import com.invenbase.app.adapters.NotificationsAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;

import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class NotificationsActivity extends BaseActivity implements NotificationsAdapter.OnNotificationActionListener {

    private ApiService apiService;
    private RecyclerView recyclerView;
    private ProgressBar progressBar;
    private TextView emptyView;
    private NotificationsAdapter adapter;

    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_notifications);

        apiService = ApiClient.getInstance(this).getApiService();

        recyclerView = findViewById(R.id.recycler_notifications);
        progressBar = findViewById(R.id.progress_bar);
        emptyView = findViewById(R.id.text_empty);

        recyclerView.setLayoutManager(new LinearLayoutManager(this));
        adapter = new NotificationsAdapter(this);
        recyclerView.setAdapter(adapter);

        setTitle(R.string.notifications);
        loadNotifications();
    }

    // Метод loadNotifications: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadNotifications() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getNotifications().enqueue(new Callback<List<Map<String, Object>>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<List<Map<String, Object>>> call, Response<List<Map<String, Object>>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful() && response.body() != null) {
                    List<Map<String, Object>> list = response.body();
                    adapter.setItems(list);
                    emptyView.setVisibility(list.isEmpty() ? View.VISIBLE : View.GONE);
                } else {
                    Toast.makeText(NotificationsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<List<Map<String, Object>>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(NotificationsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    @Override
    // Метод onMarkAsRead: обрабатывает соответствующее событие приложения.
    public void onMarkAsRead(String id) {
        apiService.markAsRead(id).enqueue(new Callback<Void>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Void> call, Response<Void> response) {
                loadNotifications();
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Void> call, Throwable t) {
                Toast.makeText(NotificationsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }
}

