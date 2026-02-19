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

import com.invenbase.app.adapters.CategoriesAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Category;
import com.invenbase.app.utils.AuthManager;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class CategoriesActivity extends BaseActivity implements CategoriesAdapter.OnCategoryActionListener {

    private ApiService apiService;
    private AuthManager authManager;
    private RecyclerView recyclerView;
    private ProgressBar progressBar;
    private TextView emptyView;
    private CategoriesAdapter adapter;
    private List<Map<String, Object>> squadsList = new ArrayList<>();
    private List<String> squadIds = new ArrayList<>();

    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_categories);

        apiService = ApiClient.getInstance(this).getApiService();
        authManager = AuthManager.getInstance(this);

        if (!authManager.isAdmin() && !authManager.isResponsible()) {
            Toast.makeText(this, R.string.access_denied, Toast.LENGTH_LONG).show();
            finish();
            return;
        }

        recyclerView = findViewById(R.id.recycler_categories);
        progressBar = findViewById(R.id.progress_bar);
        emptyView = findViewById(R.id.text_empty);
        Button buttonAdd = findViewById(R.id.button_add_category);

        recyclerView.setLayoutManager(new LinearLayoutManager(this));
        adapter = new CategoriesAdapter(this, authManager.isAdmin() || authManager.isResponsible());
        recyclerView.setAdapter(adapter);

        buttonAdd.setOnClickListener(v -> showCategoryDialog(null));

        setTitle(R.string.categories);
        loadSquads();
        loadCategories();
    }

    private void loadSquads() {
        apiService.getSquads().enqueue(new Callback<List<Map<String, Object>>>() {
            @Override
            public void onResponse(Call<List<Map<String, Object>>> call, Response<List<Map<String, Object>>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    squadsList.clear();
                    squadsList.addAll(response.body());
                }
            }
            @Override
            public void onFailure(Call<List<Map<String, Object>>> call, Throwable t) {}
        });
    }

    private static String str(Object o) {
        return o != null ? String.valueOf(o) : "";
    }

    private void loadCategories() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getCategories().enqueue(new Callback<List<Category>>() {
            @Override
            public void onResponse(Call<List<Category>> call, Response<List<Category>> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful() && response.body() != null) {
                    List<Category> list = response.body();
                    adapter.setItems(list);
                    emptyView.setVisibility(list.isEmpty() ? View.VISIBLE : View.GONE);
                } else {
                    Toast.makeText(CategoriesActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<List<Category>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(CategoriesActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    private void showCategoryDialog(@Nullable Category category) {
        View view = LayoutInflater.from(this).inflate(R.layout.dialog_category, null, false);
        EditText editName = view.findViewById(R.id.edit_name);
        EditText editDescription = view.findViewById(R.id.edit_description);
        Spinner spinnerSquad = view.findViewById(R.id.spinner_squad);

        // Spinner: "Без сквада" + список сквадов
        List<String> squadNames = new ArrayList<>();
        squadNames.add(getString(R.string.no_squad));
        squadIds.clear();
        squadIds.add("");
        for (Map<String, Object> s : squadsList) {
            squadNames.add(str(s.get("name")));
            squadIds.add(str(s.get("id")));
        }
        ArrayAdapter<String> squadAdapter = new ArrayAdapter<>(this, android.R.layout.simple_spinner_item, squadNames);
        squadAdapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
        spinnerSquad.setAdapter(squadAdapter);

        if (category != null) {
            editName.setText(category.getName());
            editDescription.setText(category.getDescription());
            String catSquadId = category.getSquadId();
            if (catSquadId != null && !catSquadId.isEmpty()) {
                for (int i = 0; i < squadIds.size(); i++) {
                    if (catSquadId.equals(squadIds.get(i))) {
                        spinnerSquad.setSelection(i);
                        break;
                    }
                }
            }
        }

        new AlertDialog.Builder(this)
                .setTitle(category == null ? R.string.add_category : R.string.edit_category)
                .setView(view)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.save, (dialog, which) -> {
                    String name = editName.getText().toString().trim();
                    String desc = editDescription.getText().toString().trim();
                    if (TextUtils.isEmpty(name)) {
                        Toast.makeText(this, R.string.fill_required_fields, Toast.LENGTH_SHORT).show();
                        return;
                    }
                    Map<String, Object> data = new HashMap<>();
                    data.put("name", name);
                    data.put("description", desc.isEmpty() ? null : desc);
                    int squadPos = spinnerSquad.getSelectedItemPosition();
                    if (squadPos >= 0 && squadPos < squadIds.size()) {
                        String sid = squadIds.get(squadPos);
                        if (sid != null && !sid.isEmpty()) {
                            data.put("squad_id", sid);
                        }
                    }
                    if (category == null) {
                        createCategory(data);
                    } else {
                        updateCategory(category.getId(), data);
                    }
                })
                .show();
    }

    private void createCategory(Map<String, Object> data) {
        progressBar.setVisibility(View.VISIBLE);
        apiService.createCategory(data).enqueue(new Callback<Category>() {
            @Override
            public void onResponse(Call<Category> call, Response<Category> response) {
                progressBar.setVisibility(View.GONE);
                if (!response.isSuccessful()) {
                    Toast.makeText(CategoriesActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
                loadCategories();
            }

            @Override
            public void onFailure(Call<Category> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(CategoriesActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    private void updateCategory(String id, Map<String, Object> data) {
        progressBar.setVisibility(View.VISIBLE);
        apiService.updateCategory(id, data).enqueue(new Callback<Category>() {
            @Override
            public void onResponse(Call<Category> call, Response<Category> response) {
                progressBar.setVisibility(View.GONE);
                if (!response.isSuccessful()) {
                    Toast.makeText(CategoriesActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
                loadCategories();
            }

            @Override
            public void onFailure(Call<Category> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(CategoriesActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    private void deleteCategory(Category target) {
        final EditText input = new EditText(this);
        input.setHint(target.getName());
        input.setMinEms(12);

        android.widget.LinearLayout wrap = new android.widget.LinearLayout(this);
        wrap.setOrientation(android.widget.LinearLayout.VERTICAL);
        int pad = (int) (40 * getResources().getDisplayMetrics().density);
        wrap.setPadding(pad, pad, pad, 0);
        wrap.addView(input);

        new AlertDialog.Builder(this)
                .setTitle(R.string.confirm_delete_category)
                .setMessage("Для подтверждения введите точное название категории.")
                .setView(wrap)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.delete, (dialog, which) -> {
                    String typed = input.getText().toString().trim();
                    if (!target.getName().equals(typed)) {
                        Toast.makeText(this, "Название категории не совпадает. Удаление отменено.", Toast.LENGTH_SHORT).show();
                        return;
                    }

                    progressBar.setVisibility(View.VISIBLE);
                    apiService.deleteCategory(target.getId()).enqueue(new Callback<Void>() {
                        @Override
                        public void onResponse(Call<Void> call, Response<Void> response) {
                            progressBar.setVisibility(View.GONE);
                            if (!response.isSuccessful()) {
                                Toast.makeText(CategoriesActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                            }
                            loadCategories();
                        }

                        @Override
                        public void onFailure(Call<Void> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(CategoriesActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                        }
                    });
                })
                .show();
    }

    @Override
    public void onEdit(Category category) {
        showCategoryDialog(category);
    }

    @Override
    public void onDelete(Category category) {
        if (!authManager.isAdmin() && !authManager.isResponsible()) {
            Toast.makeText(this, R.string.access_denied, Toast.LENGTH_SHORT).show();
            return;
        }
        deleteCategory(category);
    }
}

