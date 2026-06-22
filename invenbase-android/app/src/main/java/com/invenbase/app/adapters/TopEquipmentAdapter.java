package com.invenbase.app.adapters;

import android.content.Context;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.TopEquipmentRow;

import java.util.ArrayList;
import java.util.List;

public class TopEquipmentAdapter extends RecyclerView.Adapter<RecyclerView.ViewHolder> {
    private static final int TYPE_HEADER = 0;
    private static final int TYPE_ROW = 1;

    private final Context context;
    private List<TopEquipmentRow> items = new ArrayList<>();

    // Конструктор TopEquipmentAdapter: инициализирует объект и его зависимости.
    public TopEquipmentAdapter(Context context) {
        this.context = context;
    }

    // Метод setItems: устанавливает или обновляет значение данных.
    public void setItems(List<TopEquipmentRow> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    @Override
    // Метод getItemViewType: возвращает нужное значение для текущего контекста.
    public int getItemViewType(int position) {
        return position == 0 ? TYPE_HEADER : TYPE_ROW;
    }

    @NonNull
    @Override
    // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
    public RecyclerView.ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        if (viewType == TYPE_HEADER) {
            View v = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_top_equipment_header, parent, false);
            return new HeaderVH(v);
        }
        View v = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_top_equipment_row, parent, false);
        return new VH(v);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull RecyclerView.ViewHolder holder, int position) {
        if (holder instanceof HeaderVH) {
            HeaderVH h = (HeaderVH) holder;
            h.textNum.setText(context.getString(R.string.report_col_num));
            h.textEquipment.setText(context.getString(R.string.report_col_equipment));
            h.textCategory.setText(context.getString(R.string.category));
            h.textCount.setText(context.getString(R.string.report_col_bookings));
            h.textPercent.setText(context.getString(R.string.report_percent));
            h.textQty.setText(context.getString(R.string.report_total_qty));
        } else {
            TopEquipmentRow row = items.get(position - 1);
            VH vh = (VH) holder;
            vh.textNum.setText(String.valueOf(row.index + 1));
            vh.textEquipment.setText(row.equipmentName);
            vh.textCategory.setText(row.categoryName);
            vh.textCount.setText(String.valueOf(row.bookingsCount));
            vh.textPercent.setText(row.percent + "%");
            vh.textQty.setText(String.valueOf(row.totalQuantity));
        }
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return 1 + items.size();
    }

    static class HeaderVH extends RecyclerView.ViewHolder {
        final TextView textNum, textEquipment, textCategory, textCount, textPercent, textQty;

        HeaderVH(@NonNull View itemView) {
            super(itemView);
            textNum = itemView.findViewById(R.id.text_top_num);
            textEquipment = itemView.findViewById(R.id.text_top_equipment);
            textCategory = itemView.findViewById(R.id.text_top_category);
            textCount = itemView.findViewById(R.id.text_top_count);
            textPercent = itemView.findViewById(R.id.text_top_percent);
            textQty = itemView.findViewById(R.id.text_top_qty);
        }
    }

    static class VH extends RecyclerView.ViewHolder {
        final TextView textNum, textEquipment, textCategory, textCount, textPercent, textQty;

        VH(@NonNull View itemView) {
            super(itemView);
            textNum = itemView.findViewById(R.id.text_top_num);
            textEquipment = itemView.findViewById(R.id.text_top_equipment);
            textCategory = itemView.findViewById(R.id.text_top_category);
            textCount = itemView.findViewById(R.id.text_top_count);
            textPercent = itemView.findViewById(R.id.text_top_percent);
            textQty = itemView.findViewById(R.id.text_top_qty);
        }
    }
}
