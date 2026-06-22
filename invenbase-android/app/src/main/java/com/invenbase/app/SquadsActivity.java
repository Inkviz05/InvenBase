package com.invenbase.app;

import android.content.Intent;
import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.widget.ArrayAdapter;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.Spinner;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AlertDialog;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.adapters.SquadsAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.User;
import com.invenbase.app.utils.AuthManager;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class SquadsActivity extends BaseActivity implements SquadsAdapter.OnSquadClickListener {

    public static final String EXTRA_SQUAD_ID = "squad_id";

    private ApiService apiService;
    private AuthManager authManager;
    private RecyclerView recyclerView;
    private ProgressBar progressBar;
    private TextView textEmpty;
    private View buttonAddSquad;
    private SquadsAdapter adapter;
    private List<User> usersForResponsible = new ArrayList<>();

    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_squads);

        apiService = ApiClient.getInstance(this).getApiService();
        authManager = AuthManager.getInstance(this);

        if (!authManager.isAdmin() && !authManager.isResponsible()) {
            Toast.makeText(this, R.string.access_denied, Toast.LENGTH_LONG).show();
            finish();
            return;
        }

        recyclerView = findViewById(R.id.recycler_squads);
        progressBar = findViewById(R.id.progress_bar);
        textEmpty = findViewById(R.id.text_empty_squads);
        buttonAddSquad = findViewById(R.id.button_add_squad);

        recyclerView.setLayoutManager(new LinearLayoutManager(this));
        boolean canEdit = authManager.isAdmin();
        adapter = new SquadsAdapter(this, canEdit);
        recyclerView.setAdapter(adapter);

        buttonAddSquad.setVisibility(canEdit ? View.VISIBLE : View.GONE);
        buttonAddSquad.setOnClickListener(v -> showSquadDialog(null));

        setTitle(R.string.squads);
        if (getSupportActionBar() != null) {
            getSupportActionBar().setDisplayHomeAsUpEnabled(true);
        }

        loadUsers();
        loadSquads();
    }

    @Override
    // Метод onSupportNavigateUp: обрабатывает соответствующее событие приложения.
    public boolean onSupportNavigateUp() {
        finish();
        return true;
    }

    // Метод loadUsers: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadUsers() {
        if (!authManager.isAdmin()) return;
        apiService.getUsers().enqueue(new Callback<List<User>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<List<User>> call, Response<List<User>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    usersForResponsible.clear();
                    for (User u : response.body()) {
                        if ("admin".equals(u.getRole()) || "responsible".equals(u.getRole())) {
                            usersForResponsible.add(u);
                        }
                    }
                }
            }
            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<List<User>> call, Throwable t) {}
        });
    }

    // Метод loadSquads: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadSquads() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getSquads().enqueue(new Callback<List<Map<String, Object>>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<List<Map<String, Object>>> call, Response<List<Map<String, Object>>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful() && response.body() != null) {
                    List<Map<String, Object>> list = response.body();
                    adapter.setItems(list);
                    textEmpty.setVisibility(list.isEmpty() ? View.VISIBLE : View.GONE);
                } else {
                    Toast.makeText(SquadsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }
            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<List<Map<String, Object>>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(SquadsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    // Метод showSquadDialog: выполняет основную бизнес- или UI-логику данного участка кода.
    private void showSquadDialog(@Nullable Map<String, Object> squad) {
        View view = LayoutInflater.from(this).inflate(R.layout.dialog_squad, null, false);
        EditText editName = view.findViewById(R.id.edit_squad_name);
        EditText editLocation = view.findViewById(R.id.edit_squad_location);
        Spinner spinnerResponsible = view.findViewById(R.id.spinner_squad_responsible);
        EditText editDescription = view.findViewById(R.id.edit_squad_description);

        List<String> names = new ArrayList<>();
        names.add(getString(R.string.not_assigned));
        List<String> ids = new ArrayList<>();
        ids.add("");
        for (User u : usersForResponsible) {
            names.add((u.getFullName() != null && !u.getFullName().isEmpty() ? u.getFullName() : u.getUsername()) + " (" + ("admin".equals(u.getRole()) ? "Админ" : "Ответственный") + ")");
            ids.add(u.getId());
        }
        ArrayAdapter<String> spinnerAdapter = new ArrayAdapter<>(this, android.R.layout.simple_spinner_item, names);
        spinnerAdapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
        spinnerResponsible.setAdapter(spinnerAdapter);

        if (squad != null) {
            editName.setText(str(squad.get("name")));
            editLocation.setText(str(squad.get("location")));
            editDescription.setText(str(squad.get("description")));
            String respId = str(squad.get("responsible_user_id"));
            for (int i = 0; i < ids.size(); i++) {
                if (ids.get(i).equals(respId)) {
                    spinnerResponsible.setSelection(i);
                    break;
                }
            }
        }

        AlertDialog dialog = new AlertDialog.Builder(this)
                .setTitle(squad == null ? R.string.add_squad : R.string.edit_squad)
                .setView(view)
                .setCancelable(true)
                .create();

        view.findViewById(R.id.button_squad_cancel).setOnClickListener(v -> dialog.dismiss());
        view.findViewById(R.id.button_squad_save).setOnClickListener(v -> {
            String name = editName.getText().toString().trim();
            if (name.isEmpty()) {
                Toast.makeText(this, R.string.fill_required_fields, Toast.LENGTH_SHORT).show();
                return;
            }
            String location = editLocation.getText().toString().trim();
            String description = editDescription.getText().toString().trim();
            int pos = spinnerResponsible.getSelectedItemPosition();
            String responsibleId = (pos >= 0 && pos < ids.size()) ? ids.get(pos) : "";

            Map<String, Object> data = new HashMap<>();
            data.put("name", name);
            data.put("location", location.isEmpty() ? null : location);
            data.put("description", description.isEmpty() ? null : description);
            data.put("responsible_user_id", responsibleId.isEmpty() ? null : responsibleId);

            dialog.dismiss();
            if (squad != null) {
                updateSquad(str(squad.get("id")), data);
            } else {
                createSquad(data);
            }
        });
        dialog.show();
    }

    // Метод str: выполняет основную бизнес- или UI-логику данного участка кода.
    private static String str(Object o) {
        return o == null ? "" : String.valueOf(o);
    }

    // Метод createSquad: выполняет основную бизнес- или UI-логику данного участка кода.
    private void createSquad(Map<String, Object> data) {
        progressBar.setVisibility(View.VISIBLE);
        apiService.createSquad(data).enqueue(new Callback<Map<String, Object>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful()) {
                    Toast.makeText(SquadsActivity.this, R.string.squad_saved, Toast.LENGTH_SHORT).show();
                    loadSquads();
                } else {
                    Toast.makeText(SquadsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }
            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(SquadsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    // Метод updateSquad: выполняет основную бизнес- или UI-логику данного участка кода.
    private void updateSquad(String id, Map<String, Object> data) {
        progressBar.setVisibility(View.VISIBLE);
        apiService.updateSquad(id, data).enqueue(new Callback<Map<String, Object>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful()) {
                    Toast.makeText(SquadsActivity.this, R.string.squad_saved, Toast.LENGTH_SHORT).show();
                    loadSquads();
                } else {
                    Toast.makeText(SquadsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }
            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(SquadsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    @Override
    // Метод onSquadClick: обрабатывает соответствующее событие приложения.
    public void onSquadClick(Map<String, Object> squad) {
        String id = str(squad.get("id"));
        if (id.isEmpty()) return;
        startActivity(new Intent(this, SquadDetailActivity.class).putExtra(SquadDetailActivity.EXTRA_SQUAD_ID, id));
    }

    @Override
    // Метод onEditClick: обрабатывает соответствующее событие приложения.
    public void onEditClick(Map<String, Object> squad) {
        showSquadDialog(squad);
    }

    @Override
    // Метод onDeleteClick: обрабатывает соответствующее событие приложения.
    public void onDeleteClick(Map<String, Object> squad) {
        String name = str(squad.get("name"));
        EditText input = new EditText(this);
        input.setHint(name);
        input.setMinEms(12);
        new AlertDialog.Builder(this)
                .setTitle(R.string.delete)
                .setMessage(getString(R.string.confirm_delete_squad) + ": " + name)
                .setView(input)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.delete, (d, w) -> {
                    if (!input.getText().toString().trim().equals(name.trim())) {
                        Toast.makeText(this, getString(R.string.confirm_delete_squad), Toast.LENGTH_SHORT).show();
                        return;
                    }
                    String id = str(squad.get("id"));
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.deleteSquad(id).enqueue(new Callback<Void>() {
                        @Override
                        // Метод onResponse: обрабатывает соответствующее событие приложения.
                        public void onResponse(Call<Void> call, Response<Void> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful()) {
                                Toast.makeText(SquadsActivity.this, R.string.squad_deleted, Toast.LENGTH_SHORT).show();
                                loadSquads();
                            } else {
                                Toast.makeText(SquadsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                            }
                        }
                        @Override
                        // Метод onFailure: обрабатывает соответствующее событие приложения.
                        public void onFailure(Call<Void> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(SquadsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                        }
                    });
                })
                .show();
    }
}
