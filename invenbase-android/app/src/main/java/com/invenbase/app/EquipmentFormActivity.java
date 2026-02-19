package com.invenbase.app;

import android.os.Bundle;
import android.text.TextUtils;
import android.view.View;
import android.widget.ArrayAdapter;
import android.widget.Button;
import android.widget.CheckBox;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.Spinner;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;

import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Category;
import com.invenbase.app.models.Equipment;
import com.invenbase.app.utils.AuthManager;

import android.widget.AdapterView;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class EquipmentFormActivity extends BaseActivity {

    public static final String EXTRA_EQUIPMENT_ID = "equipment_id";
    public static final String EXTRA_IS_EDIT = "is_edit";
    public static final String EXTRA_SQUAD_ID = "squad_id";

    private ApiService apiService;
    private AuthManager authManager;
    private EditText editName;
    private EditText editDescription;
    private Spinner spinnerSquad;
    private Spinner spinnerCategory;
    private EditText editQuantity;
    private EditText editAvailableQuantity;
    private CheckBox checkUnique;
    private View quantityRow;
    private EditText editLocation;
    private Spinner spinnerStatus;
    private Button buttonSave;
    private ProgressBar progressBar;
    private List<Category> categories = new ArrayList<>();
    private List<Map<String, Object>> squadsList = new ArrayList<>();
    private List<String> squadIds = new ArrayList<>();
    private String equipmentId;

    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_equipment_form);

        apiService = ApiClient.getInstance(this).getApiService();
        authManager = AuthManager.getInstance(this);

        if (!authManager.isAdmin() && !authManager.isResponsible()) {
            Toast.makeText(this, R.string.access_denied, Toast.LENGTH_LONG).show();
            finish();
            return;
        }

        equipmentId = getIntent().getStringExtra(EXTRA_EQUIPMENT_ID);
        boolean isEdit = getIntent().getBooleanExtra(EXTRA_IS_EDIT, false);

        editName = findViewById(R.id.edit_name);
        editDescription = findViewById(R.id.edit_description);
        spinnerSquad = findViewById(R.id.spinner_squad);
        spinnerCategory = findViewById(R.id.spinner_category);
        editQuantity = findViewById(R.id.edit_quantity);
        editAvailableQuantity = findViewById(R.id.edit_available_quantity);
        checkUnique = findViewById(R.id.check_unique);
        quantityRow = findViewById(R.id.quantity_row);
        editLocation = findViewById(R.id.edit_location);

        checkUnique.setOnCheckedChangeListener((buttonView, isChecked) -> {
            if (isChecked) {
                editQuantity.setText("1");
                editAvailableQuantity.setText("1");
                editQuantity.setEnabled(false);
                editAvailableQuantity.setEnabled(false);
                quantityRow.setVisibility(View.GONE);
            } else {
                editQuantity.setEnabled(true);
                editAvailableQuantity.setEnabled(true);
                quantityRow.setVisibility(View.VISIBLE);
            }
        });
        spinnerStatus = findViewById(R.id.spinner_status);
        buttonSave = findViewById(R.id.button_save);
        progressBar = findViewById(R.id.progress_bar);

        ArrayAdapter<CharSequence> statusAdapter = ArrayAdapter.createFromResource(
                this, R.array.equipment_statuses, android.R.layout.simple_spinner_item);
        statusAdapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
        spinnerStatus.setAdapter(statusAdapter);

        buttonSave.setOnClickListener(v -> save());

        spinnerSquad.setOnItemSelectedListener(new AdapterView.OnItemSelectedListener() {
            @Override
            public void onItemSelected(AdapterView<?> parent, View view, int position, long id) {
                String squadId = position > 0 && position <= squadIds.size() ? squadIds.get(position - 1) : null;
                loadCategories(squadId, null);
            }
            @Override
            public void onNothingSelected(AdapterView<?> parent) {}
        });

        setTitle(isEdit ? R.string.edit_equipment : R.string.add_equipment);
        loadSquads();
    }

    private static String str(Object o) {
        return o == null ? "" : String.valueOf(o);
    }

    private void loadSquads() {
        apiService.getSquads().enqueue(new Callback<List<Map<String, Object>>>() {
            @Override
            public void onResponse(Call<List<Map<String, Object>>> call, Response<List<Map<String, Object>>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    squadsList.clear();
                    squadsList.addAll(response.body());
                    squadIds.clear();
                    squadIds.add("");
                    List<String> squadNames = new ArrayList<>();
                    squadNames.add(getString(R.string.no_squad));
                    for (Map<String, Object> s : squadsList) {
                        squadNames.add(str(s.get("name")));
                        squadIds.add(str(s.get("id")));
                    }
                    ArrayAdapter<String> adapter = new ArrayAdapter<>(
                            EquipmentFormActivity.this,
                            android.R.layout.simple_spinner_item,
                            squadNames);
                    adapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
                    spinnerSquad.setAdapter(adapter);

                    if (equipmentId != null) {
                        loadEquipment();
                    } else {
                        loadCategories(null, null);
                        String presetSquadId = getIntent().getStringExtra(EXTRA_SQUAD_ID);
                        if (presetSquadId != null && !presetSquadId.isEmpty()) {
                            for (int i = 0; i < squadIds.size(); i++) {
                                if (presetSquadId.equals(squadIds.get(i))) {
                                    spinnerSquad.setSelection(i);
                                    loadCategories(presetSquadId, null);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
            @Override
            public void onFailure(Call<List<Map<String, Object>>> call, Throwable t) {
                squadIds.clear();
                squadIds.add("");
                List<String> fallback = new ArrayList<>();
                fallback.add(getString(R.string.no_squad));
                ArrayAdapter<String> adapter = new ArrayAdapter<>(
                        EquipmentFormActivity.this,
                        android.R.layout.simple_spinner_item,
                        fallback);
                adapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
                spinnerSquad.setAdapter(adapter);
                loadCategories(null, null);
                if (equipmentId != null) loadEquipment();
            }
        });
    }

    private void loadCategories(@Nullable String squadId, @Nullable String selectCategoryId) {
        Call<List<Category>> call = (squadId != null && !squadId.isEmpty())
                ? apiService.getCategoriesBySquad(squadId)
                : apiService.getCategories();
        call.enqueue(new Callback<List<Category>>() {
            @Override
            public void onResponse(Call<List<Category>> call, Response<List<Category>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    categories = response.body();
                    List<String> categoryNames = new ArrayList<>();
                    categoryNames.add(getString(R.string.no_category));
                    for (Category cat : categories) {
                        categoryNames.add(cat.getName());
                    }
                    ArrayAdapter<String> adapter = new ArrayAdapter<>(
                            EquipmentFormActivity.this,
                            android.R.layout.simple_spinner_item,
                            categoryNames);
                    adapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
                    spinnerCategory.setAdapter(adapter);
                    if (selectCategoryId != null && !selectCategoryId.isEmpty()) {
                        for (int i = 0; i < categories.size(); i++) {
                            if (selectCategoryId.equals(categories.get(i).getId())) {
                                spinnerCategory.setSelection(i + 1);
                                break;
                            }
                        }
                    }
                }
            }
            @Override
            public void onFailure(Call<List<Category>> call, Throwable t) {}
        });
    }

    private void loadEquipment() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getEquipmentById(equipmentId).enqueue(new Callback<Equipment>() {
            @Override
            public void onResponse(Call<Equipment> call, Response<Equipment> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful() && response.body() != null) {
                    Equipment eq = response.body();
                    editName.setText(eq.getName());
                    editDescription.setText(eq.getDescription());
                    editQuantity.setText(String.valueOf(eq.getQuantity()));
                    editAvailableQuantity.setText(String.valueOf(eq.getAvailableQuantity()));
                    editLocation.setText(eq.getLocation());
                    checkUnique.setChecked(eq.isUnique());
                    if (eq.isUnique()) {
                        editQuantity.setEnabled(false);
                        editAvailableQuantity.setEnabled(false);
                        quantityRow.setVisibility(View.GONE);
                    } else {
                        quantityRow.setVisibility(View.VISIBLE);
                    }
                    String squadId = eq.getSquadId();
                    if (squadId != null && !squadId.isEmpty()) {
                        for (int i = 0; i < squadIds.size(); i++) {
                            if (squadId.equals(squadIds.get(i))) {
                                spinnerSquad.setSelection(i);
                                break;
                            }
                        }
                    }
                    loadCategories(squadId, eq.getCategoryId());
                    String status = eq.getStatus();
                    if (status != null) {
                        int pos = status.equals("available") ? 0 : status.equals("maintenance") ? 1 : 2;
                        spinnerStatus.setSelection(pos);
                    }
                }
            }
            @Override
            public void onFailure(Call<Equipment> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(EquipmentFormActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    private void save() {
        String name = editName.getText().toString().trim();
        boolean isUnique = checkUnique.isChecked();
        String qtyStr = editQuantity.getText().toString().trim();
        String availStr = editAvailableQuantity.getText().toString().trim();

        if (TextUtils.isEmpty(name)) {
            Toast.makeText(this, R.string.fill_required_fields, Toast.LENGTH_SHORT).show();
            return;
        }
        if (!isUnique && TextUtils.isEmpty(qtyStr)) {
            Toast.makeText(this, R.string.fill_required_fields, Toast.LENGTH_SHORT).show();
            return;
        }

        Map<String, Object> data = new HashMap<>();
        data.put("name", name);
        data.put("description", editDescription.getText().toString().trim());
        int squadPos = spinnerSquad.getSelectedItemPosition();
        if (squadPos >= 0 && squadPos < squadIds.size()) {
            String sid = squadIds.get(squadPos);
            if (sid != null && !sid.isEmpty()) {
                data.put("squad_id", sid);
            }
        }
        int catPos = spinnerCategory.getSelectedItemPosition();
        if (catPos > 0 && catPos <= categories.size()) {
            data.put("category_id", categories.get(catPos - 1).getId());
        }
        data.put("is_unique", isUnique);
        if (isUnique) {
            data.put("quantity", 1);
            data.put("available_quantity", 1);
        } else {
            int qty = Integer.parseInt(qtyStr);
            data.put("quantity", qty);
            data.put("available_quantity", TextUtils.isEmpty(availStr) ? qty : Integer.parseInt(availStr));
        }
        data.put("location", editLocation.getText().toString().trim());
        String status = spinnerStatus.getSelectedItemPosition() == 0 ? "available" :
                spinnerStatus.getSelectedItemPosition() == 1 ? "maintenance" : "unavailable";
        data.put("status", status);

        progressBar.setVisibility(View.VISIBLE);
        buttonSave.setEnabled(false);

        Call<Equipment> call = equipmentId != null
                ? apiService.updateEquipment(equipmentId, data)
                : apiService.createEquipment(data);

        call.enqueue(new Callback<Equipment>() {
            @Override
            public void onResponse(Call<Equipment> call, Response<Equipment> response) {
                progressBar.setVisibility(View.GONE);
                buttonSave.setEnabled(true);
                if (response.isSuccessful()) {
                    Toast.makeText(EquipmentFormActivity.this, R.string.success, Toast.LENGTH_SHORT).show();
                    finish();
                } else {
                    Toast.makeText(EquipmentFormActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<Equipment> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                buttonSave.setEnabled(true);
                Toast.makeText(EquipmentFormActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }
}
