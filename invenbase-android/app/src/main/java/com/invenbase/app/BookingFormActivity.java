package com.invenbase.app;

import android.app.DatePickerDialog;
import android.app.TimePickerDialog;
import android.os.Bundle;
import android.text.TextUtils;
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

import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Booking;
import com.invenbase.app.models.Equipment;

import java.text.SimpleDateFormat;
import java.util.Calendar;
import java.util.HashMap;
import java.util.Locale;
import java.util.Map;
import java.util.TimeZone;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class BookingFormActivity extends BaseActivity {

    public static final String EXTRA_EQUIPMENT_ID = "equipment_id";

    private ApiService apiService;
    private TextView textEquipmentInfo;
    private EditText editQuantity;
    private EditText editStartDate;
    private EditText editEndDate;
    private EditText editPurpose;
    private Spinner spinnerPermissionType;
    private Button buttonCreate;
    private Button buttonCancel;
    private ProgressBar progressBar;

    private Equipment equipment;
    private final Calendar startCalendar = Calendar.getInstance();
    private final Calendar endCalendar = Calendar.getInstance();

    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_booking_form);

        apiService = ApiClient.getInstance(this).getApiService();

        textEquipmentInfo = findViewById(R.id.text_equipment_info);
        editQuantity = findViewById(R.id.edit_quantity);
        editStartDate = findViewById(R.id.edit_start_date);
        editEndDate = findViewById(R.id.edit_end_date);
        editPurpose = findViewById(R.id.edit_purpose);
        spinnerPermissionType = findViewById(R.id.spinner_permission_type);
        buttonCreate = findViewById(R.id.button_create_booking);
        buttonCancel = findViewById(R.id.button_cancel);
        progressBar = findViewById(R.id.progress_bar);

        ArrayAdapter<CharSequence> adapter = ArrayAdapter.createFromResource(
                this,
                R.array.permission_types,
                android.R.layout.simple_spinner_item
        );
        adapter.setDropDownViewResource(android.R.layout.simple_spinner_dropdown_item);
        spinnerPermissionType.setAdapter(adapter);

        editStartDate.setOnClickListener(v -> pickDateTime(startCalendar, editStartDate));
        editEndDate.setOnClickListener(v -> pickDateTime(endCalendar, editEndDate));

        buttonCancel.setOnClickListener(v -> finish());
        buttonCreate.setOnClickListener(v -> submit());

        String equipmentId = getIntent().getStringExtra(EXTRA_EQUIPMENT_ID);
        if (equipmentId != null && !equipmentId.trim().isEmpty()) {
            loadEquipment(equipmentId);
        } else {
            textEquipmentInfo.setText(R.string.no_data);
        }

        setTitle(R.string.create_booking);
    }

    // Метод pickDateTime: выполняет основную бизнес- или UI-логику данного участка кода.
    private void pickDateTime(Calendar calendar, EditText target) {
        Calendar now = Calendar.getInstance();
        DatePickerDialog datePicker = new DatePickerDialog(
                this,
                (view, year, month, dayOfMonth) -> {
                    calendar.set(Calendar.YEAR, year);
                    calendar.set(Calendar.MONTH, month);
                    calendar.set(Calendar.DAY_OF_MONTH, dayOfMonth);
                    TimePickerDialog timePicker = new TimePickerDialog(
                            this,
                            (timeView, hourOfDay, minute) -> {
                                calendar.set(Calendar.HOUR_OF_DAY, hourOfDay);
                                calendar.set(Calendar.MINUTE, minute);
                                target.setText(formatLocal(calendar));
                            },
                            now.get(Calendar.HOUR_OF_DAY),
                            now.get(Calendar.MINUTE),
                            true
                    );
                    timePicker.show();
                },
                now.get(Calendar.YEAR),
                now.get(Calendar.MONTH),
                now.get(Calendar.DAY_OF_MONTH)
        );
        datePicker.show();
    }

    // Метод loadEquipment: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadEquipment(String equipmentId) {
        progressBar.setVisibility(View.VISIBLE);
        apiService.getEquipmentById(equipmentId).enqueue(new Callback<Equipment>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Equipment> call, Response<Equipment> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful() && response.body() != null) {
                    equipment = response.body();
                    String info = equipment.getName() +
                            " (доступно: " + equipment.getAvailableQuantity() +
                            " из " + equipment.getQuantity() + ")";
                    textEquipmentInfo.setText(info);
                } else {
                    textEquipmentInfo.setText(R.string.equipment_not_found);
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Equipment> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                textEquipmentInfo.setText(R.string.error);
            }
        });
    }

    // Метод submit: выполняет основную бизнес- или UI-логику данного участка кода.
    private void submit() {
        if (equipment == null) {
            Toast.makeText(this, R.string.equipment_not_found, Toast.LENGTH_SHORT).show();
            return;
        }

        String qtyStr = editQuantity.getText().toString().trim();
        String startStr = editStartDate.getText().toString().trim();
        String endStr = editEndDate.getText().toString().trim();

        if (TextUtils.isEmpty(qtyStr) || TextUtils.isEmpty(startStr) || TextUtils.isEmpty(endStr)) {
            Toast.makeText(this, R.string.fill_required_fields, Toast.LENGTH_SHORT).show();
            return;
        }

        int qty = Integer.parseInt(qtyStr);
        if (qty <= 0 || qty > equipment.getAvailableQuantity()) {
            Toast.makeText(this, R.string.invalid_quantity, Toast.LENGTH_SHORT).show();
            return;
        }

        if (endCalendar.getTimeInMillis() <= startCalendar.getTimeInMillis()) {
            Toast.makeText(this, R.string.invalid_dates, Toast.LENGTH_SHORT).show();
            return;
        }

        String permissionType = spinnerPermissionType.getSelectedItemPosition() == 1 ? "external" : "internal";
        String purpose = editPurpose.getText().toString().trim();

        Map<String, Object> data = new HashMap<>();
        data.put("equipment_id", equipment.getId());
        data.put("quantity", qty);
        data.put("permission_type", permissionType);
        data.put("purpose", TextUtils.isEmpty(purpose) ? null : purpose);
        data.put("start_date", buildIso(startCalendar));
        data.put("end_date", buildIso(endCalendar));

        progressBar.setVisibility(View.VISIBLE);
        buttonCreate.setEnabled(false);

        apiService.createBooking(data).enqueue(new Callback<Booking>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Booking> call, Response<Booking> response) {
                progressBar.setVisibility(View.GONE);
                buttonCreate.setEnabled(true);
                if (response.isSuccessful()) {
                    Toast.makeText(BookingFormActivity.this, R.string.booking_created, Toast.LENGTH_SHORT).show();
                    finish();
                } else {
                    Toast.makeText(BookingFormActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Booking> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                buttonCreate.setEnabled(true);
                Toast.makeText(BookingFormActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    // Метод formatLocal: выполняет основную бизнес- или UI-логику данного участка кода.
    private String formatLocal(Calendar calendar) {
        SimpleDateFormat fmt = new SimpleDateFormat("dd.MM.yyyy HH:mm", Locale.getDefault());
        return fmt.format(calendar.getTime());
    }

    // Метод buildIso: выполняет основную бизнес- или UI-логику данного участка кода.
    private String buildIso(Calendar calendar) {
        SimpleDateFormat fmt = new SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Locale.US);
        fmt.setTimeZone(TimeZone.getTimeZone("UTC"));
        return fmt.format(calendar.getTime());
    }
}
