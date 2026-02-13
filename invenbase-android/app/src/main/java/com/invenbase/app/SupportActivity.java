package com.invenbase.app;

import android.os.Bundle;
import android.view.View;
import android.widget.EditText;
import android.widget.LinearLayout;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AlertDialog;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.google.android.material.tabs.TabLayout;
import com.invenbase.app.adapters.SupportAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.utils.AuthManager;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class SupportActivity extends BaseActivity implements SupportAdapter.SupportListener {

    private ApiService apiService;
    private AuthManager authManager;
    private RecyclerView recyclerView;
    private ProgressBar progressBar;
    private TextView textRequestsTitle;
    private TextView textEmpty;
    private EditText editSubject;
    private EditText editMessage;
    private SupportAdapter adapter;
    private LinearLayout sectionCreate;
    private LinearLayout sectionRequests;

    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_support);

        apiService = ApiClient.getInstance(this).getApiService();
        authManager = AuthManager.getInstance(this);

        recyclerView = findViewById(R.id.recycler_support_requests);
        progressBar = findViewById(R.id.progress_bar);
        textRequestsTitle = findViewById(R.id.text_requests_title);
        textEmpty = findViewById(R.id.text_support_empty);
        editSubject = findViewById(R.id.edit_support_subject);
        editMessage = findViewById(R.id.edit_support_message);
        sectionCreate = findViewById(R.id.section_create);
        sectionRequests = findViewById(R.id.section_requests);

        setTitle(R.string.support_title);
        if (getSupportActionBar() != null) {
            getSupportActionBar().setDisplayHomeAsUpEnabled(true);
        }

        boolean isAdmin = authManager.isAdmin();
        String currentUserId = authManager.getUser() != null ? authManager.getUser().getId() : "";
        textRequestsTitle.setText(isAdmin ? R.string.support_all_requests : R.string.support_my_requests);

        TabLayout tabs = findViewById(R.id.tabs_support);
        if (isAdmin) {
            // Админ не создаёт заявки — только список «Все заявки»
            tabs.addTab(tabs.newTab().setText(R.string.support_all_requests));
            sectionCreate.setVisibility(View.GONE);
            sectionRequests.setVisibility(View.VISIBLE);
        } else {
            tabs.addTab(tabs.newTab().setText(R.string.support_tab_create));
            tabs.addTab(tabs.newTab().setText(R.string.support_tab_requests));
            tabs.addOnTabSelectedListener(new TabLayout.OnTabSelectedListener() {
                @Override
                public void onTabSelected(TabLayout.Tab tab) {
                    boolean showCreate = (tab.getPosition() == 0);
                    sectionCreate.setVisibility(showCreate ? View.VISIBLE : View.GONE);
                    sectionRequests.setVisibility(showCreate ? View.GONE : View.VISIBLE);
                    if (!showCreate) {
                        loadRequests();
                    }
                }

                @Override
                public void onTabUnselected(TabLayout.Tab tab) {}

                @Override
                public void onTabReselected(TabLayout.Tab tab) {}
            });
        }

        recyclerView.setLayoutManager(new LinearLayoutManager(this));
        adapter = new SupportAdapter(this, isAdmin, currentUserId);
        recyclerView.setAdapter(adapter);

        View submitBtn = findViewById(R.id.button_submit_ticket);
        if (submitBtn != null) {
            submitBtn.setOnClickListener(v -> submitTicket());
        }
        loadRequests();
    }

    @Override
    public boolean onSupportNavigateUp() {
        finish();
        return true;
    }

    private void loadRequests() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getSupportRequests().enqueue(new Callback<List<Map<String, Object>>>() {
            @Override
            public void onResponse(Call<List<Map<String, Object>>> call, Response<List<Map<String, Object>>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful() && response.body() != null) {
                    List<Map<String, Object>> list = response.body();
                    adapter.setItems(list);
                    textEmpty.setVisibility(list.isEmpty() ? View.VISIBLE : View.GONE);
                } else {
                    Toast.makeText(SupportActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<List<Map<String, Object>>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(SupportActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    private void submitTicket() {
        String subject = editSubject.getText() != null ? editSubject.getText().toString().trim() : "";
        String message = editMessage.getText() != null ? editMessage.getText().toString().trim() : "";
        if (subject.isEmpty() || message.isEmpty()) {
            Toast.makeText(this, R.string.fill_required_fields, Toast.LENGTH_SHORT).show();
            return;
        }
        Map<String, String> data = new HashMap<>();
        data.put("subject", subject);
        data.put("message", message);
        progressBar.setVisibility(View.VISIBLE);
        apiService.createSupportRequest(data).enqueue(new Callback<Map<String, Object>>() {
            @Override
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful()) {
                    editSubject.setText("");
                    editMessage.setText("");
                    Toast.makeText(SupportActivity.this, R.string.support_ticket_sent, Toast.LENGTH_SHORT).show();
                    loadRequests();
                    TabLayout tabs = findViewById(R.id.tabs_support);
                    if (tabs != null && tabs.getTabCount() > 1) {
                        tabs.getTabAt(1).select();
                    }
                } else {
                    Toast.makeText(SupportActivity.this, response.code() == 400 ? getString(R.string.support_admin_cannot_create) : getString(R.string.error), Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(SupportActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    @Override
    public void onAddMessageClick(String requestId) {
        EditText input = new EditText(this);
        input.setHint(R.string.support_message_hint);
        input.setMinLines(3);
        input.setPadding(48, 32, 48, 32);
        new AlertDialog.Builder(this)
                .setTitle(R.string.support_add_message)
                .setView(input)
                .setPositiveButton(android.R.string.ok, (dialog, which) -> {
                    String text = input.getText() != null ? input.getText().toString().trim() : "";
                    if (text.isEmpty()) {
                        Toast.makeText(this, R.string.fill_required_fields, Toast.LENGTH_SHORT).show();
                        return;
                    }
                    Map<String, String> data = new HashMap<>();
                    data.put("message", text);
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.addSupportMessage(requestId, data).enqueue(new Callback<Map<String, Object>>() {
                        @Override
                        public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful()) {
                                Toast.makeText(SupportActivity.this, R.string.support_message_added, Toast.LENGTH_SHORT).show();
                                loadRequests();
                            } else {
                                Toast.makeText(SupportActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                            }
                        }

                        @Override
                        public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(SupportActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                        }
                    });
                })
                .setNegativeButton(android.R.string.cancel, null)
                .show();
    }

    @Override
    public void onDeleteRequest(String requestId) {
        new AlertDialog.Builder(this)
                .setTitle(R.string.support_delete_request)
                .setMessage(R.string.support_delete_confirm)
                .setPositiveButton(android.R.string.ok, (dialog, which) -> {
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.deleteSupportRequest(requestId).enqueue(new Callback<Void>() {
                        @Override
                        public void onResponse(Call<Void> call, Response<Void> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful()) {
                                Toast.makeText(SupportActivity.this, R.string.support_deleted, Toast.LENGTH_SHORT).show();
                                loadRequests();
                            } else {
                                Toast.makeText(SupportActivity.this, response.code() == 400 ? R.string.support_delete_only_closed : R.string.error, Toast.LENGTH_SHORT).show();
                            }
                        }

                        @Override
                        public void onFailure(Call<Void> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(SupportActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                        }
                    });
                })
                .setNegativeButton(android.R.string.cancel, null)
                .show();
    }

    @Override
    public void onSendReply(String requestId, String status, String comment) {
        Map<String, Object> data = new HashMap<>();
        data.put("status", status);
        if (comment != null && !comment.isEmpty()) {
            data.put("admin_comment", comment);
        }
        progressBar.setVisibility(View.VISIBLE);
        apiService.updateSupportRequest(requestId, data).enqueue(new Callback<Map<String, Object>>() {
            @Override
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful()) {
                    Toast.makeText(SupportActivity.this, R.string.support_reply_saved, Toast.LENGTH_SHORT).show();
                    loadRequests();
                } else {
                    Toast.makeText(SupportActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(SupportActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }
}
