package com.invenbase.app.fragments;

import android.content.Intent;
import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.fragment.app.Fragment;

import com.google.android.material.card.MaterialCardView;
import com.invenbase.app.CategoriesActivity;
import com.invenbase.app.EquipmentFormActivity;
import com.invenbase.app.LogsActivity;
import com.invenbase.app.ReportsActivity;
import com.invenbase.app.R;
import com.invenbase.app.SquadsActivity;
import com.invenbase.app.SupportActivity;
import com.invenbase.app.UsersActivity;
import com.invenbase.app.utils.AuthManager;

public class AdminFragment extends Fragment {

    private AuthManager authManager;

    @Override
    // Метод onCreate: обрабатывает соответствующее событие приложения.
    public void onCreate(@Nullable Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        authManager = AuthManager.getInstance(requireContext());
    }

    @Nullable
    @Override
    // Метод onCreateView: обрабатывает соответствующее событие приложения.
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container, @Nullable Bundle savedInstanceState) {
        return inflater.inflate(R.layout.fragment_admin, container, false);
    }

    @Override
    // Метод onViewCreated: обрабатывает соответствующее событие приложения.
    public void onViewCreated(@NonNull View view, @Nullable Bundle savedInstanceState) {
        super.onViewCreated(view, savedInstanceState);

        MaterialCardView cardAddEquipment = view.findViewById(R.id.card_add_equipment);
        MaterialCardView cardReports = view.findViewById(R.id.card_reports);
        MaterialCardView cardSupport = view.findViewById(R.id.card_support);
        MaterialCardView cardCategories = view.findViewById(R.id.card_categories);
        MaterialCardView cardSquads = view.findViewById(R.id.card_squads);
        MaterialCardView cardUsers = view.findViewById(R.id.card_users);
        MaterialCardView cardLogs = view.findViewById(R.id.card_logs);

        // «Добавить оборудование» — для администратора и ответственного (карточка всегда видна на вкладке Управление)
        cardAddEquipment.setOnClickListener(v -> startActivity(new Intent(requireContext(), EquipmentFormActivity.class)));

        // Пользователи — только для администратора
        if (!authManager.isAdmin()) {
            cardUsers.setVisibility(View.GONE);
        }

        cardReports.setOnClickListener(v -> startActivity(new Intent(requireContext(), ReportsActivity.class)));
        cardSupport.setOnClickListener(v -> startActivity(new Intent(requireContext(), SupportActivity.class)));
        cardCategories.setOnClickListener(v -> startActivity(new Intent(requireContext(), CategoriesActivity.class)));
        cardSquads.setOnClickListener(v -> startActivity(new Intent(requireContext(), SquadsActivity.class)));
        cardUsers.setOnClickListener(v -> startActivity(new Intent(requireContext(), UsersActivity.class)));
        cardLogs.setOnClickListener(v -> startActivity(new Intent(requireContext(), LogsActivity.class)));
    }
}
