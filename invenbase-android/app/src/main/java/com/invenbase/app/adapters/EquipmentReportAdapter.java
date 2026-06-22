package com.invenbase.app.adapters;

import android.content.Context;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.EquipmentReportRow;

import java.util.ArrayList;
import java.util.List;

public class EquipmentReportAdapter extends RecyclerView.Adapter<RecyclerView.ViewHolder> {
    private static final int TYPE_HEADER = 0;
    private static final int TYPE_ROW = 1;

    private final Context context;
    private List<EquipmentReportRow> items = new ArrayList<>();

    // Конструктор EquipmentReportAdapter: инициализирует объект и его зависимости.
    public EquipmentReportAdapter(Context context) {
        this.context = context;
    }

    // Метод setItems: устанавливает или обновляет значение данных.
    public void setItems(List<EquipmentReportRow> items) {
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
            View v = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_equipment_report_header, parent, false);
            return new HeaderVH(v);
        }
        View v = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_equipment_report_row, parent, false);
        return new VH(v);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull RecyclerView.ViewHolder holder, int position) {
        if (holder instanceof HeaderVH) {
            HeaderVH h = (HeaderVH) holder;
            h.textNum.setText(context.getString(R.string.report_col_num));
            h.textName.setText(context.getString(R.string.report_col_name));
            h.textCategory.setText(context.getString(R.string.category));
            h.textTotal.setText(context.getString(R.string.total));
            h.textAvailable.setText(context.getString(R.string.available));
            h.textBooked.setText(context.getString(R.string.report_in_booking));
            h.textStatus.setText(context.getString(R.string.status));
        } else {
            EquipmentReportRow row = items.get(position - 1);
            VH vh = (VH) holder;
            vh.textNum.setText(String.valueOf(row.index + 1));
            vh.textName.setText(row.name);
            vh.textCategory.setText(row.category);
            vh.textTotal.setText(String.valueOf(row.total));
            vh.textAvailable.setText(String.valueOf(row.available));
            vh.textBooked.setText(String.valueOf(row.booked));
            vh.textStatus.setText(row.status);
        }
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return 1 + items.size();
    }

    static class HeaderVH extends RecyclerView.ViewHolder {
        final TextView textNum, textName, textCategory, textTotal, textAvailable, textBooked, textStatus;

        HeaderVH(@NonNull View itemView) {
            super(itemView);
            textNum = itemView.findViewById(R.id.text_row_num);
            textName = itemView.findViewById(R.id.text_row_name);
            textCategory = itemView.findViewById(R.id.text_row_category);
            textTotal = itemView.findViewById(R.id.text_row_total);
            textAvailable = itemView.findViewById(R.id.text_row_available);
            textBooked = itemView.findViewById(R.id.text_row_booked);
            textStatus = itemView.findViewById(R.id.text_row_status);
        }
    }

    static class VH extends RecyclerView.ViewHolder {
        final TextView textNum, textName, textCategory, textTotal, textAvailable, textBooked, textStatus;

        VH(View itemView) {
            super(itemView);
            textNum = itemView.findViewById(R.id.text_row_num);
            textName = itemView.findViewById(R.id.text_row_name);
            textCategory = itemView.findViewById(R.id.text_row_category);
            textTotal = itemView.findViewById(R.id.text_row_total);
            textAvailable = itemView.findViewById(R.id.text_row_available);
            textBooked = itemView.findViewById(R.id.text_row_booked);
            textStatus = itemView.findViewById(R.id.text_row_status);
        }
    }
}
