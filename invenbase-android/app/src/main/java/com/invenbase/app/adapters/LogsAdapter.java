package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.google.gson.Gson;
import com.invenbase.app.R;

import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;
import java.util.Locale;
import java.util.Map;

public class LogsAdapter extends RecyclerView.Adapter<LogsAdapter.LogViewHolder> {

    private List<Map<String, Object>> items = new ArrayList<>();

    // Метод setItems: устанавливает или обновляет значение данных.
    public void setItems(List<Map<String, Object>> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    @NonNull
    @Override
    // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
    public LogViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_log, parent, false);
        return new LogViewHolder(view);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull LogViewHolder holder, int position) {
        holder.bind(items.get(position));
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return items.size();
    }

    class LogViewHolder extends RecyclerView.ViewHolder {
        private final TextView textTime;
        private final TextView textAction;
        private final TextView textEntityType;
        private final TextView textDetails;

        LogViewHolder(@NonNull View itemView) {
            super(itemView);
            textTime = itemView.findViewById(R.id.text_time);
            textAction = itemView.findViewById(R.id.text_action);
            textEntityType = itemView.findViewById(R.id.text_entity_type);
            textDetails = itemView.findViewById(R.id.text_details);
        }

        // Метод bind: выполняет основную бизнес- или UI-логику данного участка кода.
        void bind(Map<String, Object> data) {
            String createdAt = safeString(data.get("created_at"));
            textTime.setText(formatDate(createdAt));
            textAction.setText(safeString(data.get("action")));
            textEntityType.setText(safeString(data.get("entity_type")));

            Object details = data.get("details");
            if (details != null) {
                try {
                    Gson gson = new Gson();
                    String json = gson.toJson(details);
                    textDetails.setText(json);
                } catch (Exception e) {
                    textDetails.setText(safeString(details));
                }
            } else {
                textDetails.setText("-");
            }
        }

        // Метод safeString: выполняет основную бизнес- или UI-логику данного участка кода.
        private String safeString(Object value) {
            return value == null ? "" : String.valueOf(value);
        }

        // Метод formatDate: выполняет основную бизнес- или UI-логику данного участка кода.
        private String formatDate(String iso) {
            if (iso == null || iso.isEmpty()) return "";
            try {
                SimpleDateFormat isoFmt = new SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss", Locale.US);
                Date date = isoFmt.parse(iso);
                if (date == null) return iso;
                SimpleDateFormat out = new SimpleDateFormat("dd.MM.yyyy HH:mm:ss", Locale.getDefault());
                return out.format(date);
            } catch (ParseException e) {
                return iso;
            }
        }
    }
}
