package com.invenbase.app;

import android.app.AlertDialog;
import android.os.Bundle;
import android.text.TextUtils;
import android.view.LayoutInflater;
import android.view.View;
import android.widget.ArrayAdapter;
import android.widget.Button;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.Spinner;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.adapters.UsersAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.User;
import com.invenbase.app.utils.AuthManager;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class UsersActivity extends BaseActivity implements UsersAdapter.OnUserActionListener {

    private ApiService apiService;
    private AuthManager authManager;
    private RecyclerView recyclerView;
    private ProgressBar progressBar;
    private TextView emptyView;
    private UsersAdapter adapter;

    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_users);

        apiService = ApiClient.getInstance(this).getApiService();
        authManager = AuthManager.getInstance(this);

        if (!authManager.isAdmin()) {
            Toast.makeText(this, R.string.access_denied, Toast.LENGTH_LONG).show();
            finish();
            return;
        }

        recyclerView = findViewById(R.id.recycler_users);
        progressBar = findViewById(R.id.progress_bar);
        emptyView = findViewById(R.id.text_empty);
        Button buttonAdd = findViewById(R.id.button_add_user);

        recyclerView.setLayoutManager(new LinearLayoutManager(this));
        adapter = new UsersAdapter(this, authManager.getUser());
        recyclerView.setAdapter(adapter);

        buttonAdd.setOnClickListener(v -> showUserDialog(null));

        setTitle(R.string.users);
        loadUsers();
    }

    // Метод loadUsers: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadUsers() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getUsers().enqueue(new Callback<List<User>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<List<User>> call, Response<List<User>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful() && response.body() != null) {
                    List<User> list = response.body();
                    adapter.setItems(list);
                    emptyView.setVisibility(list.isEmpty() ? View.VISIBLE : View.GONE);
                } else {
                    Toast.makeText(UsersActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<List<User>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(UsersActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    // Метод showUserDialog: выполняет основную бизнес- или UI-логику данного участка кода.
    private void showUserDialog(@Nullable User user) {
        View view = LayoutInflater.from(this).inflate(R.layout.dialog_user, null, false);
        EditText editUsername = view.findViewById(R.id.edit_username);
        EditText editPassword = view.findViewById(R.id.edit_password);
        EditText editFullName = view.findViewById(R.id.edit_full_name);
        EditText editEmail = view.findViewById(R.id.edit_email);
        Spinner spinnerRole = view.findViewById(R.id.spinner_role);

        ArrayAdapter<CharSequence> roleAdapter = ArrayAdapter.createFromResource(
                this, R.array.user_roles, android.R.layout.simple_spinner_item);
        roleAdapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
        spinnerRole.setAdapter(roleAdapter);

        if (user != null) {
            editUsername.setText(user.getUsername());
            editFullName.setText(user.getFullName());
            editEmail.setText(user.getEmail());
            int rolePos = user.getRole().equals("admin") ? 2 : user.getRole().equals("responsible") ? 1 : 0;
            spinnerRole.setSelection(rolePos);
        }

        new AlertDialog.Builder(this)
                .setTitle(user == null ? R.string.add_user : R.string.edit_user)
                .setView(view)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.save, (dialog, which) -> {
                    String username = editUsername.getText().toString().trim();
                    String password = editPassword.getText().toString();
                    String fullName = editFullName.getText().toString().trim();
                    String email = editEmail.getText().toString().trim();
                    String role = spinnerRole.getSelectedItemPosition() == 2 ? "admin" :
                            spinnerRole.getSelectedItemPosition() == 1 ? "responsible" : "user";

                    if (TextUtils.isEmpty(username) || (user == null && TextUtils.isEmpty(password))) {
                        Toast.makeText(this, R.string.fill_required_fields, Toast.LENGTH_SHORT).show();
                        return;
                    }

                    Map<String, Object> data = new HashMap<>();
                    data.put("username", username);
                    if (!TextUtils.isEmpty(password)) {
                        data.put("password", password);
                    }
                    data.put("full_name", fullName.isEmpty() ? null : fullName);
                    data.put("email", email.isEmpty() ? null : email);
                    data.put("role", role);

                    if (user == null) {
                        createUser(data);
                    } else {
                        updateUser(user.getId(), data);
                    }
                })
                .show();
    }

    // Метод createUser: выполняет основную бизнес- или UI-логику данного участка кода.
    private void createUser(Map<String, Object> data) {
        progressBar.setVisibility(View.VISIBLE);
        apiService.createUser(data).enqueue(new Callback<User>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<User> call, Response<User> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful()) {
                    Toast.makeText(UsersActivity.this, R.string.user_created, Toast.LENGTH_SHORT).show();
                } else {
                    Toast.makeText(UsersActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
                loadUsers();
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<User> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(UsersActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    // Метод updateUser: выполняет основную бизнес- или UI-логику данного участка кода.
    private void updateUser(String id, Map<String, Object> data) {
        progressBar.setVisibility(View.VISIBLE);
        apiService.updateUser(id, data).enqueue(new Callback<User>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<User> call, Response<User> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful()) {
                    Toast.makeText(UsersActivity.this, R.string.user_updated, Toast.LENGTH_SHORT).show();
                } else {
                    Toast.makeText(UsersActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
                loadUsers();
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<User> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(UsersActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    // Метод deleteUser: выполняет основную бизнес- или UI-логику данного участка кода.
    private void deleteUser(String id) {
        new AlertDialog.Builder(this)
                .setMessage(R.string.confirm_delete_user)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.delete, (dialog, which) -> {
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.deleteUser(id).enqueue(new Callback<Void>() {
                        @Override
                        // Метод onResponse: обрабатывает соответствующее событие приложения.
                        public void onResponse(Call<Void> call, Response<Void> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful()) {
                                Toast.makeText(UsersActivity.this, R.string.user_deleted, Toast.LENGTH_SHORT).show();
                            } else {
                                Toast.makeText(UsersActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                            }
                            loadUsers();
                        }

                        @Override
                        // Метод onFailure: обрабатывает соответствующее событие приложения.
                        public void onFailure(Call<Void> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(UsersActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                        }
                    });
                })
                .show();
    }

    @Override
    // Метод onEdit: обрабатывает соответствующее событие приложения.
    public void onEdit(User user) {
        showUserDialog(user);
    }

    @Override
    // Метод onDelete: обрабатывает соответствующее событие приложения.
    public void onDelete(User user) {
        if (user.getId().equals(authManager.getUser().getId())) {
            Toast.makeText(this, R.string.cannot_delete_self, Toast.LENGTH_SHORT).show();
            return;
        }
        deleteUser(user.getId());
    }
}
