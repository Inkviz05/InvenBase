package com.invenbase.app.adapters;

import android.graphics.drawable.GradientDrawable;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.LinearLayout;
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

public class StatsAdapter extends RecyclerView.Adapter<StatsAdapter.StatViewHolder> {
    private List<StatItem> items = new ArrayList<>();
    
    private static final Map<String, IconConfig> ICON_CONFIGS = new HashMap<>();
    
    static {
        ICON_CONFIGS.put("Всего", new IconConfig("📦", 0xFFA855F7, 0xFFEC4899));
        ICON_CONFIGS.put("Доступно", new IconConfig("✅", 0xFF10B981, 0xFF059669));
        ICON_CONFIGS.put("Забронировано", new IconConfig("📋", 0xFF22D3EE, 0xFF06B6D4));
        ICON_CONFIGS.put("Ожидает одобрения", new IconConfig("⏳", 0xFFF59E0B, 0xFFD97706));
        ICON_CONFIGS.put("Ожидают одобрения", new IconConfig("⏳", 0xFFF59E0B, 0xFFD97706));
        ICON_CONFIGS.put("Одобрено", new IconConfig("✓", 0xFF10B981, 0xFF059669));
        ICON_CONFIGS.put("Одобрены", new IconConfig("✓", 0xFF10B981, 0xFF059669));
        ICON_CONFIGS.put("Бронирования", new IconConfig("📅", 0xFF22D3EE, 0xFF06B6D4));
        ICON_CONFIGS.put("Уведомления", new IconConfig("🔔", 0xFFA855F7, 0xFFEC4899));
        // Отчёты (как в веб-версии)
        ICON_CONFIGS.put("Всего позиций", new IconConfig("📦", 0xFFA855F7, 0xFFEC4899));
        ICON_CONFIGS.put("Доступно сейчас", new IconConfig("✅", 0xFF10B981, 0xFF059669));
        ICON_CONFIGS.put("В бронировании", new IconConfig("📋", 0xFFF59E0B, 0xFFD97706));
        ICON_CONFIGS.put("Всего бронирований", new IconConfig("📊", 0xFFA855F7, 0xFFEC4899));
        ICON_CONFIGS.put("Отклонены", new IconConfig("✕", 0xFFEF4444, 0xFFDC2626));
        ICON_CONFIGS.put("Истекли", new IconConfig("⏱", 0xFF6B7280, 0xFF4B5563));
        ICON_CONFIGS.put("Отменены", new IconConfig("↩", 0xFFF59E0B, 0xFFD97706));
        ICON_CONFIGS.put("Завершены", new IconConfig("✔", 0xFF10B981, 0xFF059669));
    }
    
    private static class IconConfig {
        String emoji;
        int startColor;
        int endColor;
        
        IconConfig(String emoji, int startColor, int endColor) {
            this.emoji = emoji;
            this.startColor = startColor;
            this.endColor = endColor;
        }
    }

    // Метод setItems: устанавливает или обновляет значение данных.
    public void setItems(List<StatItem> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    @NonNull
    @Override
    // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
    public StatViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
            .inflate(R.layout.item_stat, parent, false);
        return new StatViewHolder(view);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull StatViewHolder holder, int position) {
        holder.bind(items.get(position));
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return items.size();
    }

    static class StatViewHolder extends RecyclerView.ViewHolder {
        private final TextView textTitle;
        private final TextView textValue;
        private final TextView iconStat;
        private final LinearLayout iconContainer;

        StatViewHolder(@NonNull View itemView) {
            super(itemView);
            textTitle = itemView.findViewById(R.id.text_stat_title);
            textValue = itemView.findViewById(R.id.text_stat_value);
            iconStat = itemView.findViewById(R.id.icon_stat);
            iconContainer = itemView.findViewById(R.id.icon_container);
        }

        // Метод bind: выполняет основную бизнес- или UI-логику данного участка кода.
        void bind(StatItem item) {
            textTitle.setText(item.getTitle());
            textValue.setText(item.getValue());
            
            IconConfig config = ICON_CONFIGS.get(item.getTitle());
            if (config != null) {
                iconStat.setText(config.emoji);
                GradientDrawable gradient = new GradientDrawable(
                    GradientDrawable.Orientation.TL_BR,
                    new int[]{config.startColor, config.endColor}
                );
                gradient.setCornerRadius(16f * itemView.getContext().getResources().getDisplayMetrics().density);
                iconContainer.setBackground(gradient);
            } else {
                // Default gradient
                iconStat.setText("📊");
                GradientDrawable gradient = new GradientDrawable(
                    GradientDrawable.Orientation.TL_BR,
                    new int[]{0xFFA855F7, 0xFFEC4899}
                );
                gradient.setCornerRadius(16f * itemView.getContext().getResources().getDisplayMetrics().density);
                iconContainer.setBackground(gradient);
            }
        }
    }
}
