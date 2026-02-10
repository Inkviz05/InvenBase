package com.invenbase.app;

import android.os.Bundle;
import android.view.View;
import android.widget.ProgressBar;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.GridLayoutManager;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.adapters.ReportCategoryAdapter;
import com.invenbase.app.adapters.StatsAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.ReportCategoryItem;
import com.invenbase.app.models.StatItem;
import com.invenbase.app.utils.AuthManager;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class ReportsActivity extends BaseActivity {

    private ApiService apiService;
    private AuthManager authManager;
    private RecyclerView recyclerEquipment;
    private RecyclerView recyclerCategoryStats;
    private RecyclerView recyclerBookings;
    private ProgressBar progressBar;

    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_reports);

        apiService = ApiClient.getInstance(this).getApiService();
        authManager = AuthManager.getInstance(this);

        if (!authManager.isAdmin() && !authManager.isResponsible()) {
            Toast.makeText(this, R.string.access_denied, Toast.LENGTH_LONG).show();
            finish();
            return;
        }

        recyclerEquipment = findViewById(R.id.recycler_equipment_stats);
        recyclerCategoryStats = findViewById(R.id.recycler_category_stats);
        recyclerBookings = findViewById(R.id.recycler_booking_stats);
        progressBar = findViewById(R.id.progress_bar);

        recyclerEquipment.setLayoutManager(new GridLayoutManager(this, 3));
        recyclerCategoryStats.setLayoutManager(new LinearLayoutManager(this));
        recyclerBookings.setLayoutManager(new GridLayoutManager(this, 2));

        setTitle(R.string.reports);
        loadReports();
    }

    private void loadReports() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getEquipmentReport().enqueue(new Callback<Map<String, Object>>() {
            @Override
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    Map<String, Object> equipmentReport = response.body();
                    apiService.getBookingReport().enqueue(new Callback<Map<String, Object>>() {
                        @Override
                        public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> bookingResponse) {
                            progressBar.setVisibility(View.GONE);
                            if (bookingResponse.isSuccessful() && bookingResponse.body() != null) {
                                Map<String, Object> bookingReport = bookingResponse.body();
                                showReports(equipmentReport, bookingReport);
                            } else {
                                Toast.makeText(ReportsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                            }
                        }

                        @Override
                        public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(ReportsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                        }
                    });
                } else {
                    progressBar.setVisibility(View.GONE);
                    Toast.makeText(ReportsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(ReportsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    @SuppressWarnings("unchecked")
    private void showReports(Map<String, Object> equipmentReport, Map<String, Object> bookingReport) {
        // Блок «Отчёт по оборудованию» — три карточки как в веб-версии
        List<StatItem> equipmentStats = new ArrayList<>();
        equipmentStats.add(new StatItem(getString(R.string.report_total_positions), String.valueOf(getInt(equipmentReport, "total_equipment"))));
        equipmentStats.add(new StatItem(getString(R.string.report_available_now), String.valueOf(getInt(equipmentReport, "available_equipment"))));
        equipmentStats.add(new StatItem(getString(R.string.report_in_booking), String.valueOf(getInt(equipmentReport, "booked_equipment"))));

        StatsAdapter equipmentAdapter = new StatsAdapter();
        equipmentAdapter.setItems(equipmentStats);
        recyclerEquipment.setAdapter(equipmentAdapter);

        // Статистика по категориям (как в веб-версии)
        List<ReportCategoryItem> categoryItems = new ArrayList<>();
        Object byCategoryObj = equipmentReport != null ? equipmentReport.get("by_category") : null;
        if (byCategoryObj instanceof List) {
            for (Object o : (List<?>) byCategoryObj) {
                if (o instanceof Map) {
                    Map<String, Object> cat = (Map<String, Object>) o;
                    String name = cat.get("category_name") != null ? String.valueOf(cat.get("category_name")) : "";
                    long total = getLong(cat, "total");
                    long available = getLong(cat, "available");
                    long booked = getLong(cat, "booked");
                    categoryItems.add(new ReportCategoryItem(name, total, available, booked));
                }
            }
        }
        ReportCategoryAdapter categoryAdapter = new ReportCategoryAdapter();
        categoryAdapter.setItems(categoryItems);
        recyclerCategoryStats.setAdapter(categoryAdapter);

        // Блок «Отчёт по бронированиям» — все карточки как в веб-версии
        List<StatItem> bookingStats = new ArrayList<>();
        int totalBookings = getInt(bookingReport, "total");
        bookingStats.add(new StatItem(getString(R.string.report_booking_total), String.valueOf(totalBookings)));
        bookingStats.add(new StatItem(getString(R.string.report_pending_approval), String.valueOf(getInt(bookingReport, "pending"))));
        bookingStats.add(new StatItem(getString(R.string.report_approved), String.valueOf(getInt(bookingReport, "approved"))));
        bookingStats.add(new StatItem(getString(R.string.report_rejected), String.valueOf(getInt(bookingReport, "rejected"))));
        bookingStats.add(new StatItem(getString(R.string.expired), String.valueOf(getInt(bookingReport, "expired"))));
        bookingStats.add(new StatItem(getString(R.string.report_cancelled), String.valueOf(getInt(bookingReport, "cancelled"))));
        bookingStats.add(new StatItem(getString(R.string.report_completed), String.valueOf(getInt(bookingReport, "completed"))));

        StatsAdapter bookingAdapter = new StatsAdapter();
        bookingAdapter.setItems(bookingStats);
        recyclerBookings.setAdapter(bookingAdapter);
    }

    private int getInt(Map<String, Object> map, String key) {
        if (map == null || map.get(key) == null) return 0;
        Object value = map.get(key);
        if (value instanceof Number) return ((Number) value).intValue();
        return 0;
    }

    private long getLong(Map<String, Object> map, String key) {
        if (map == null || map.get(key) == null) return 0L;
        Object value = map.get(key);
        if (value instanceof Number) return ((Number) value).longValue();
        return 0L;
    }
}
