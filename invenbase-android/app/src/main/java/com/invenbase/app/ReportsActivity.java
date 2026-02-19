package com.invenbase.app;

import android.app.DatePickerDialog;
import android.os.Bundle;
import android.view.View;
import android.widget.LinearLayout;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.GridLayoutManager;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.google.android.material.button.MaterialButton;
import com.invenbase.app.adapters.EquipmentReportAdapter;
import com.invenbase.app.adapters.ReportCategoryAdapter;
import com.invenbase.app.adapters.ReportStatAdapter;
import com.invenbase.app.adapters.TopCategoryAdapter;
import com.invenbase.app.adapters.TopEquipmentAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Booking;
import com.invenbase.app.models.Equipment;
import com.invenbase.app.models.EquipmentReportRow;
import com.invenbase.app.models.ReportCategoryItem;
import com.invenbase.app.models.StatItem;
import com.invenbase.app.models.TopCategoryRow;
import com.invenbase.app.models.TopEquipmentRow;
import com.invenbase.app.utils.AuthManager;

import java.util.ArrayList;
import java.util.Calendar;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class ReportsActivity extends BaseActivity {

    private ApiService apiService;
    private AuthManager authManager;
    private ProgressBar progressBar;

    private RecyclerView recyclerEquipmentStats;
    private RecyclerView recyclerCategoryStats;
    private RecyclerView recyclerEquipmentTable;
    private RecyclerView recyclerBookingStats;
    private RecyclerView recyclerTopEquipment;
    private RecyclerView recyclerTopCategories;
    private LinearLayout periodChips;
    private TextView textDateFrom;
    private TextView textDateTo;

    private Map<String, Object> equipmentReport;
    private Map<String, Object> bookingReport;
    private List<Equipment> allEquipment = new ArrayList<>();
    private List<Booking> allBookings = new ArrayList<>();

    private String fromDate = "";
    private String toDate = "";
    private String periodPreset = "all";

    private static final String[] PRESET_KEYS = {"all", "today", "yesterday", "week", "month"};

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

        progressBar = findViewById(R.id.progress_bar);
        recyclerEquipmentStats = findViewById(R.id.recycler_equipment_stats);
        recyclerCategoryStats = findViewById(R.id.recycler_category_stats);
        recyclerEquipmentTable = findViewById(R.id.recycler_equipment_table);
        recyclerBookingStats = findViewById(R.id.recycler_booking_stats);
        recyclerTopEquipment = findViewById(R.id.recycler_top_equipment);
        recyclerTopCategories = findViewById(R.id.recycler_top_categories);
        periodChips = findViewById(R.id.period_chips);
        textDateFrom = findViewById(R.id.text_date_from);
        textDateTo = findViewById(R.id.text_date_to);

        recyclerEquipmentStats.setLayoutManager(new GridLayoutManager(this, 3));
        recyclerCategoryStats.setLayoutManager(new LinearLayoutManager(this));
        recyclerEquipmentTable.setLayoutManager(new LinearLayoutManager(this));
        recyclerBookingStats.setLayoutManager(new GridLayoutManager(this, 2));
        recyclerTopEquipment.setLayoutManager(new LinearLayoutManager(this));
        recyclerTopCategories.setLayoutManager(new LinearLayoutManager(this));

        setupPeriodChips();
        setupDatePickers();
        textDateFrom.setText("—");
        textDateTo.setText("—");

        setTitle(R.string.reports);
        loadReports();
    }

    private void setupPeriodChips() {
        int[] labels = {R.string.period_all, R.string.period_today, R.string.period_yesterday, R.string.period_week, R.string.period_month};
        for (int i = 0; i < PRESET_KEYS.length; i++) {
            final String key = PRESET_KEYS[i];
            MaterialButton btn = new MaterialButton(this);
            btn.setText(labels[i]);
            btn.setLayoutParams(new LinearLayout.LayoutParams(LinearLayout.LayoutParams.WRAP_CONTENT, LinearLayout.LayoutParams.WRAP_CONTENT));
            LinearLayout.LayoutParams lp = (LinearLayout.LayoutParams) btn.getLayoutParams();
            lp.setMarginEnd((int) (8 * getResources().getDisplayMetrics().density));
            btn.getLayoutParams();
            btn.setOnClickListener(v -> applyPeriodPreset(key));
            periodChips.addView(btn);
        }
    }

    private void setupDatePickers() {
        textDateFrom.setOnClickListener(v -> showDatePicker(true));
        textDateTo.setOnClickListener(v -> showDatePicker(false));
    }

    private void showDatePicker(boolean isFrom) {
        Calendar c = Calendar.getInstance();
        String cur = isFrom ? fromDate : toDate;
        if (cur != null && !cur.isEmpty() && cur.length() >= 10) {
            try {
                int y = Integer.parseInt(cur.substring(0, 4));
                int m = Integer.parseInt(cur.substring(5, 7)) - 1;
                int d = Integer.parseInt(cur.substring(8, 10));
                c.set(y, m, d);
            } catch (Exception ignored) {}
        }
        new DatePickerDialog(this, (view, year, month, dayOfMonth) -> {
            String date = String.format(Locale.US, "%04d-%02d-%02d", year, month + 1, dayOfMonth);
            if (isFrom) {
                fromDate = date;
                textDateFrom.setText(date);
            } else {
                toDate = date;
                textDateTo.setText(date);
            }
            periodPreset = "custom";
            refreshBookingSection();
        }, c.get(Calendar.YEAR), c.get(Calendar.MONTH), c.get(Calendar.DAY_OF_MONTH)).show();
    }

    private void applyPeriodPreset(String preset) {
        periodPreset = preset;
        Calendar today = Calendar.getInstance();

        if ("all".equals(preset)) {
            fromDate = "";
            toDate = "";
        } else if ("today".equals(preset)) {
            fromDate = formatDate(today.get(Calendar.YEAR), today.get(Calendar.MONTH), today.get(Calendar.DAY_OF_MONTH));
            toDate = fromDate;
        } else if ("yesterday".equals(preset)) {
            today.add(Calendar.DAY_OF_MONTH, -1);
            fromDate = formatDate(today.get(Calendar.YEAR), today.get(Calendar.MONTH), today.get(Calendar.DAY_OF_MONTH));
            toDate = fromDate;
        } else if ("week".equals(preset)) {
            today.add(Calendar.DAY_OF_MONTH, -6);
            fromDate = formatDate(today.get(Calendar.YEAR), today.get(Calendar.MONTH), today.get(Calendar.DAY_OF_MONTH));
            today.add(Calendar.DAY_OF_MONTH, 6);
            toDate = formatDate(today.get(Calendar.YEAR), today.get(Calendar.MONTH), today.get(Calendar.DAY_OF_MONTH));
        } else if ("month".equals(preset)) {
            fromDate = formatDate(today.get(Calendar.YEAR), today.get(Calendar.MONTH), 1);
            toDate = formatDate(today.get(Calendar.YEAR), today.get(Calendar.MONTH), today.get(Calendar.DAY_OF_MONTH));
        }

        textDateFrom.setText(fromDate.isEmpty() ? "—" : fromDate);
        textDateTo.setText(toDate.isEmpty() ? "—" : toDate);
        refreshBookingSection();
    }

    private String formatDate(int year, int month, int day) {
        return String.format(Locale.US, "%04d-%02d-%02d", year, month + 1, day);
    }

    private void loadReports() {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getEquipmentReport().enqueue(new Callback<Map<String, Object>>() {
            @Override
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    equipmentReport = response.body();
                }
                loadNext();
            }
            @Override
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(ReportsActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
        apiService.getBookingReport().enqueue(new Callback<Map<String, Object>>() {
            @Override
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    bookingReport = response.body();
                }
                loadNext();
            }
            @Override
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                loadNext();
            }
        });
        apiService.getEquipment().enqueue(new Callback<List<Equipment>>() {
            @Override
            public void onResponse(Call<List<Equipment>> call, Response<List<Equipment>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    allEquipment = response.body();
                }
                loadNext();
            }
            @Override
            public void onFailure(Call<List<Equipment>> call, Throwable t) {
                loadNext();
            }
        });
        apiService.getBookings().enqueue(new Callback<List<Booking>>() {
            @Override
            public void onResponse(Call<List<Booking>> call, Response<List<Booking>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    allBookings = response.body();
                }
                loadNext();
            }
            @Override
            public void onFailure(Call<List<Booking>> call, Throwable t) {
                loadNext();
            }
        });
    }

    private int loadCount = 0;
    private static final int LOAD_TOTAL = 4;

    private void loadNext() {
        loadCount++;
        if (loadCount >= LOAD_TOTAL) {
            loadCount = 0;
            progressBar.setVisibility(View.GONE);
            showAllReports();
        }
    }

    private void showAllReports() {
        showEquipmentSection();
        showBookingSection();
    }

    private void showEquipmentSection() {
        if (equipmentReport == null) return;

        List<StatItem> equipmentStats = new ArrayList<>();
        equipmentStats.add(new StatItem(getString(R.string.report_total_positions), String.valueOf(getInt(equipmentReport, "total_equipment"))));
        equipmentStats.add(new StatItem(getString(R.string.report_available_now), String.valueOf(getInt(equipmentReport, "available_equipment"))));
        equipmentStats.add(new StatItem(getString(R.string.report_in_booking), String.valueOf(getInt(equipmentReport, "booked_equipment"))));

        ReportStatAdapter equipmentAdapter = new ReportStatAdapter();
        equipmentAdapter.setItems(equipmentStats);
        recyclerEquipmentStats.setAdapter(equipmentAdapter);

        List<ReportCategoryItem> categoryItems = new ArrayList<>();
        Object byCategoryObj = equipmentReport.get("by_category");
        if (byCategoryObj instanceof List) {
            for (Object o : (List<?>) byCategoryObj) {
                if (o instanceof Map) {
                    Map<String, Object> cat = (Map<String, Object>) o;
                    String name = str(cat.get("category_name"));
                    if (name.isEmpty()) name = getString(R.string.no_category);
                    categoryItems.add(new ReportCategoryItem(name, getLong(cat, "total"), getLong(cat, "available"), getLong(cat, "booked")));
                }
            }
        }
        ReportCategoryAdapter categoryAdapter = new ReportCategoryAdapter();
        categoryAdapter.setItems(categoryItems);
        recyclerCategoryStats.setAdapter(categoryAdapter);

        Map<String, String> catNameById = new HashMap<>();
        if (byCategoryObj instanceof List) {
            for (Object o : (List<?>) byCategoryObj) {
                if (o instanceof Map) {
                    Map<String, Object> c = (Map<String, Object>) o;
                    Object id = c.get("category_id");
                    String nm = str(c.get("category_name"));
                    if (id != null) catNameById.put(str(id), nm.isEmpty() ? getString(R.string.no_category) : nm);
                }
            }
        }

        List<EquipmentReportRow> equipmentRows = new ArrayList<>();
        for (int i = 0; i < allEquipment.size(); i++) {
            Equipment eq = allEquipment.get(i);
            int total = eq.getQuantity();
            int available = eq.getAvailableQuantity();
            int booked = Math.max(0, total - available);
            String catName = eq.getCategoryId() != null ? catNameById.getOrDefault(eq.getCategoryId(), getString(R.string.no_category)) : getString(R.string.no_category);
            String status = "available".equals(eq.getStatus()) ? getString(R.string.equipment_status_available) : "maintenance".equals(eq.getStatus()) ? getString(R.string.equipment_status_maintenance) : str(eq.getStatus());
            equipmentRows.add(new EquipmentReportRow(i, eq.getName(), catName, total, available, booked, status));
        }
        EquipmentReportAdapter equipmentTableAdapter = new EquipmentReportAdapter(this);
        equipmentTableAdapter.setItems(equipmentRows);
        recyclerEquipmentTable.setAdapter(equipmentTableAdapter);
    }

    private List<Booking> getFilteredBookings() {
        List<Booking> result = new ArrayList<>();
        for (Booking b : allBookings) {
            String start = b.getStartDate();
            if (start == null || start.length() < 10) continue;
            String iso = start.substring(0, 10);
            if (fromDate != null && !fromDate.isEmpty() && iso.compareTo(fromDate) < 0) continue;
            if (toDate != null && !toDate.isEmpty() && iso.compareTo(toDate) > 0) continue;
            result.add(b);
        }
        return result;
    }

    private static class EquipUsage {
        String eqName;
        String catName;
        int bookingsCount;
        int totalQuantity;
    }

    private void computeUsageStats() {
        List<Booking> filtered = getFilteredBookings();

        Map<String, String> catNameById = new HashMap<>();
        Object byCat = equipmentReport != null ? equipmentReport.get("by_category") : null;
        if (byCat instanceof List) {
            for (Object o : (List<?>) byCat) {
                if (o instanceof Map) {
                    Map<String, Object> c = (Map<String, Object>) o;
                    Object id = c.get("category_id");
                    String nm = str(c.get("category_name"));
                    if (id != null) catNameById.put(str(id), nm.isEmpty() ? getString(R.string.no_category) : nm);
                }
            }
        }

        Map<String, Equipment> equipmentById = new HashMap<>();
        for (Equipment e : allEquipment) equipmentById.put(e.getId(), e);

        Map<String, EquipUsage> equipCount = new HashMap<>();
        Map<String, Integer> catCount = new HashMap<>();
        for (Booking b : filtered) {
            String eqId = b.getEquipmentId();
            if (eqId == null || eqId.isEmpty()) continue;
            Equipment eq = equipmentById.get(eqId);
            String eqName = b.getEquipmentName() != null ? b.getEquipmentName() : (eq != null ? eq.getName() : "—");
            String catName = eq != null && eq.getCategoryId() != null ? catNameById.getOrDefault(eq.getCategoryId(), getString(R.string.no_category)) : getString(R.string.no_category);

            EquipUsage u = equipCount.get(eqId);
            if (u == null) {
                u = new EquipUsage();
                u.eqName = eqName;
                u.catName = catName;
                u.bookingsCount = 0;
                u.totalQuantity = 0;
                equipCount.put(eqId, u);
            }
            u.bookingsCount++;
            u.totalQuantity += b.getQuantity() > 0 ? b.getQuantity() : 0;

            catCount.put(catName, catCount.getOrDefault(catName, 0) + 1);
        }

        List<EquipUsage> equipmentUsage = new ArrayList<>(equipCount.values());
        equipmentUsage.sort((a, b) -> Integer.compare(b.bookingsCount, a.bookingsCount));
        int totalEquip = 0;
        for (EquipUsage r : equipmentUsage) totalEquip += r.bookingsCount;
        List<TopEquipmentRow> topEquipment = new ArrayList<>();
        for (int i = 0; i < Math.min(20, equipmentUsage.size()); i++) {
            EquipUsage r = equipmentUsage.get(i);
            String pct = totalEquip > 0 ? String.format(Locale.US, "%.1f", 100.0 * r.bookingsCount / totalEquip) : "0.0";
            topEquipment.add(new TopEquipmentRow(i, r.eqName, r.catName, r.bookingsCount, pct, r.totalQuantity));
        }

        List<TopCategoryRow> categoryUsage = new ArrayList<>();
        int totalCat = 0;
        for (Integer v : catCount.values()) totalCat += v;
        List<Map.Entry<String, Integer>> catEntries = new ArrayList<>(catCount.entrySet());
        catEntries.sort((a, b) -> Integer.compare(b.getValue(), a.getValue()));
        for (int i = 0; i < Math.min(20, catEntries.size()); i++) {
            Map.Entry<String, Integer> e = catEntries.get(i);
            String pct = totalCat > 0 ? String.format(Locale.US, "%.1f", 100.0 * e.getValue() / totalCat) : "0.0";
            categoryUsage.add(new TopCategoryRow(i, e.getKey(), e.getValue(), pct));
        }

        TopEquipmentAdapter topEquipAdapter = new TopEquipmentAdapter(this);
        topEquipAdapter.setItems(topEquipment);
        recyclerTopEquipment.setAdapter(topEquipAdapter);

        TopCategoryAdapter topCatAdapter = new TopCategoryAdapter(this);
        topCatAdapter.setItems(categoryUsage);
        recyclerTopCategories.setAdapter(topCatAdapter);
    }

    private void showBookingSection() {
        if (bookingReport == null) return;

        List<StatItem> bookingStats = new ArrayList<>();
        bookingStats.add(new StatItem(getString(R.string.report_booking_total), String.valueOf(getInt(bookingReport, "total"))));
        bookingStats.add(new StatItem(getString(R.string.report_pending_approval), String.valueOf(getInt(bookingReport, "pending"))));
        bookingStats.add(new StatItem(getString(R.string.report_approved), String.valueOf(getInt(bookingReport, "approved"))));
        bookingStats.add(new StatItem(getString(R.string.report_rejected), String.valueOf(getInt(bookingReport, "rejected"))));
        bookingStats.add(new StatItem(getString(R.string.expired), String.valueOf(getInt(bookingReport, "expired"))));
        bookingStats.add(new StatItem(getString(R.string.report_cancelled), String.valueOf(getInt(bookingReport, "cancelled"))));
        bookingStats.add(new StatItem(getString(R.string.report_completed), String.valueOf(getInt(bookingReport, "completed"))));

        ReportStatAdapter bookingAdapter = new ReportStatAdapter();
        bookingAdapter.setItems(bookingStats);
        recyclerBookingStats.setAdapter(bookingAdapter);

        computeUsageStats();
    }

    private void refreshBookingSection() {
        if (bookingReport != null) showBookingSection();
    }

    private int getInt(Map<String, Object> map, String key) {
        if (map == null || map.get(key) == null) return 0;
        Object v = map.get(key);
        return v instanceof Number ? ((Number) v).intValue() : 0;
    }

    private long getLong(Map<String, Object> map, String key) {
        if (map == null || map.get(key) == null) return 0;
        Object v = map.get(key);
        return v instanceof Number ? ((Number) v).longValue() : 0;
    }

    private static String str(Object o) {
        return o != null ? String.valueOf(o) : "";
    }
}
