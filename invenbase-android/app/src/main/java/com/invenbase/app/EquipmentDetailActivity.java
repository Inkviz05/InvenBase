package com.invenbase.app;

import android.content.Context;
import android.content.Intent;
import android.os.Bundle;
import android.view.View;
import android.widget.Button;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.Nullable;
import androidx.appcompat.app.AppCompatActivity;

import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Equipment;
import com.invenbase.app.utils.AuthManager;
import com.invenbase.app.utils.CartManager;

import java.text.SimpleDateFormat;
import java.util.Date;
import java.util.HashMap;
import java.util.Locale;
import java.util.Map;
import java.util.TimeZone;

import okhttp3.ResponseBody;
import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class EquipmentDetailActivity extends BaseActivity {
    public static final String EXTRA_EQUIPMENT_ID = "equipment_id";

    private ApiService apiService;
    private CartManager cartManager;
    private AuthManager authManager;
    private TextView textName;
    private TextView textDescription;
    private TextView textCategory;
    private TextView textQuantity;
    private TextView textAvailable;
    private TextView textLocation;
    private TextView textStatus;
    private TextView textQr;
    private ProgressBar progressBar;
    private View contentContainer;
    private Button buttonAddToCart;
    private Button buttonBookOne;
    private Button buttonOpenCart;
    private Button buttonEdit;
    private Button buttonQrCode;
    private Button buttonDelete;
    private Equipment equipment;

    public static void open(Context context, String equipmentId) {
        Intent intent = new Intent(context, EquipmentDetailActivity.class);
        intent.putExtra(EXTRA_EQUIPMENT_ID, equipmentId);
        context.startActivity(intent);
    }

    @Override
    protected void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_equipment_detail);

        apiService = ApiClient.getInstance(this).getApiService();
        cartManager = new CartManager(this);
        authManager = AuthManager.getInstance(this);

        textName = findViewById(R.id.text_equipment_name);
        textDescription = findViewById(R.id.text_equipment_description);
        textCategory = findViewById(R.id.text_equipment_category);
        textQuantity = findViewById(R.id.text_equipment_quantity);
        textAvailable = findViewById(R.id.text_equipment_available);
        textLocation = findViewById(R.id.text_equipment_location);
        textStatus = findViewById(R.id.text_equipment_status);
        textQr = findViewById(R.id.text_equipment_qr);
        progressBar = findViewById(R.id.progress_bar);
        contentContainer = findViewById(R.id.content_container);
        buttonAddToCart = findViewById(R.id.button_add_to_cart);
        buttonBookOne = findViewById(R.id.button_book_one);
        buttonOpenCart = findViewById(R.id.button_open_cart);
        buttonEdit = findViewById(R.id.button_edit);
        buttonQrCode = findViewById(R.id.button_qr_code);
        buttonDelete = findViewById(R.id.button_delete);

        buttonOpenCart.setOnClickListener(v -> startActivity(new Intent(this, CartActivity.class)));
        buttonAddToCart.setOnClickListener(v -> addToCart());
        buttonBookOne.setOnClickListener(v -> openBookingForm());
        buttonEdit.setOnClickListener(v -> openEditForm());
        buttonQrCode.setOnClickListener(v -> generateQRCode());
        buttonDelete.setOnClickListener(v -> deleteEquipment());

        String equipmentId = getIntent().getStringExtra(EXTRA_EQUIPMENT_ID);
        if (equipmentId == null || equipmentId.trim().isEmpty()) {
            Toast.makeText(this, R.string.error, Toast.LENGTH_SHORT).show();
            finish();
            return;
        }

        setTitle(getString(R.string.equipment_detail));
        loadEquipment(equipmentId);
    }

    private void loadEquipment(String equipmentId) {
        progressBar.setVisibility(View.VISIBLE);
        contentContainer.setVisibility(View.GONE);

        apiService.getEquipmentById(equipmentId).enqueue(new Callback<Equipment>() {
            @Override
            public void onResponse(Call<Equipment> call, Response<Equipment> response) {
                progressBar.setVisibility(View.GONE);
                if (response.isSuccessful() && response.body() != null) {
                    equipment = response.body();
                    bindEquipment();
                } else {
                    Toast.makeText(EquipmentDetailActivity.this, R.string.equipment_not_found, Toast.LENGTH_SHORT).show();
                    finish();
                }
            }

            @Override
            public void onFailure(Call<Equipment> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                finish();
            }
        });
    }

    private void bindEquipment() {
        contentContainer.setVisibility(View.VISIBLE);
        textName.setText(equipment.getName());
        textDescription.setText(equipment.getDescription() != null ? equipment.getDescription() : getString(R.string.no_data));
        textCategory.setText(equipment.getCategoryName() != null ? equipment.getCategoryName() : getString(R.string.no_data));
        textQuantity.setText(String.valueOf(equipment.getQuantity()));
        textAvailable.setText(String.valueOf(equipment.getAvailableQuantity()));
        textLocation.setText(equipment.getLocation() != null ? equipment.getLocation() : getString(R.string.no_data));
        textStatus.setText(equipment.getStatus() != null ? equipment.getStatus() : getString(R.string.no_data));
        textQr.setText(equipment.getQrCode() != null ? equipment.getQrCode() : getString(R.string.no_data));

        boolean inCart = cartManager.isInCart(equipment.getId());
        buttonAddToCart.setText(inCart ? getString(R.string.in_cart) : getString(R.string.add_to_cart));
        buttonAddToCart.setEnabled(!inCart);
        buttonBookOne.setEnabled(equipment.getAvailableQuantity() > 0);

        boolean canEdit = authManager.isAdmin() || authManager.isResponsible();
        buttonEdit.setVisibility(canEdit ? View.VISIBLE : View.GONE);
        buttonQrCode.setVisibility(canEdit ? View.VISIBLE : View.GONE);
        buttonDelete.setVisibility(authManager.isAdmin() ? View.VISIBLE : View.GONE);
    }

    private void addToCart() {
        if (equipment == null) {
            return;
        }
        cartManager.addItem(equipment, 1);
        Toast.makeText(this, R.string.added_to_cart, Toast.LENGTH_SHORT).show();
        bindEquipment();
    }

    private void openBookingForm() {
        if (equipment == null) {
            return;
        }
        if (equipment.getAvailableQuantity() <= 0) {
            Toast.makeText(this, R.string.not_available, Toast.LENGTH_SHORT).show();
            return;
        }
        Intent intent = new Intent(this, BookingFormActivity.class);
        intent.putExtra(BookingFormActivity.EXTRA_EQUIPMENT_ID, equipment.getId());
        startActivity(intent);
    }

    private void openEditForm() {
        if (equipment == null) return;
        Intent intent = new Intent(this, EquipmentFormActivity.class);
        intent.putExtra(EquipmentFormActivity.EXTRA_EQUIPMENT_ID, equipment.getId());
        intent.putExtra(EquipmentFormActivity.EXTRA_IS_EDIT, true);
        startActivity(intent);
    }

    private void deleteEquipment() {
        if (equipment == null) return;
        new android.app.AlertDialog.Builder(this)
                .setMessage(R.string.confirm_delete_equipment)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.delete, (dialog, which) -> {
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.deleteEquipment(equipment.getId()).enqueue(new Callback<Void>() {
                        @Override
                        public void onResponse(Call<Void> call, Response<Void> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful()) {
                                Toast.makeText(EquipmentDetailActivity.this, R.string.equipment_deleted, Toast.LENGTH_SHORT).show();
                                finish();
                            } else {
                                Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                            }
                        }

                        @Override
                        public void onFailure(Call<Void> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                        }
                    });
                })
                .show();
    }

    private void generateQRCode() {
        if (equipment == null) return;
        
        progressBar.setVisibility(View.VISIBLE);
        
        // Сначала получаем данные QR-кода
        apiService.getQRCodeData(equipment.getId()).enqueue(new Callback<Map<String, Object>>() {
            @Override
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    String qrData = (String) response.body().get("qr_code");
                    
                    // Затем генерируем изображение QR-кода
                    apiService.generateQRCode(equipment.getId()).enqueue(new Callback<ResponseBody>() {
                        @Override
                        public void onResponse(Call<ResponseBody> call, Response<ResponseBody> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful() && response.body() != null) {
                                showQRCodeDialog(response.body(), qrData);
                            } else {
                                Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                            }
                        }

                        @Override
                        public void onFailure(Call<ResponseBody> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                        }
                    });
                } else {
                    progressBar.setVisibility(View.GONE);
                    Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
            }
        });
    }

    private void showQRCodeDialog(ResponseBody qrBody, String qrData) {
        try {
            byte[] qrBytes = qrBody.bytes();
            android.graphics.Bitmap qrBitmap = android.graphics.BitmapFactory.decodeByteArray(qrBytes, 0, qrBytes.length);
            
            android.app.AlertDialog.Builder builder = new android.app.AlertDialog.Builder(this);
            View dialogView = getLayoutInflater().inflate(R.layout.dialog_qr_code, null);
            
            android.widget.ImageView imageView = dialogView.findViewById(R.id.image_qr_code);
            TextView textQrData = dialogView.findViewById(R.id.text_qr_data);
            Button buttonDownload = dialogView.findViewById(R.id.button_download_qr);
            
            imageView.setImageBitmap(qrBitmap);
            if (qrData != null) {
                textQrData.setText(qrData);
            } else {
                textQrData.setVisibility(View.GONE);
            }
            
            buttonDownload.setOnClickListener(v -> {
                // Сохраняем QR-код в галерею
                String filename = "qr-code-" + equipment.getId() + ".png";
                android.content.ContentValues values = new android.content.ContentValues();
                values.put(android.provider.MediaStore.Images.Media.DISPLAY_NAME, filename);
                values.put(android.provider.MediaStore.Images.Media.MIME_TYPE, "image/png");
                
                android.net.Uri uri = getContentResolver().insert(
                    android.provider.MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values);
                
                try {
                    java.io.OutputStream out = getContentResolver().openOutputStream(uri);
                    qrBitmap.compress(android.graphics.Bitmap.CompressFormat.PNG, 100, out);
                    out.close();
                    Toast.makeText(this, getString(R.string.qr_code_saved), Toast.LENGTH_SHORT).show();
                } catch (Exception e) {
                    Toast.makeText(this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            });
            
            builder.setView(dialogView);
            builder.setPositiveButton(R.string.cancel, null);
            builder.setTitle(R.string.qr_code);
            builder.show();
        } catch (Exception e) {
            Toast.makeText(this, R.string.error, Toast.LENGTH_SHORT).show();
        }
    }
}
