package com.invenbase.app;

import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ProgressBar;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Category;
import com.invenbase.app.models.Equipment;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class SquadDetailActivity extends BaseActivity {

    public static final String EXTRA_SQUAD_ID = "squad_id";

    private ApiService apiService;
    private String squadId;
    private ProgressBar progressBar;
    private TextView textName;
    private TextView textLocation;
    private TextView textResponsible;
    private TextView textDescription;
    private TextView textNoEquipment;
    private RecyclerView recyclerEquipment;
    private TextView textNoCategories;
    private RecyclerView recyclerCategories;
    private EquipmentListAdapter equipmentAdapter;
    private CategoryListAdapter categoryAdapter;

    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_squad_detail);

        apiService = ApiClient.getInstance(this).getApiService();
        squadId = getIntent().getStringExtra(EXTRA_SQUAD_ID);
        if (squadId == null || squadId.isEmpty()) {
            finish();
            return;
        }

        progressBar = findViewById(R.id.progress_bar);
        textName = findViewById(R.id.text_squad_name);
        textLocation = findViewById(R.id.text_squad_location);
        textResponsible = findViewById(R.id.text_squad_responsible);
        textDescription = findViewById(R.id.text_squad_description);
        textNoEquipment = findViewById(R.id.text_squad_no_equipment);
        recyclerEquipment = findViewById(R.id.recycler_squad_equipment);
        textNoCategories = findViewById(R.id.text_squad_no_categories);
        recyclerCategories = findViewById(R.id.recycler_squad_categories);

        recyclerEquipment.setLayoutManager(new LinearLayoutManager(this));
        equipmentAdapter = new EquipmentListAdapter(equipment -> EquipmentDetailActivity.open(this, equipment.getId()));
        recyclerEquipment.setAdapter(equipmentAdapter);

        recyclerCategories.setLayoutManager(new LinearLayoutManager(this));
        categoryAdapter = new CategoryListAdapter();
        recyclerCategories.setAdapter(categoryAdapter);

        setTitle(R.string.squad_detail);
        if (getSupportActionBar() != null) {
            getSupportActionBar().setDisplayHomeAsUpEnabled(true);
        }

        loadSquad();
        loadEquipment();
        loadCategories();
    }

    @Override
    // Метод onSupportNavigateUp: обрабатывает соответствующее событие приложения.
    public boolean onSupportNavigateUp() {
        finish();
        return true;
    }

    // Метод loadSquad: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadSquad() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getSquad(squadId).enqueue(new Callback<Map<String, Object>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    Map<String, Object> squad = response.body();
                    textName.setText(str(squad.get("name")));
                    String loc = str(squad.get("location"));
                    if (!loc.isEmpty()) {
                        textLocation.setVisibility(View.VISIBLE);
                        textLocation.setText(loc);
                    } else {
                        textLocation.setVisibility(View.GONE);
                    }
                    String resp = str(squad.get("responsible_name"));
                    if (!resp.isEmpty()) {
                        textResponsible.setVisibility(View.VISIBLE);
                        textResponsible.setText(getString(R.string.squad_responsible) + ": " + resp);
                    } else {
                        textResponsible.setVisibility(View.GONE);
                    }
                    String desc = str(squad.get("description"));
                    if (!desc.isEmpty()) {
                        textDescription.setVisibility(View.VISIBLE);
                        textDescription.setText(desc);
                    } else {
                        textDescription.setVisibility(View.GONE);
                    }
                }
                progressBar.setVisibility(View.GONE);
            }
            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
            }
        });
    }

    // Метод loadEquipment: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadEquipment() {
        apiService.getSquadEquipment(squadId).enqueue(new Callback<List<Equipment>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<List<Equipment>> call, Response<List<Equipment>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    List<Equipment> list = response.body();
                    equipmentAdapter.setItems(list);
                    textNoEquipment.setVisibility(list.isEmpty() ? View.VISIBLE : View.GONE);
                }
            }
            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<List<Equipment>> call, Throwable t) {}
        });
    }

    // Метод loadCategories: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadCategories() {
        apiService.getCategoriesBySquad(squadId).enqueue(new Callback<List<Category>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<List<Category>> call, Response<List<Category>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    List<Category> list = response.body();
                    List<Category> forSquad = new ArrayList<>();
                    for (Category c : list) {
                        if (squadId.equals(c.getSquadId())) {
                            forSquad.add(c);
                        }
                    }
                    categoryAdapter.setItems(forSquad);
                    textNoCategories.setVisibility(forSquad.isEmpty() ? View.VISIBLE : View.GONE);
                }
            }
            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<List<Category>> call, Throwable t) {}
        });
    }

    // Метод str: выполняет основную бизнес- или UI-логику данного участка кода.
    private static String str(Object o) {
        return o == null ? "" : String.valueOf(o);
    }

    private static class EquipmentListAdapter extends RecyclerView.Adapter<EquipmentListAdapter.VH> {
        private final List<Equipment> items = new ArrayList<>();
        private final OnEquipmentClickListener listener;

        interface OnEquipmentClickListener {
            // Метод onEquipmentClick: обрабатывает соответствующее событие приложения.
            void onEquipmentClick(Equipment equipment);
        }

        EquipmentListAdapter(OnEquipmentClickListener listener) {
            this.listener = listener;
        }

        // Метод setItems: устанавливает или обновляет значение данных.
        void setItems(List<Equipment> list) {
            items.clear();
            if (list != null) items.addAll(list);
            notifyDataSetChanged();
        }

        @NonNull
        @Override
        // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
        public VH onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
            View v = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_squad_equipment, parent, false);
            return new VH(v);
        }

        @Override
        // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
        public void onBindViewHolder(@NonNull VH holder, int position) {
            Equipment eq = items.get(position);
            holder.name.setText(eq.getName());
            holder.available.setText(eq.getQuantity() > 0 ? (eq.getAvailableQuantity() + "/" + eq.getQuantity()) : "");
            holder.itemView.setOnClickListener(v -> listener.onEquipmentClick(eq));
        }

        @Override
        // Метод getItemCount: возвращает нужное значение для текущего контекста.
        public int getItemCount() {
            return items.size();
        }

        static class VH extends RecyclerView.ViewHolder {
            final TextView name;
            final TextView available;

            VH(@NonNull View itemView) {
                super(itemView);
                name = itemView.findViewById(R.id.text_equipment_name);
                available = itemView.findViewById(R.id.text_equipment_available);
            }
        }
    }

    private static class CategoryListAdapter extends RecyclerView.Adapter<CategoryListAdapter.VH> {
        private final List<Category> items = new ArrayList<>();

        // Метод setItems: устанавливает или обновляет значение данных.
        void setItems(List<Category> list) {
            items.clear();
            if (list != null) items.addAll(list);
            notifyDataSetChanged();
        }

        @NonNull
        @Override
        // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
        public VH onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
            View v = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_squad_category, parent, false);
            return new VH(v);
        }

        @Override
        // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
        public void onBindViewHolder(@NonNull VH holder, int position) {
            Category c = items.get(position);
            holder.name.setText(c.getName());
            if (c.getDescription() != null && !c.getDescription().isEmpty()) {
                holder.desc.setVisibility(View.VISIBLE);
                holder.desc.setText(c.getDescription());
            } else {
                holder.desc.setVisibility(View.GONE);
            }
        }

        @Override
        // Метод getItemCount: возвращает нужное значение для текущего контекста.
        public int getItemCount() {
            return items.size();
        }

        static class VH extends RecyclerView.ViewHolder {
            final TextView name;
            final TextView desc;

            VH(@NonNull View itemView) {
                super(itemView);
                name = itemView.findViewById(R.id.text_category_name);
                desc = itemView.findViewById(R.id.text_category_description);
            }
        }
    }
}
