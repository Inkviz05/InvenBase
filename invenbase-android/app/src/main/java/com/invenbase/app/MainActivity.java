package com.invenbase.app;

import android.content.Intent;
import android.os.Build;
import android.os.Bundle;
import android.view.Menu;
import android.view.MenuItem;
import android.view.View;
import android.content.pm.PackageManager;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;

import androidx.annotation.NonNull;
import androidx.core.view.GravityCompat;
import androidx.drawerlayout.widget.DrawerLayout;
import androidx.fragment.app.Fragment;

import com.google.android.material.bottomnavigation.BottomNavigationView;
import com.google.android.material.navigation.NavigationBarView;
import com.google.android.material.navigation.NavigationView;
import com.invenbase.app.fragments.AdminFragment;
import com.invenbase.app.fragments.BookingsFragment;
import com.invenbase.app.fragments.DashboardFragment;
import com.invenbase.app.fragments.EquipmentFragment;
import com.invenbase.app.fragments.ScannerFragment;
import com.invenbase.app.utils.AuthManager;

public class MainActivity extends BaseActivity implements NavigationView.OnNavigationItemSelectedListener {
    private BottomNavigationView bottomNavigation;
    private AuthManager authManager;
    private DrawerLayout drawerLayout;
    private NavigationView navigationView;
    private static final int PERMISSIONS_REQUEST_CODE = 1001;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);

        authManager = AuthManager.getInstance(this);

        // Проверяем авторизацию
        if (!authManager.isAuthenticated()) {
            startActivity(new Intent(this, LoginActivity.class));
            finish();
            return;
        }

        drawerLayout = findViewById(R.id.drawer_layout);
        navigationView = findViewById(R.id.navigation_view);
        bottomNavigation = findViewById(R.id.bottom_navigation);

        if (drawerLayout == null || navigationView == null || bottomNavigation == null) {
            finish();
            return;
        }

        navigationView.setNavigationItemSelectedListener(this);
        setupDrawerHeader();
        setupDrawerVisibility();
        setupBottomNavVisibility();

        bottomNavigation.setOnItemSelectedListener(navListener);

        // Загружаем Dashboard по умолчанию
        if (savedInstanceState == null) {
            loadFragment(new DashboardFragment());
        }

        // Запрашиваем разрешения при первом запуске
        requestInitialPermissionsIfNeeded();
    }

    private void requestInitialPermissionsIfNeeded() {
        // Проверяем, запрашивали ли уже
        android.content.SharedPreferences prefs =
                getSharedPreferences("invenbase_prefs", MODE_PRIVATE);
        boolean asked = prefs.getBoolean("initial_permissions_asked", false);
        if (asked) {
            return;
        }

        java.util.List<String> permissionsToRequest = new java.util.ArrayList<>();

        // Уведомления — только Android 13+
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            if (ContextCompat.checkSelfPermission(this, android.Manifest.permission.POST_NOTIFICATIONS)
                    != PackageManager.PERMISSION_GRANTED) {
                permissionsToRequest.add(android.Manifest.permission.POST_NOTIFICATIONS);
            }
        }

        // Камера
        if (ContextCompat.checkSelfPermission(this, android.Manifest.permission.CAMERA)
                != PackageManager.PERMISSION_GRANTED) {
            permissionsToRequest.add(android.Manifest.permission.CAMERA);
        }

        if (!permissionsToRequest.isEmpty()) {
            ActivityCompat.requestPermissions(
                    this,
                    permissionsToRequest.toArray(new String[0]),
                    PERMISSIONS_REQUEST_CODE
            );
        }

        prefs.edit().putBoolean("initial_permissions_asked", true).apply();
    }

    private NavigationBarView.OnItemSelectedListener navListener =
        item -> {
            Fragment selectedFragment = null;
            int itemId = item.getItemId();

            if (itemId == R.id.nav_dashboard) {
                selectedFragment = new DashboardFragment();
            } else if (itemId == R.id.nav_equipment) {
                selectedFragment = new EquipmentFragment();
            } else if (itemId == R.id.nav_bookings) {
                selectedFragment = new BookingsFragment();
            } else if (itemId == R.id.nav_scanner) {
                selectedFragment = new ScannerFragment();
            } else if (itemId == R.id.nav_admin) {
                selectedFragment = new AdminFragment();
            }

            if (selectedFragment != null) {
                loadFragment(selectedFragment);
                return true;
            }
            return false;
        };

    private void loadFragment(Fragment fragment) {
        getSupportFragmentManager()
            .beginTransaction()
            .replace(R.id.fragment_container, fragment)
            .commit();
    }

    private void setupDrawerHeader() {
        try {
            View header = navigationView.getHeaderView(0);
            if (header == null) return;
            android.widget.TextView textUser = header.findViewById(R.id.text_header_user);
            android.widget.TextView textRole = header.findViewById(R.id.text_header_role);
            com.invenbase.app.models.User user = authManager.getUser();
            if (user != null) {
                if (textUser != null) {
                    textUser.setText(user.getFullName() != null && !user.getFullName().isEmpty()
                            ? user.getFullName()
                            : user.getUsername());
                }
                if (textRole != null) {
                    String role = user.isAdmin() ? "Администратор" : user.isResponsible() ? "Ответственный" : "Пользователь";
                    textRole.setText(role);
                }
            }
        } catch (Throwable ignored) { }
    }

    private void setupDrawerVisibility() {
        Menu menu = navigationView.getMenu();
        boolean isAdmin = authManager.isAdmin();
        boolean isResponsible = authManager.isResponsible();

        // Админские разделы
        menu.findItem(R.id.drawer_admin_panel).setVisible(isAdmin || isResponsible);
        menu.findItem(R.id.drawer_categories).setVisible(isAdmin || isResponsible);
        menu.findItem(R.id.drawer_reports).setVisible(isAdmin || isResponsible);
        menu.findItem(R.id.drawer_logs).setVisible(isAdmin || isResponsible);
        menu.findItem(R.id.drawer_users).setVisible(isAdmin);
    }

    private void setupBottomNavVisibility() {
        boolean isAdmin = authManager.isAdmin();
        boolean isResponsible = authManager.isResponsible();
        android.view.Menu menu = bottomNavigation.getMenu();
        android.view.MenuItem adminItem = menu.findItem(R.id.nav_admin);
        if (adminItem != null) {
            adminItem.setVisible(isAdmin || isResponsible);
        }
    }

    @Override
    public boolean onCreateOptionsMenu(Menu menu) {
        getMenuInflater().inflate(R.menu.main_menu, menu);
        return true;
    }

    @Override
    public boolean onOptionsItemSelected(@NonNull MenuItem item) {
        int itemId = item.getItemId();
        
        if (itemId == android.R.id.home) {
            if (drawerLayout != null) {
                drawerLayout.openDrawer(GravityCompat.START);
                return true;
            }
        } else if (itemId == R.id.menu_notifications) {
            startActivity(new Intent(this, NotificationsActivity.class));
            return true;
        } else if (itemId == R.id.menu_cart) {
            startActivity(new Intent(this, CartActivity.class));
            return true;
        } else if (itemId == R.id.menu_categories) {
            startActivity(new Intent(this, CategoriesActivity.class));
            return true;
        } else if (itemId == R.id.menu_users) {
            startActivity(new Intent(this, UsersActivity.class));
            return true;
        } else if (itemId == R.id.menu_reports) {
            startActivity(new Intent(this, ReportsActivity.class));
            return true;
        } else if (itemId == R.id.menu_logs) {
            startActivity(new Intent(this, LogsActivity.class));
            return true;
        } else if (itemId == R.id.menu_logout) {
            authManager.logout();
            startActivity(new Intent(this, LoginActivity.class));
            finish();
            return true;
        }
        return super.onOptionsItemSelected(item);
    }

    @Override
    public boolean onNavigationItemSelected(@NonNull MenuItem item) {
        int itemId = item.getItemId();

        if (itemId == R.id.drawer_dashboard) {
            bottomNavigation.setSelectedItemId(R.id.nav_dashboard);
        } else if (itemId == R.id.drawer_equipment) {
            bottomNavigation.setSelectedItemId(R.id.nav_equipment);
        } else if (itemId == R.id.drawer_bookings) {
            bottomNavigation.setSelectedItemId(R.id.nav_bookings);
        } else if (itemId == R.id.drawer_scanner) {
            bottomNavigation.setSelectedItemId(R.id.nav_scanner);
        } else if (itemId == R.id.drawer_admin_panel) {
            bottomNavigation.setSelectedItemId(R.id.nav_admin);
        } else if (itemId == R.id.drawer_categories) {
            startActivity(new Intent(this, CategoriesActivity.class));
        } else if (itemId == R.id.drawer_reports) {
            startActivity(new Intent(this, ReportsActivity.class));
        } else if (itemId == R.id.drawer_logs) {
            startActivity(new Intent(this, LogsActivity.class));
        } else if (itemId == R.id.drawer_users) {
            startActivity(new Intent(this, UsersActivity.class));
        } else if (itemId == R.id.drawer_notifications) {
            startActivity(new Intent(this, NotificationsActivity.class));
        } else if (itemId == R.id.drawer_cart) {
            startActivity(new Intent(this, CartActivity.class));
        } else if (itemId == R.id.drawer_logout) {
            authManager.logout();
            startActivity(new Intent(this, LoginActivity.class));
            finish();
        }

        if (drawerLayout != null) {
            drawerLayout.closeDrawer(GravityCompat.START);
        }
        return true;
    }
}
