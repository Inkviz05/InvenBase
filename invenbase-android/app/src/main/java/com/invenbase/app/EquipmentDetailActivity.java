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

import java.io.IOException;
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
        buttonDelete.setVisibility(canEdit ? View.VISIBLE : View.GONE);
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
        final android.widget.EditText input = new android.widget.EditText(this);
        input.setHint(equipment.getName());
        input.setMinEms(12);

        android.widget.LinearLayout wrap = new android.widget.LinearLayout(this);
        wrap.setOrientation(android.widget.LinearLayout.VERTICAL);
        int pad = (int) (40 * getResources().getDisplayMetrics().density);
        wrap.setPadding(pad, pad, pad, 0);
        wrap.addView(input);

        new android.app.AlertDialog.Builder(this)
                .setTitle(R.string.confirm_delete_equipment)
                .setMessage(getString(R.string.confirm_delete_equipment) + "\n\n" +
                        "Для подтверждения введите точное название оборудования.")
                .setView(wrap)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.delete, (dialog, which) -> {
                    String typed = input.getText().toString().trim();
                    if (!equipment.getName().equals(typed)) {
                        Toast.makeText(this, "Название оборудования не совпадает. Удаление отменено.", Toast.LENGTH_SHORT).show();
                        return;
                    }

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
        
        // Сначала получаем данные QR-кода (код, название, описание)
        apiService.getQRCodeData(equipment.getId()).enqueue(new Callback<Map<String, Object>>() {
            @Override
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    Map<String, Object> data = response.body();
                    final String qrData = data.get("qr_code") != null ? data.get("qr_code").toString() : null;
                    final String qrName = data.get("name") != null ? data.get("name").toString() : (equipment.getName() != null ? equipment.getName() : "");
                    String qrDesc = data.get("description") != null ? data.get("description").toString() : "";
                    final String qrDescFinal = (qrDesc != null && !qrDesc.trim().isEmpty()) ? qrDesc : "—";

                    apiService.generateQRCode(equipment.getId()).enqueue(new Callback<ResponseBody>() {
                        @Override
                        public void onResponse(Call<ResponseBody> call, Response<ResponseBody> response) {
                            if (response.isSuccessful() && response.body() != null) {
                                try {
                                    final byte[] bodyBytes = response.body().bytes();
                                    runOnUiThread(() -> {
                                        progressBar.setVisibility(View.GONE);
                                        showQRCodeDialog(bodyBytes, qrData, qrName, qrDescFinal);
                                    });
                                } catch (IOException e) {
                                    runOnUiThread(() -> {
                                        progressBar.setVisibility(View.GONE);
                                        Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                                    });
                                }
                            } else {
                                runOnUiThread(() -> {
                                    progressBar.setVisibility(View.GONE);
                                    Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                                });
                            }
                        }

                        @Override
                        public void onFailure(Call<ResponseBody> call, Throwable t) {
                            runOnUiThread(() -> {
                                progressBar.setVisibility(View.GONE);
                                Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                            });
                        }
                    });
                } else {
                    runOnUiThread(() -> {
                        progressBar.setVisibility(View.GONE);
                        Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                    });
                }
            }

            @Override
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                runOnUiThread(() -> {
                    progressBar.setVisibility(View.GONE);
                    Toast.makeText(EquipmentDetailActivity.this, R.string.error, Toast.LENGTH_SHORT).show();
                });
            }
        });
    }

    private static String trimTo(String s, int maxLen) {
        if (s == null) return "";
        return s.length() <= maxLen ? s : s.substring(0, maxLen - 1) + "…";
    }

    private android.graphics.Bitmap buildQRLabelBitmap(android.graphics.Bitmap qrBitmap, String qrData, String name, String desc) {
        float density = getResources().getDisplayMetrics().density;
        int qrSize = Math.min(qrBitmap.getWidth(), qrBitmap.getHeight());
        int padding = (int) (24 * density);
        int lineHeight = (int) (18 * density);
        int textSizePx = (int) (13 * density);
        // Ширина картинки с запасом для текста (минимум ~320dp), QR по центру
        int minWidth = (int) (320 * density);
        int width = Math.max(qrSize + padding * 2, minWidth);
        int height = padding + qrSize + padding + lineHeight * 3 + padding;

        android.graphics.Bitmap label = android.graphics.Bitmap.createBitmap(width, height, android.graphics.Bitmap.Config.ARGB_8888);
        android.graphics.Canvas canvas = new android.graphics.Canvas(label);
        canvas.drawColor(android.graphics.Color.WHITE);

        int qrX = (width - qrSize) / 2;
        android.graphics.Rect src = new android.graphics.Rect(0, 0, qrBitmap.getWidth(), qrBitmap.getHeight());
        android.graphics.Rect dst = new android.graphics.Rect(qrX, padding, qrX + qrSize, padding + qrSize);
        canvas.drawBitmap(qrBitmap, src, dst, null);

        android.graphics.Paint paint = new android.graphics.Paint(android.graphics.Paint.ANTI_ALIAS_FLAG);
        paint.setColor(android.graphics.Color.BLACK);
        paint.setTextSize(textSizePx);
        paint.setTextAlign(android.graphics.Paint.Align.CENTER);
        int centerX = width / 2;
        int y = padding + qrSize + padding + lineHeight;
        String codeText = trimTo(getString(R.string.qr_manual_code) + (qrData != null ? qrData : ""), 52);
        canvas.drawText(codeText, centerX, y, paint);
        y += lineHeight;
        String nameText = trimTo(getString(R.string.name) + ": " + (name != null ? name : ""), 52);
        canvas.drawText(nameText, centerX, y, paint);
        y += lineHeight;
        String descStr = (desc != null && !desc.equals("—")) ? desc : "—";
        String descText = trimTo(getString(R.string.description) + ": " + descStr, 52);
        canvas.drawText(descText, centerX, y, paint);
        return label;
    }

    private void showQRCodeDialog(byte[] qrBytes, String qrData, String qrName, String qrDesc) {
        try {
            android.graphics.Bitmap qrBitmap = android.graphics.BitmapFactory.decodeByteArray(qrBytes, 0, qrBytes.length);
            if (qrBitmap == null) {
                // Сервер может отдавать SVG (старая сборка) — BitmapFactory не декодирует SVG
                boolean looksLikeSvg = qrBytes.length > 10 && (
                    (qrBytes[0] == '<' && (qrBytes[1] == '?' || (qrBytes[1] == 's' && qrBytes[2] == 'v'))) ||
                    (qrBytes[0] == (byte) 0xef && qrBytes[1] == (byte) 0xbb && qrBytes[2] == (byte) 0xbf && qrBytes[3] == '<')
                );
                Toast.makeText(this, looksLikeSvg ? R.string.qr_server_needs_png : R.string.error, Toast.LENGTH_LONG).show();
                return;
            }
            android.graphics.Bitmap labelBitmap = buildQRLabelBitmap(qrBitmap, qrData, qrName, qrDesc);

            android.app.AlertDialog.Builder builder = new android.app.AlertDialog.Builder(this);
            View dialogView = getLayoutInflater().inflate(R.layout.dialog_qr_code, null);

            android.widget.ImageView imageView = dialogView.findViewById(R.id.image_qr_code);
            TextView textQrData = dialogView.findViewById(R.id.text_qr_data);
            Button buttonDownload = dialogView.findViewById(R.id.button_download_qr);
            Button buttonPrint = dialogView.findViewById(R.id.button_print_qr);

            imageView.setImageBitmap(qrBitmap);
            if (qrData != null) {
                textQrData.setText(getString(R.string.qr_manual_code) + qrData);
                textQrData.setVisibility(View.VISIBLE);
            } else {
                textQrData.setVisibility(View.GONE);
            }

            String baseName = equipment.getName() != null ? equipment.getName().replaceAll("[\\\\/:*?\"<>|\\p{Cntrl}]", "_").replaceAll("\\s+", " ").trim() : "";
            if (baseName.isEmpty()) baseName = equipment.getId() != null ? equipment.getId().length() >= 8 ? equipment.getId().substring(0, 8) : equipment.getId() : "qr";
            String filename = "qr-" + baseName + ".png";

            buttonDownload.setOnClickListener(v -> {
                android.content.ContentValues values = new android.content.ContentValues();
                values.put(android.provider.MediaStore.Images.Media.DISPLAY_NAME, filename);
                values.put(android.provider.MediaStore.Images.Media.MIME_TYPE, "image/png");
                android.net.Uri uri = getContentResolver().insert(
                    android.provider.MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values);
                try {
                    java.io.OutputStream out = getContentResolver().openOutputStream(uri);
                    labelBitmap.compress(android.graphics.Bitmap.CompressFormat.PNG, 100, out);
                    if (out != null) out.close();
                    Toast.makeText(this, getString(R.string.qr_code_saved), Toast.LENGTH_SHORT).show();
                } catch (Exception e) {
                    Toast.makeText(this, R.string.error, Toast.LENGTH_SHORT).show();
                }
            });

            if (buttonPrint != null) {
                buttonPrint.setVisibility(View.VISIBLE);
                buttonPrint.setOnClickListener(v -> {
                    try {
                        android.content.ContentValues values = new android.content.ContentValues();
                        values.put(android.provider.MediaStore.Images.Media.DISPLAY_NAME, filename);
                        values.put(android.provider.MediaStore.Images.Media.MIME_TYPE, "image/png");
                        android.net.Uri uri = getContentResolver().insert(
                            android.provider.MediaStore.Images.Media.EXTERNAL_CONTENT_URI, values);
                        if (uri != null) {
                            java.io.OutputStream out = getContentResolver().openOutputStream(uri);
                            labelBitmap.compress(android.graphics.Bitmap.CompressFormat.PNG, 100, out);
                            if (out != null) out.close();
                            Intent intent = new Intent(Intent.ACTION_SEND);
                            intent.setType("image/png");
                            intent.putExtra(Intent.EXTRA_STREAM, uri);
                            intent.addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION);
                            startActivity(Intent.createChooser(intent, getString(R.string.print_or_share)));
                        } else {
                            Toast.makeText(this, R.string.error, Toast.LENGTH_SHORT).show();
                        }
                    } catch (Exception e) {
                        Toast.makeText(this, R.string.error, Toast.LENGTH_SHORT).show();
                    }
                });
            }

            builder.setView(dialogView);
            builder.setPositiveButton(R.string.cancel, null);
            builder.setTitle(R.string.qr_code);
            builder.show();
        } catch (Exception e) {
            Toast.makeText(this, R.string.error, Toast.LENGTH_SHORT).show();
        }
    }
}
