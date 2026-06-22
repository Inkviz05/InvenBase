package com.invenbase.app.fragments;

import android.content.Intent;
import android.os.Bundle;
import android.app.AlertDialog;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.EditText;
import android.widget.Toast;
import android.widget.ProgressBar;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.fragment.app.Fragment;
import androidx.recyclerview.widget.GridLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.google.android.material.bottomnavigation.BottomNavigationView;
import com.invenbase.app.BookingFormActivity;
import com.invenbase.app.CartActivity;
import com.invenbase.app.NotificationsActivity;
import com.invenbase.app.SupportActivity;
import com.invenbase.app.R;
import com.invenbase.app.adapters.StatsAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Booking;
import com.invenbase.app.models.Equipment;
import com.invenbase.app.models.StatItem;
import com.invenbase.app.utils.AuthManager;
import com.invenbase.app.utils.CartManager;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class DashboardFragment extends Fragment {
    private TextView textWelcome;
    private TextView textNotificationsInfo;
    private RecyclerView recyclerStats;
    private ProgressBar progressBar;
    private AuthManager authManager;
    private ApiService apiService;
    private StatsAdapter statsAdapter;
    private CartManager cartManager;
    private Button buttonQuickCart;
    private Button buttonQuickEquipment;
    private Button buttonQuickAddEquipment;
    private Button buttonQuickScanner;
    private Button buttonQuickManualInput;
    private Button buttonQuickSupport;
    private Button buttonViewNotifications;

    @Nullable
    @Override
    // Метод onCreateView: обрабатывает соответствующее событие приложения.
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container, @Nullable Bundle savedInstanceState) {
        View view = inflater.inflate(R.layout.fragment_dashboard, container, false);
        
        authManager = AuthManager.getInstance(requireContext());
        apiService = ApiClient.getInstance(requireContext()).getApiService();
        cartManager = new CartManager(requireContext());
        
        textWelcome = view.findViewById(R.id.text_welcome);
        textNotificationsInfo = view.findViewById(R.id.text_notifications_info);
        recyclerStats = view.findViewById(R.id.recycler_stats);
        progressBar = view.findViewById(R.id.progress_bar);
        buttonQuickCart = view.findViewById(R.id.button_quick_cart);
        buttonQuickEquipment = view.findViewById(R.id.button_quick_equipment);
        buttonQuickAddEquipment = view.findViewById(R.id.button_quick_add_equipment);
        buttonQuickScanner = view.findViewById(R.id.button_quick_scanner);
        buttonQuickManualInput = view.findViewById(R.id.button_quick_manual_input);
        buttonQuickSupport = view.findViewById(R.id.button_quick_support);
        buttonViewNotifications = view.findViewById(R.id.button_view_notifications);
        Button buttonLogout = view.findViewById(R.id.button_logout);

        recyclerStats.setLayoutManager(new GridLayoutManager(requireContext(), 2));
        statsAdapter = new StatsAdapter();
        recyclerStats.setAdapter(statsAdapter);

        if (textWelcome != null && authManager.getUser() != null) {
            String welcomeText = "Добро пожаловать, " +
                    (authManager.getUser().getFullName() != null && !authManager.getUser().getFullName().isEmpty()
                            ? authManager.getUser().getFullName()
                            : authManager.getUser().getUsername()) + "!";
            textWelcome.setText(welcomeText);
        }
        
        buttonLogout.setOnClickListener(v -> {
            authManager.logout();
            if (getActivity() != null) {
                startActivity(new Intent(getActivity(), com.invenbase.app.LoginActivity.class));
                getActivity().finish();
            }
        });

        setupQuickActions();
        loadStats();
        
        return view;
    }

    // Метод loadStats: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadStats() {
        progressBar.setVisibility(View.VISIBLE);
        if (authManager.isAdmin() || authManager.isResponsible()) {
            loadAdminStats();
        } else {
            loadUserStats();
        }
    }

    // Метод loadAdminStats: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadAdminStats() {
        apiService.getEquipmentReport().enqueue(new Callback<Map<String, Object>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> response) {
                if (response.isSuccessful() && response.body() != null) {
                    Map<String, Object> equipmentReport = response.body();
                    apiService.getBookingReport().enqueue(new Callback<Map<String, Object>>() {
                        @Override
                        // Метод onResponse: обрабатывает соответствующее событие приложения.
                        public void onResponse(Call<Map<String, Object>> call, Response<Map<String, Object>> bookingResponse) {
                            if (bookingResponse.isSuccessful() && bookingResponse.body() != null) {
                                Map<String, Object> bookingReport = bookingResponse.body();
                                apiService.getUnreadCount().enqueue(new Callback<Map<String, Integer>>() {
                                    @Override
                                    // Метод onResponse: обрабатывает соответствующее событие приложения.
                                    public void onResponse(Call<Map<String, Integer>> call, Response<Map<String, Integer>> notifResponse) {
                                        int unread = 0;
                                        if (notifResponse.isSuccessful() && notifResponse.body() != null && notifResponse.body().get("count") != null) {
                                            unread = notifResponse.body().get("count");
                                        }
                                        showAdminStats(equipmentReport, bookingReport, unread);
                                    }

                                    @Override
                                    // Метод onFailure: обрабатывает соответствующее событие приложения.
                                    public void onFailure(Call<Map<String, Integer>> call, Throwable t) {
                                        showAdminStats(equipmentReport, bookingReport, 0);
                                    }
                                });
                            } else {
                                showAdminStats(equipmentReport, null, 0);
                            }
                        }

                        @Override
                        // Метод onFailure: обрабатывает соответствующее событие приложения.
                        public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                            showAdminStats(equipmentReport, null, 0);
                        }
                    });
                } else {
                    showAdminStats(null, null, 0);
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<Map<String, Object>> call, Throwable t) {
                showAdminStats(null, null, 0);
            }
        });
    }

    // Метод loadUserStats: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadUserStats() {
        apiService.getEquipment().enqueue(new Callback<List<Equipment>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<List<Equipment>> call, Response<List<Equipment>> response) {
                List<Equipment> equipment = response.isSuccessful() && response.body() != null ? response.body() : new ArrayList<>();
                apiService.getBookings().enqueue(new Callback<List<Booking>>() {
                    @Override
                    // Метод onResponse: обрабатывает соответствующее событие приложения.
                    public void onResponse(Call<List<Booking>> call, Response<List<Booking>> response) {
                        List<Booking> bookings = response.isSuccessful() && response.body() != null ? response.body() : new ArrayList<>();
                        apiService.getUnreadCount().enqueue(new Callback<Map<String, Integer>>() {
                            @Override
                            // Метод onResponse: обрабатывает соответствующее событие приложения.
                            public void onResponse(Call<Map<String, Integer>> call, Response<Map<String, Integer>> notifResponse) {
                                int unread = 0;
                                if (notifResponse.isSuccessful() && notifResponse.body() != null && notifResponse.body().get("count") != null) {
                                    unread = notifResponse.body().get("count");
                                }
                                showUserStats(equipment, bookings, unread);
                            }

                            @Override
                            // Метод onFailure: обрабатывает соответствующее событие приложения.
                            public void onFailure(Call<Map<String, Integer>> call, Throwable t) {
                                showUserStats(equipment, bookings, 0);
                            }
                        });
                    }

                    @Override
                    // Метод onFailure: обрабатывает соответствующее событие приложения.
                    public void onFailure(Call<List<Booking>> call, Throwable t) {
                        showUserStats(equipment, new ArrayList<>(), 0);
                    }
                });
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<List<Equipment>> call, Throwable t) {
                showUserStats(new ArrayList<>(), new ArrayList<>(), 0);
            }
        });
    }

    // Метод setupQuickActions: устанавливает или обновляет значение данных.
    private void setupQuickActions() {
        // Обновляем видимость кнопки корзины
        updateCartButton();
        
        buttonQuickCart.setOnClickListener(v -> {
            if (getActivity() != null) {
                startActivity(new Intent(getActivity(), CartActivity.class));
            }
        });
        
        buttonQuickEquipment.setOnClickListener(v -> {
            if (getActivity() != null) {
                BottomNavigationView bottomNav = getActivity().findViewById(R.id.bottom_navigation);
                if (bottomNav != null) {
                    bottomNav.setSelectedItemId(R.id.nav_equipment);
                }
            }
        });

        buttonQuickAddEquipment.setOnClickListener(v -> {
            if (getActivity() != null && (authManager.isAdmin() || authManager.isResponsible())) {
                BottomNavigationView bottomNav = getActivity().findViewById(R.id.bottom_navigation);
                if (bottomNav != null) {
                    bottomNav.setSelectedItemId(R.id.nav_equipment);
                }
                // Дополнительно можно открыть экран добавления оборудования,
                // если он реализован отдельной Activity или через FAB на экране оборудования.
            }
        });

        buttonQuickScanner.setOnClickListener(v -> {
            if (getActivity() != null) {
                BottomNavigationView bottomNav = getActivity().findViewById(R.id.bottom_navigation);
                if (bottomNav != null) {
                    bottomNav.setSelectedItemId(R.id.nav_scanner);
                }
            }
        });

        buttonQuickManualInput.setOnClickListener(v -> showManualInputDialog());
        
        buttonViewNotifications.setOnClickListener(v -> {
            if (getActivity() != null) {
                startActivity(new Intent(getActivity(), NotificationsActivity.class));
            }
        });

        buttonQuickSupport.setOnClickListener(v -> {
            if (getActivity() != null) {
                startActivity(new Intent(getActivity(), SupportActivity.class));
            }
        });
    }
    
    // Метод updateCartButton: выполняет основную бизнес- или UI-логику данного участка кода.
    private void updateCartButton() {
        int cartCount = cartManager.getItems().size();
        if (cartCount > 0) {
            buttonQuickCart.setVisibility(View.VISIBLE);
            buttonQuickCart.setText(getString(R.string.book_from_cart) + " (" + cartCount + ")");
        } else {
            buttonQuickCart.setVisibility(View.GONE);
        }
        
        // Настраиваем быстрые действия по ролям
        boolean isAdminOrResponsible = authManager.isAdmin() || authManager.isResponsible();

        if (isAdminOrResponsible) {
            // Администратор / ответственный: выбрать оборудование, сканировать QR (кнопка «Добавить оборудование» — на вкладке Управление)
            buttonQuickEquipment.setVisibility(View.VISIBLE);
            buttonQuickAddEquipment.setVisibility(View.GONE);
            buttonQuickScanner.setVisibility(View.VISIBLE);
            buttonQuickManualInput.setVisibility(View.GONE);
        } else {
            // Обычный пользователь: сканировать QR, ввести вручную
            buttonQuickEquipment.setVisibility(View.GONE);
            buttonQuickAddEquipment.setVisibility(View.GONE);
            buttonQuickScanner.setVisibility(View.VISIBLE);
            buttonQuickManualInput.setVisibility(View.VISIBLE);
        }
    }

    // Метод showManualInputDialog: выполняет основную бизнес- или UI-логику данного участка кода.
    private void showManualInputDialog() {
        if (getContext() == null) return;

        final EditText input = new EditText(requireContext());
        input.setHint(R.string.manual_input_hint);

        new AlertDialog.Builder(requireContext())
            .setTitle(R.string.manual_input)
            .setView(input)
            .setPositiveButton(R.string.find_equipment, (dialog, which) -> {
                String code = input.getText().toString().trim();
                if (code.isEmpty()) {
                    Toast.makeText(requireContext(), R.string.enter_code, Toast.LENGTH_SHORT).show();
                    return;
                }

                // Ищем оборудование по введённому коду
                apiService.getEquipmentByQR(code).enqueue(new Callback<Equipment>() {
                    @Override
                    // Метод onResponse: обрабатывает соответствующее событие приложения.
                    public void onResponse(Call<Equipment> call, Response<Equipment> response) {
                        if (response.isSuccessful() && response.body() != null && getActivity() != null) {
                            Equipment equipment = response.body();
                            Intent intent = new Intent(getActivity(), com.invenbase.app.EquipmentDetailActivity.class);
                            intent.putExtra(com.invenbase.app.EquipmentDetailActivity.EXTRA_EQUIPMENT_ID, equipment.getId());
                            startActivity(intent);
                        } else {
                            Toast.makeText(requireContext(), R.string.equipment_not_found, Toast.LENGTH_LONG).show();
                        }
                    }

                    @Override
                    // Метод onFailure: обрабатывает соответствующее событие приложения.
                    public void onFailure(Call<Equipment> call, Throwable t) {
                        String errorMsg = t.getMessage();
                        if (errorMsg == null || errorMsg.isEmpty()) {
                            errorMsg = getString(R.string.error);
                        }
                        Toast.makeText(requireContext(), errorMsg, Toast.LENGTH_LONG).show();
                    }
                });
            })
            .setNegativeButton(R.string.cancel, (dialog, which) -> {
                // ничего не делаем
            })
            .show();
    }

    // Метод showUserStats: выполняет основную бизнес- или UI-логику данного участка кода.
    private void showUserStats(List<Equipment> equipment, List<Booking> bookings, int unread) {
        String userId = authManager.getUser() != null ? authManager.getUser().getId() : null;
        int myBookings = 0;
        int pending = 0;
        int approved = 0;
        for (Booking booking : bookings) {
            if (userId == null || userId.equals(booking.getUserId())) {
                myBookings++;
                if ("pending".equals(booking.getStatus())) {
                    pending++;
                } else if ("approved".equals(booking.getStatus())) {
                    approved++;
                }
            }
        }

        List<StatItem> items = new ArrayList<>();
        items.add(new StatItem(getString(R.string.bookings), String.valueOf(myBookings)));
        items.add(new StatItem(getString(R.string.pending), String.valueOf(pending)));
        items.add(new StatItem(getString(R.string.approved), String.valueOf(approved)));
        items.add(new StatItem(getString(R.string.notifications), String.valueOf(unread)));

        statsAdapter.setItems(items);
        updateNotificationsInfo(unread);
        updateCartButton();
        progressBar.setVisibility(View.GONE);
    }
    
    // Метод showAdminStats: выполняет основную бизнес- или UI-логику данного участка кода.
    private void showAdminStats(Map<String, Object> equipmentReport, Map<String, Object> bookingReport, int unread) {
        List<StatItem> items = new ArrayList<>();
        int totalEquipment = getInt(equipmentReport, "total_equipment");
        int availableEquipment = getInt(equipmentReport, "available_equipment");
        int bookedEquipment = getInt(equipmentReport, "booked_equipment");
        int pending = getInt(bookingReport, "pending");
        int approved = getInt(bookingReport, "approved");

        items.add(new StatItem(getString(R.string.total), String.valueOf(totalEquipment)));
        items.add(new StatItem(getString(R.string.available), String.valueOf(availableEquipment)));
        items.add(new StatItem(getString(R.string.booked), String.valueOf(bookedEquipment)));
        items.add(new StatItem(getString(R.string.pending), String.valueOf(pending)));
        items.add(new StatItem(getString(R.string.approved), String.valueOf(approved)));
        items.add(new StatItem(getString(R.string.notifications), String.valueOf(unread)));

        statsAdapter.setItems(items);
        updateNotificationsInfo(unread);
        updateCartButton();
        progressBar.setVisibility(View.GONE);
    }
    
    // Метод updateNotificationsInfo: выполняет основную бизнес- или UI-логику данного участка кода.
    private void updateNotificationsInfo(int unread) {
        if (unread > 0) {
            textNotificationsInfo.setText(getString(R.string.unread_notifications, unread));
            textNotificationsInfo.setTextColor(getResources().getColor(android.R.color.holo_blue_light, null));
        } else {
            textNotificationsInfo.setText(getString(R.string.no_notifications));
            textNotificationsInfo.setTextColor(getResources().getColor(android.R.color.darker_gray, null));
        }
    }

    // Метод getInt: возвращает нужное значение для текущего контекста.
    private int getInt(Map<String, Object> map, String key) {
        if (map == null || map.get(key) == null) {
            return 0;
        }
        Object value = map.get(key);
        if (value instanceof Number) {
            return ((Number) value).intValue();
        }
        return 0;
    }
    
    @Override
    // Метод onResume: обрабатывает соответствующее событие приложения.
    public void onResume() {
        super.onResume();
        updateCartButton();
    }
}
