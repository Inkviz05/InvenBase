package com.invenbase.app;

import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.adapters.CartAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Booking;
import com.invenbase.app.models.CartItem;
import com.invenbase.app.utils.CartManager;

import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
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

    private void loadCart() {
        List<CartItem> items = cartManager.getItems();
        adapter.setItems(items);
        boolean hasItems = !items.isEmpty();
        recyclerCart.setVisibility(hasItems ? View.VISIBLE : View.GONE);
        buttonCreateBookings.setEnabled(hasItems);
        textEmpty.setVisibility(hasItems ? View.GONE : View.VISIBLE);
    }

    private void createBulkBookings() {
        List<CartItem> items = cartManager.getItems();
        if (items.isEmpty()) {
            return;
        }

        progressBar.setVisibility(View.VISIBLE);
        buttonCreateBookings.setEnabled(false);
        createNextBooking(items, 0);
    }

    private void createNextBooking(List<CartItem> items, int index) {
        if (index >= items.size()) {
            progressBar.setVisibility(View.GONE);
            cartManager.clear();
            loadCart();
            Toast.makeText(this, R.string.booking_created, Toast.LENGTH_SHORT).show();
            return;
        }

        CartItem item = items.get(index);
        Map<String, Object> data = buildDefaultBookingPayload(item.getEquipmentId(), item.getQuantity());
        apiService.createBooking(data).enqueue(new Callback<Booking>() {
            @Override
            public void onResponse(Call<Booking> call, Response<Booking> response) {
                if (response.isSuccessful()) {
                    createNextBooking(items, index + 1);
                } else {
                    progressBar.setVisibility(View.GONE);
                    buttonCreateBookings.setEnabled(true);
                    Toast.makeText(CartActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<Booking> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                buttonCreateBookings.setEnabled(true);
                Toast.makeText(CartActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    private Map<String, Object> buildDefaultBookingPayload(String equipmentId, int quantity) {
        Map<String, Object> data = new HashMap<>();
        data.put("equipment_id", equipmentId);
        data.put("quantity", quantity);
        data.put("permission_type", "internal");
        data.put("purpose", "QR");
        data.put("start_date", buildIsoDate(new Date()));
        data.put("end_date", buildIsoDate(new Date(System.currentTimeMillis() + 24L * 60L * 60L * 1000L)));
        return data;
    }

    private String buildIsoDate(Date date) {
        SimpleDateFormat format = new SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss.SSS'Z'", Locale.US);
        format.setTimeZone(TimeZone.getTimeZone("UTC"));
        return format.format(date);
    }
}
