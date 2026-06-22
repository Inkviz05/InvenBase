package com.invenbase.app.fragments;

import android.Manifest;
import android.content.pm.PackageManager;
import android.app.AlertDialog;
import android.content.Intent;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.text.InputType;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.EditText;
import android.widget.ProgressBar;
import android.widget.TextView;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;
import androidx.fragment.app.Fragment;

import com.invenbase.app.CartActivity;
import com.invenbase.app.EquipmentDetailActivity;
import com.invenbase.app.R;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Equipment;
import com.journeyapps.barcodescanner.BarcodeCallback;
import com.journeyapps.barcodescanner.BarcodeResult;
import com.journeyapps.barcodescanner.DecoratedBarcodeView;
import com.journeyapps.barcodescanner.DefaultDecoderFactory;

import java.util.Arrays;
import java.util.Collection;
import java.util.Collections;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class ScannerFragment extends Fragment {
    private DecoratedBarcodeView barcodeView;
    private static final int CAMERA_PERMISSION_REQUEST = 100;
    private static final long SCAN_TIMEOUT_MS = 10000; // 10 секунд таймаут
    private static final long RESUME_DELAY_MS = 500; // Задержка перед возобновлением сканирования
    private static final long SCAN_COOLDOWN_MS = 1500; // Защита от повторного сканирования (уменьшено для быстрого отклика)
    private ApiService apiService;
    private boolean isHandlingResult = false;
    private Handler handler;
    private ProgressBar progressBar;
    private TextView statusText;
    private Runnable timeoutRunnable;
    private long lastScanTime = 0;
    private View startScanContainer;
    private android.widget.Button buttonStartScanning;
    private android.widget.Button buttonStopScanning;
    private boolean isScanning = false;

    @Nullable
    @Override
    // Метод onCreateView: обрабатывает соответствующее событие приложения.
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container, @Nullable Bundle savedInstanceState) {
        View view = inflater.inflate(R.layout.fragment_scanner, container, false);
        
        barcodeView = view.findViewById(R.id.barcode_scanner);
        progressBar = view.findViewById(R.id.progress_scanning);
        statusText = view.findViewById(R.id.text_scan_status);
        startScanContainer = view.findViewById(R.id.start_scan_container);
        buttonStartScanning = view.findViewById(R.id.button_start_scanning);
        buttonStopScanning = view.findViewById(R.id.button_stop_scanning);
        apiService = ApiClient.getInstance(requireContext()).getApiService();
        handler = new Handler(Looper.getMainLooper());

        buttonStartScanning.setOnClickListener(v -> {
            if (ContextCompat.checkSelfPermission(requireContext(), Manifest.permission.CAMERA) 
                    == PackageManager.PERMISSION_GRANTED) {
                startScanning();
            } else {
                ActivityCompat.requestPermissions(requireActivity(), 
                    new String[]{Manifest.permission.CAMERA}, 
                    CAMERA_PERMISSION_REQUEST);
            }
        });
        
        buttonStopScanning.setOnClickListener(v -> stopScanning());
        
        // Оптимизация настроек сканера
        setupScannerSettings();
        
        // Показываем начальный экран
        showStartScreen();
        
        return view;
    }
    
    // Метод setupScannerSettings: устанавливает или обновляет значение данных.
    private void setupScannerSettings() {
        // Настройка для более быстрого сканирования - только QR-коды
        Collection<com.google.zxing.BarcodeFormat> formats = Arrays.asList(
            com.google.zxing.BarcodeFormat.QR_CODE
        );
        barcodeView.getBarcodeView().setDecoderFactory(new DefaultDecoderFactory(formats));
        
        // Оптимизация настроек камеры для быстрого сканирования
        // Фонарик выключен по умолчанию через настройки DecoratedBarcodeView
    }

    // Метод startScanning: выполняет основную бизнес- или UI-логику данного участка кода.
    private void startScanning() {
        if (barcodeView == null) return;
        
        isScanning = true;
        isHandlingResult = false;
        showScanningScreen();
        updateScanStatus(getString(R.string.scanning_in_progress), false);
        barcodeView.resume();
        
        barcodeView.decodeContinuous(new BarcodeCallback() {
            @Override
            // Метод barcodeResult: выполняет основную бизнес- или UI-логику данного участка кода.
            public void barcodeResult(BarcodeResult result) {
                if (result.getText() != null && !isHandlingResult) {
                    long currentTime = System.currentTimeMillis();
                    // Защита от повторного сканирования одного и того же кода
                    if (currentTime - lastScanTime < SCAN_COOLDOWN_MS) {
                        return;
                    }
                    lastScanTime = currentTime;
                    
                    isHandlingResult = true;
                    cancelTimeout();
                    barcodeView.pause();
                    
                    // Визуальная обратная связь - вибрация
                    try {
                        android.os.Vibrator vibrator = (android.os.Vibrator) requireContext().getSystemService(android.content.Context.VIBRATOR_SERVICE);
                        if (vibrator != null && vibrator.hasVibrator()) {
                            vibrator.vibrate(android.os.VibrationEffect.createOneShot(100, android.os.VibrationEffect.DEFAULT_AMPLITUDE));
                        }
                    } catch (Exception e) {
                        // Игнорируем ошибки вибрации
                    }
                    
                    updateScanStatus(getString(R.string.processing_qr), true);
                    handleQrCode(result.getText());
                }
            }

            @Override
            // Метод possibleResultPoints: выполняет основную бизнес- или UI-логику данного участка кода.
            public void possibleResultPoints(java.util.List<com.google.zxing.ResultPoint> resultPoints) {
                // Визуальная обратная связь при обнаружении QR-кода
                if (!isHandlingResult && resultPoints != null && !resultPoints.isEmpty()) {
                    updateScanStatus(getString(R.string.qr_detected), false);
                }
            }
        });
        
        // Устанавливаем таймаут для автоматического перезапуска при зависании
        startTimeout();
    }
    
    // Метод startTimeout: выполняет основную бизнес- или UI-логику данного участка кода.
    private void startTimeout() {
        cancelTimeout();
        timeoutRunnable = () -> {
            if (!isHandlingResult && isScanning) {
                android.util.Log.d("ScannerFragment", "Scan timeout, resuming scanner");
                if (barcodeView != null && ContextCompat.checkSelfPermission(requireContext(), Manifest.permission.CAMERA) 
                        == PackageManager.PERMISSION_GRANTED) {
                    barcodeView.resume();
                }
            }
        };
        handler.postDelayed(timeoutRunnable, SCAN_TIMEOUT_MS);
    }
    
    // Метод restartScanner: выполняет основную бизнес- или UI-логику данного участка кода.
    private void restartScanner() {
        if (barcodeView != null && isScanning) {
            barcodeView.pause();
            handler.postDelayed(() -> {
                if (getContext() != null && !isHandlingResult && isScanning) {
                    if (ContextCompat.checkSelfPermission(requireContext(), Manifest.permission.CAMERA) 
                            == PackageManager.PERMISSION_GRANTED) {
                        barcodeView.resume();
                    }
                }
            }, 500);
        }
    }
    
    // Метод cancelTimeout: выполняет основную бизнес- или UI-логику данного участка кода.
    private void cancelTimeout() {
        if (timeoutRunnable != null) {
            handler.removeCallbacks(timeoutRunnable);
            timeoutRunnable = null;
        }
    }
    
    // Метод updateScanStatus: выполняет основную бизнес- или UI-логику данного участка кода.
    private void updateScanStatus(String status, boolean showProgress) {
        if (statusText != null) {
            statusText.setText(status);
        }
        if (progressBar != null) {
            progressBar.setVisibility(showProgress ? View.VISIBLE : View.GONE);
        }
    }
    
    // Метод stopScanning: выполняет основную бизнес- или UI-логику данного участка кода.
    private void stopScanning() {
        cancelTimeout();
        isScanning = false;
        isHandlingResult = false;
        lastScanTime = 0;
        if (barcodeView != null) {
            barcodeView.pause();
        }
        showStartScreen();
    }
    
    // Метод showStartScreen: выполняет основную бизнес- или UI-логику данного участка кода.
    private void showStartScreen() {
        if (startScanContainer != null) {
            startScanContainer.setVisibility(View.VISIBLE);
        }
        if (barcodeView != null) {
            barcodeView.setVisibility(View.GONE);
        }
        if (statusText != null && statusText.getParent() != null) {
            ((View) statusText.getParent()).setVisibility(View.GONE);
        }
        if (buttonStopScanning != null) {
            buttonStopScanning.setVisibility(View.GONE);
        }
    }
    
    // Метод showScanningScreen: выполняет основную бизнес- или UI-логику данного участка кода.
    private void showScanningScreen() {
        if (startScanContainer != null) {
            startScanContainer.setVisibility(View.GONE);
        }
        if (barcodeView != null) {
            barcodeView.setVisibility(View.VISIBLE);
        }
        if (statusText != null && statusText.getParent() != null) {
            ((View) statusText.getParent()).setVisibility(View.VISIBLE);
        }
        if (buttonStopScanning != null) {
            buttonStopScanning.setVisibility(View.VISIBLE);
        }
    }

    @Override
    // Метод onRequestPermissionsResult: обрабатывает соответствующее событие приложения.
    public void onRequestPermissionsResult(int requestCode, @NonNull String[] permissions, @NonNull int[] grantResults) {
        if (requestCode == CAMERA_PERMISSION_REQUEST) {
            if (grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                // Разрешение получено, но не запускаем автоматически - пользователь нажмет кнопку
            } else {
                Toast.makeText(requireContext(), R.string.camera_permission_required, Toast.LENGTH_LONG).show();
            }
        }
    }

    @Override
    // Метод onResume: обрабатывает соответствующее событие приложения.
    public void onResume() {
        super.onResume();
        // Возобновляем сканирование только если оно было активно
        if (isScanning && !isHandlingResult) {
            handler.postDelayed(() -> {
                if (getContext() != null && ContextCompat.checkSelfPermission(requireContext(), Manifest.permission.CAMERA) 
                        == PackageManager.PERMISSION_GRANTED) {
                    if (barcodeView != null) {
                        barcodeView.resume();
                    }
                }
            }, 300);
        }
    }

    @Override
    // Метод onPause: обрабатывает соответствующее событие приложения.
    public void onPause() {
        super.onPause();
        cancelTimeout();
        isHandlingResult = false;
        if (barcodeView != null) {
            barcodeView.pause();
        }
    }
    
    @Override
    // Метод onDestroyView: обрабатывает соответствующее событие приложения.
    public void onDestroyView() {
        super.onDestroyView();
        cancelTimeout();
        if (handler != null) {
            handler.removeCallbacksAndMessages(null);
        }
    }

    // Метод handleQrCode: выполняет основную бизнес- или UI-логику данного участка кода.
    private void handleQrCode(String qrCode) {
        android.util.Log.d("ScannerFragment", "Processing QR code: " + qrCode);
        updateScanStatus(getString(R.string.searching_equipment), true);
        
        Call<Equipment> call = apiService.getEquipmentByQR(qrCode);
        call.enqueue(new Callback<Equipment>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Equipment> call, Response<Equipment> response) {
                if (response.isSuccessful() && response.body() != null) {
                    Equipment equipment = response.body();
                    updateScanStatus(getString(R.string.equipment_found) + ": " + equipment.getName(), false);
                    
                    // Успешная вибрация
                    try {
                        android.os.Vibrator vibrator = (android.os.Vibrator) requireContext().getSystemService(android.content.Context.VIBRATOR_SERVICE);
                        if (vibrator != null && vibrator.hasVibrator()) {
                            vibrator.vibrate(android.os.VibrationEffect.createOneShot(200, android.os.VibrationEffect.DEFAULT_AMPLITUDE));
                        }
                    } catch (Exception e) {
                        // Игнорируем ошибки вибрации
                    }
                    
                    // Небольшая задержка для показа сообщения, затем переход
                    handler.postDelayed(() -> {
                        if (getContext() != null && getActivity() != null) {
                            // Останавливаем сканирование перед переходом
                            stopScanning();
                            openEquipmentDetails(equipment.getId());
                        }
                    }, 800);
                } else {
                    updateScanStatus(getString(R.string.equipment_not_found), false);
                    Toast.makeText(requireContext(), R.string.equipment_not_found, Toast.LENGTH_LONG).show();
                    handler.postDelayed(() -> {
                        if (getContext() != null) {
                            resumeScanning();
                        }
                    }, 2000);
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Equipment> call, Throwable t) {
                android.util.Log.e("ScannerFragment", "Error fetching equipment", t);
                updateScanStatus(getString(R.string.error_occurred), false);
                String errorMsg = t.getMessage();
                if (errorMsg == null || errorMsg.isEmpty()) {
                    errorMsg = getString(R.string.error);
                }
                Toast.makeText(requireContext(), errorMsg, Toast.LENGTH_LONG).show();
                handler.postDelayed(() -> {
                    if (getContext() != null) {
                        resumeScanning();
                    }
                }, 2000);
            }
        });
    }

    // Метод openEquipmentDetails: выполняет основную бизнес- или UI-логику данного участка кода.
    private void openEquipmentDetails(String equipmentId) {
        // Останавливаем сканер перед переходом
        if (barcodeView != null) {
            barcodeView.pause();
        }
        cancelTimeout();
        
        Intent intent = new Intent(requireContext(), EquipmentDetailActivity.class);
        intent.putExtra(EquipmentDetailActivity.EXTRA_EQUIPMENT_ID, equipmentId);
        startActivity(intent);
    }

    // Метод resumeScanning: выполняет основную бизнес- или UI-логику данного участка кода.
    private void resumeScanning() {
        handler.postDelayed(() -> {
            if (getContext() != null && !isHandlingResult && isScanning) {
                isHandlingResult = false;
                lastScanTime = 0; // Сбрасываем защиту от повторного сканирования
                if (ContextCompat.checkSelfPermission(requireContext(), Manifest.permission.CAMERA) 
                        == PackageManager.PERMISSION_GRANTED) {
                    if (barcodeView != null) {
                        barcodeView.resume();
                    }
                }
            }
        }, RESUME_DELAY_MS);
    }

    // Метод showManualInputDialog: выполняет основную бизнес- или UI-логику данного участка кода.
    private void showManualInputDialog() {
        // Останавливаем сканирование только если оно активно
        if (isScanning && barcodeView != null) {
            barcodeView.pause();
        }
        cancelTimeout();
        
        EditText input = new EditText(requireContext());
        input.setInputType(InputType.TYPE_CLASS_TEXT);
        input.setHint(getString(R.string.manual_input_hint));
        input.setPadding(32, 32, 32, 32);

        new AlertDialog.Builder(requireContext())
            .setTitle(R.string.manual_input)
            .setView(input)
            .setPositiveButton(R.string.find_equipment, (dialog, which) -> {
                String code = input.getText().toString().trim();
                if (code.isEmpty()) {
                    Toast.makeText(requireContext(), R.string.enter_code, Toast.LENGTH_SHORT).show();
                    if (isScanning) {
                        resumeScanning();
                    }
                    return;
                }
                isHandlingResult = true;
                handleQrCode(code);
            })
            .setNegativeButton(R.string.cancel, (dialog, which) -> {
                if (isScanning) {
                    resumeScanning();
                }
            })
            .setOnCancelListener(dialog -> {
                if (isScanning) {
                    resumeScanning();
                }
            })
            .show();
    }
}
