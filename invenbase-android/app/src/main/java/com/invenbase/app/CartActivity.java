package com.invenbase.app;

import android.app.DatePickerDialog;
import android.app.TimePickerDialog;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.google.gson.Gson;
import com.invenbase.app.adapters.CartAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.ApiError;
import com.invenbase.app.models.Booking;
import com.invenbase.app.models.BulkBookingsRequest;
import com.invenbase.app.models.CartItem;
import com.invenbase.app.models.CreateBookingItem;
import com.invenbase.app.utils.CartManager;

import java.io.IOException;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Calendar;
import java.util.List;
import java.util.Date;
import java.util.Locale;
import java.util.TimeZone;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class CartActivity extends BaseActivity {
    private CartManager cartManager;
    private ApiService apiService;
    private CartAdapter adapter;
    private RecyclerView recyclerCart;
    private TextView textEmpty;
    private Button buttonCreateBookings;
    private ProgressBar progressBar;
    private View cardPeriod;
    private EditText editPeriodStart;
    private EditText editPeriodEnd;
    private final Calendar startCalendar = Calendar.getInstance();
    private final Calendar endCalendar = Calendar.getInstance();

    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_cart);

        cartManager = new CartManager(this);
        apiService = ApiClient.getInstance(this).getApiService();

        recyclerCart = findViewById(R.id.recycler_cart);
        textEmpty = findViewById(R.id.text_cart_empty);
        buttonCreateBookings = findViewById(R.id.button_create_bookings);
        progressBar = findViewById(R.id.progress_bar);
        cardPeriod = findViewById(R.id.card_period);
        editPeriodStart = findViewById(R.id.edit_period_start);
        editPeriodEnd = findViewById(R.id.edit_period_end);

        initPeriodCalendars();
        editPeriodStart.setOnClickListener(v -> pickDateTime(startCalendar, editPeriodStart));
        editPeriodEnd.setOnClickListener(v -> pickDateTime(endCalendar, editPeriodEnd));

        adapter = new CartAdapter();
        recyclerCart.setLayoutManager(new LinearLayoutManager(this));
        recyclerCart.setAdapter(adapter);
        adapter.setOnCartActionListener(item -> {
            cartManager.removeItem(item.getEquipmentId());
            loadCart();
        });

        buttonCreateBookings.setOnClickListener(v -> createBulkBookings());
        setTitle(getString(R.string.cart_title));
        loadCart();
    }

    private void initPeriodCalendars() {
        // Инициализируем значения календарей, но поля оставляем пустыми —
        // пользователь должен сам выбрать период явно.
        startCalendar.setTimeInMillis(System.currentTimeMillis());
        endCalendar.setTimeInMillis(System.currentTimeMillis());
        endCalendar.add(Calendar.DAY_OF_MONTH, 1);
    }

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

    private String formatLocal(Calendar calendar) {
        SimpleDateFormat fmt = new SimpleDateFormat("dd.MM.yyyy HH:mm", Locale.getDefault());
        return fmt.format(calendar.getTime());
    }

    private void loadCart() {
        List<CartItem> items = cartManager.getItems();
        adapter.setItems(items);
        boolean hasItems = !items.isEmpty();
        recyclerCart.setVisibility(hasItems ? View.VISIBLE : View.GONE);
        buttonCreateBookings.setEnabled(hasItems);
        textEmpty.setVisibility(hasItems ? View.GONE : View.VISIBLE);
        cardPeriod.setVisibility(hasItems ? View.VISIBLE : View.GONE);
    }

    private void createBulkBookings() {
        List<CartItem> items = cartManager.getItems();
        if (items.isEmpty()) {
            return;
        }
        if (editPeriodStart.getText().toString().trim().isEmpty()
                || editPeriodEnd.getText().toString().trim().isEmpty()) {
            Toast.makeText(this, R.string.fill_period_fields, Toast.LENGTH_SHORT).show();
            return;
        }
        if (endCalendar.getTimeInMillis() <= startCalendar.getTimeInMillis()) {
            Toast.makeText(this, R.string.invalid_period, Toast.LENGTH_SHORT).show();
            return;
        }

        String startIso = buildIsoDate(new Date(startCalendar.getTimeInMillis()));
        String endIso = buildIsoDate(new Date(endCalendar.getTimeInMillis()));
        List<CreateBookingItem> bookings = new ArrayList<>();
        for (CartItem item : items) {
            bookings.add(new CreateBookingItem(
                    item.getEquipmentId(),
                    item.getQuantity(),
                    startIso,
                    endIso,
                    "QR",
                    "internal"
            ));
        }
        BulkBookingsRequest body = new BulkBookingsRequest(bookings);

        progressBar.setVisibility(View.VISIBLE);
        buttonCreateBookings.setEnabled(false);
        apiService.createBulkBookings(body).enqueue(new Callback<List<Booking>>() {
            @Override
            public void onResponse(Call<List<Booking>> call, Response<List<Booking>> response) {
                progressBar.setVisibility(View.GONE);
                buttonCreateBookings.setEnabled(true);
                if (response.isSuccessful() && response.body() != null) {
                    cartManager.clear();
                    loadCart();
                    Toast.makeText(CartActivity.this, R.string.booking_created, Toast.LENGTH_SHORT).show();
                } else {
                    String msg = getErrorMessage(response);
                    Toast.makeText(CartActivity.this, msg, Toast.LENGTH_LONG).show();
                }
            }

            @Override
            public void onFailure(Call<List<Booking>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                buttonCreateBookings.setEnabled(true);
                String msg = t.getMessage() != null ? t.getMessage() : getString(R.string.error);
                Toast.makeText(CartActivity.this, msg, Toast.LENGTH_LONG).show();
            }
        });
    }

    private String getErrorMessage(Response<?> response) {
        if (response.errorBody() == null) return getString(R.string.error);
        try {
            String body = response.errorBody().string();
            ApiError err = new Gson().fromJson(body, ApiError.class);
            if (err != null && err.getMessage() != null && !err.getMessage().isEmpty()) {
                return err.getMessage();
            }
        } catch (IOException ignored) { }
        return getString(R.string.error);
    }

    private String buildIsoDate(Date date) {
        SimpleDateFormat format = new SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Locale.US);
        format.setTimeZone(TimeZone.getTimeZone("UTC"));
        return format.format(date);
    }
}
