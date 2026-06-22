package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.core.content.ContextCompat;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.StatItem;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public class ReportStatAdapter extends RecyclerView.Adapter<ReportStatAdapter.VH> {
    private List<StatItem> items = new ArrayList<>();

    private static final Map<String, Integer> VALUE_COLORS = new HashMap<>();
    static {
        VALUE_COLORS.put("Доступно сейчас", R.color.success);
        VALUE_COLORS.put("В бронировании", R.color.warning);
        VALUE_COLORS.put("Одобрены", R.color.success);
        VALUE_COLORS.put("Отклонены", R.color.error);
        VALUE_COLORS.put("Истекли", R.color.text_secondary);
        VALUE_COLORS.put("Отменены", R.color.warning);
        VALUE_COLORS.put("Завершены", R.color.success);
    }

    // Метод setItems: устанавливает или обновляет значение данных.
    public void setItems(List<StatItem> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    @NonNull
    @Override
    // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
    public VH onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View v = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_stat_report, parent, false);
        return new VH(v);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull VH holder, int position) {
        StatItem item = items.get(position);
        holder.textTitle.setText(item.getTitle());
        holder.textValue.setText(item.getValue());
        Integer colorRes = VALUE_COLORS.get(item.getTitle());
        int color = colorRes != null ? ContextCompat.getColor(holder.itemView.getContext(), colorRes)
                : ContextCompat.getColor(holder.itemView.getContext(), R.color.text_primary);
        holder.textValue.setTextColor(color);
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return items.size();
    }

    static class VH extends RecyclerView.ViewHolder {
        final TextView textTitle, textValue;

        VH(@NonNull View itemView) {
            super(itemView);
            textTitle = itemView.findViewById(R.id.text_stat_title);
            textValue = itemView.findViewById(R.id.text_stat_value);
        }
    }
}
