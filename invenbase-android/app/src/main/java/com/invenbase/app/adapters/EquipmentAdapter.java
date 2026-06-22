package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.Equipment;

import java.util.ArrayList;
import java.util.List;

public class EquipmentAdapter extends RecyclerView.Adapter<EquipmentAdapter.EquipmentViewHolder> {
    private List<Equipment> equipmentList;
    private OnEquipmentClickListener listener;

    public interface OnEquipmentClickListener {
        // Метод onEquipmentClick: обрабатывает соответствующее событие приложения.
        void onEquipmentClick(Equipment equipment);
    }

    // Конструктор EquipmentAdapter: инициализирует объект и его зависимости.
    public EquipmentAdapter() {
        this.equipmentList = new ArrayList<>();
    }

    // Метод setEquipmentList: устанавливает или обновляет значение данных.
    public void setEquipmentList(List<Equipment> equipmentList) {
        this.equipmentList = equipmentList;
        notifyDataSetChanged();
    }

    // Метод setOnEquipmentClickListener: устанавливает или обновляет значение данных.
    public void setOnEquipmentClickListener(OnEquipmentClickListener listener) {
        this.listener = listener;
    }

    @NonNull
    @Override
    // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
    public EquipmentViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
            .inflate(R.layout.item_equipment, parent, false);
        return new EquipmentViewHolder(view);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull EquipmentViewHolder holder, int position) {
        Equipment equipment = equipmentList.get(position);
        holder.bind(equipment);
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return equipmentList.size();
    }

    class EquipmentViewHolder extends RecyclerView.ViewHolder {
        private TextView textName;
        private TextView textDescription;
        private TextView textAvailable;
        private TextView textTotal;
        private TextView textQrIndicator;

        // Конструктор EquipmentViewHolder: инициализирует объект и его зависимости.
        public EquipmentViewHolder(@NonNull View itemView) {
            super(itemView);
            textName = itemView.findViewById(R.id.text_name);
            textDescription = itemView.findViewById(R.id.text_description);
            textAvailable = itemView.findViewById(R.id.text_available);
            textTotal = itemView.findViewById(R.id.text_total);
            textQrIndicator = itemView.findViewById(R.id.text_qr_indicator);

            itemView.setOnClickListener(v -> {
                if (listener != null) {
                    listener.onEquipmentClick(equipmentList.get(getAdapterPosition()));
                }
            });
        }

        // Метод bind: выполняет основную бизнес- или UI-логику данного участка кода.
        public void bind(Equipment equipment) {
            textName.setText(equipment.getName());
            textDescription.setText(equipment.getDescription() != null ? 
                equipment.getDescription() : "Нет описания");
            textAvailable.setText("Доступно: " + equipment.getAvailableQuantity());
            textTotal.setText("Всего: " + equipment.getQuantity());
            
            // Показываем индикатор QR-кода, если он есть
            if (equipment.getQrCode() != null && !equipment.getQrCode().isEmpty()) {
                textQrIndicator.setVisibility(View.VISIBLE);
            } else {
                textQrIndicator.setVisibility(View.GONE);
            }
        }
    }
}
