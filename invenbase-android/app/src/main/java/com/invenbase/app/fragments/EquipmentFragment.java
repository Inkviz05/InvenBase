package com.invenbase.app.fragments;

import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ProgressBar;
import android.widget.Toast;
import android.widget.Button;
import android.widget.LinearLayout;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.fragment.app.Fragment;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import android.content.Intent;

import com.google.android.material.bottomnavigation.BottomNavigationView;
import com.invenbase.app.EquipmentDetailActivity;
import com.invenbase.app.EquipmentFormActivity;
import com.invenbase.app.R;
import com.invenbase.app.adapters.EquipmentAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Equipment;
import com.invenbase.app.utils.AuthManager;

import java.util.List;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class EquipmentFragment extends Fragment {
    private RecyclerView recyclerEquipment;
    private ProgressBar progressBar;
    private EquipmentAdapter adapter;
    private ApiService apiService;
    private AuthManager authManager;
    private com.google.android.material.card.MaterialCardView regularUserContainer;
    private Button buttonOpenScanner;
    private Button buttonAddEquipment;
    private boolean isRegularUser = false;

    @Nullable
    @Override
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container, @Nullable Bundle savedInstanceState) {
        View view = inflater.inflate(R.layout.fragment_equipment, container, false);
        
        recyclerEquipment = view.findViewById(R.id.recycler_equipment);
        progressBar = view.findViewById(R.id.progress_bar);
        regularUserContainer = view.findViewById(R.id.regular_user_container);
        buttonOpenScanner = view.findViewById(R.id.button_open_scanner);
        buttonAddEquipment = view.findViewById(R.id.button_add_equipment);
        
        recyclerEquipment.setLayoutManager(new LinearLayoutManager(requireContext()));
        adapter = new EquipmentAdapter();
        recyclerEquipment.setAdapter(adapter);
        
        apiService = ApiClient.getInstance(requireContext()).getApiService();
        authManager = AuthManager.getInstance(requireContext());

        buttonOpenScanner.setOnClickListener(v -> {
            if (getActivity() != null) {
                BottomNavigationView bottomNavigation = getActivity().findViewById(R.id.bottom_navigation);
                if (bottomNavigation != null) {
                    bottomNavigation.setSelectedItemId(R.id.nav_scanner);
                }
            }
        });

        adapter.setOnEquipmentClickListener(equipment -> {
            EquipmentDetailActivity.open(requireContext(), equipment.getId());
        });

        buttonAddEquipment.setOnClickListener(v -> {
            Intent intent = new Intent(requireContext(), EquipmentFormActivity.class);
            intent.putExtra(EquipmentFormActivity.EXTRA_IS_EDIT, false);
            startActivity(intent);
        });

        // Разграничение прав как в веб-версии
        boolean isAdmin = authManager.isAdmin();
        boolean isResponsible = authManager.isResponsible();
        isRegularUser = !isAdmin && !isResponsible;
        
        View headerCard = view.findViewById(R.id.header_card);
        if (headerCard != null) {
            headerCard.setVisibility(isAdmin || isResponsible ? View.VISIBLE : View.GONE);
        }
        buttonAddEquipment.setVisibility(isAdmin || isResponsible ? View.VISIBLE : View.GONE);
        
        // Для обычных пользователей показываем только контейнер с кнопкой открыть сканер
        // Для админов и ответственных - показываем список оборудования
        if (isRegularUser) {
            regularUserContainer.setVisibility(View.VISIBLE);
            recyclerEquipment.setVisibility(View.GONE);
        } else {
            regularUserContainer.setVisibility(View.GONE);
            recyclerEquipment.setVisibility(View.VISIBLE);
            loadEquipment();
        }
        
        return view;
    }

    @Override
    public void onResume() {
        super.onResume();
        // Обновляем список только для админов и ответственных
        if (authManager != null && recyclerEquipment.getVisibility() == View.VISIBLE) {
            loadEquipment();
        }
    }

    private void loadEquipment() {
        progressBar.setVisibility(View.VISIBLE);
        
        Call<List<Equipment>> call = apiService.getEquipment();
        call.enqueue(new Callback<List<Equipment>>() {
            @Override
            public void onResponse(Call<List<Equipment>> call, Response<List<Equipment>> response) {
                progressBar.setVisibility(View.GONE);
                
                if (response.isSuccessful() && response.body() != null) {
                    adapter.setEquipmentList(response.body());
                } else {
                    Toast.makeText(requireContext(), R.string.error, Toast.LENGTH_SHORT).show();
                }
            }

            @Override
            public void onFailure(Call<List<Equipment>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(requireContext(), R.string.error + ": " + t.getMessage(), Toast.LENGTH_SHORT).show();
            }
        });
    }

}
