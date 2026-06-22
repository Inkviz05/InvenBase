package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;

import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;
import java.util.Locale;
import java.util.Map;

public class NotificationsAdapter extends RecyclerView.Adapter<NotificationsAdapter.NotificationViewHolder> {

    public interface OnNotificationActionListener {
        // Метод onMarkAsRead: обрабатывает соответствующее событие приложения.
        void onMarkAsRead(String id);
    }

    private final OnNotificationActionListener listener;
    private List<Map<String, Object>> items = new ArrayList<>();

    // Конструктор NotificationsAdapter: инициализирует объект и его зависимости.
    public NotificationsAdapter(OnNotificationActionListener listener) {
        this.listener = listener;
    }

    // Метод setItems: устанавливает или обновляет значение данных.
    public void setItems(List<Map<String, Object>> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    @NonNull
    @Override
    // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
    public NotificationViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_notification, parent, false);
        return new NotificationViewHolder(view);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull NotificationViewHolder holder, int position) {
        holder.bind(items.get(position));
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return items.size();
    }

    class NotificationViewHolder extends RecyclerView.ViewHolder {
        private final TextView textTitle;
        private final TextView textMessage;
        private final TextView textDate;
        private final View unreadIndicator;
        private final Button buttonMarkRead;

        NotificationViewHolder(@NonNull View itemView) {
            super(itemView);
            textTitle = itemView.findViewById(R.id.text_title);
            textMessage = itemView.findViewById(R.id.text_message);
            textDate = itemView.findViewById(R.id.text_date);
            unreadIndicator = itemView.findViewById(R.id.view_unread);
            buttonMarkRead = itemView.findViewById(R.id.button_mark_read);
        }

        // Метод bind: выполняет основную бизнес- или UI-логику данного участка кода.
        void bind(Map<String, Object> data) {
            String id = String.valueOf(data.get("id"));
            String title = safeString(data.get("title"));
            String message = safeString(data.get("message"));
            String createdAt = safeString(data.get("created_at"));
            boolean isRead = Boolean.TRUE.equals(data.get("is_read"));

            textTitle.setText(title);
            textMessage.setText(stripUuidFromMessage(message));
            textDate.setText(formatDate(createdAt));

            unreadIndicator.setVisibility(isRead ? View.GONE : View.VISIBLE);
            buttonMarkRead.setVisibility(isRead ? View.GONE : View.VISIBLE);

            buttonMarkRead.setOnClickListener(v -> {
                if (listener != null) {
                    listener.onMarkAsRead(id);
                }
            });
        }

        // Метод safeString: выполняет основную бизнес- или UI-логику данного участка кода.
        private String safeString(Object value) {
            return value == null ? "" : String.valueOf(value);
        }

        /** Убирает UUID из текста уведомления (чтобы не показывать id оборудования и т.п.) */
        private String stripUuidFromMessage(String message) {
            if (message == null || message.isEmpty()) return message;
            String s = message.replaceAll("[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}", "");
            s = s.replaceAll("\\n\\n+", "\n").trim();
            return s.isEmpty() ? "—" : s;
        }

        // Метод formatDate: выполняет основную бизнес- или UI-логику данного участка кода.
        private String formatDate(String iso) {
            if (iso == null || iso.isEmpty()) return "";
            try {
                SimpleDateFormat isoFmt = new SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss", Locale.US);
                Date date = isoFmt.parse(iso);
                if (date == null) return iso;
                SimpleDateFormat out = new SimpleDateFormat("dd.MM.yyyy HH:mm", Locale.getDefault());
                return out.format(date);
            } catch (ParseException e) {
                return iso;
            }
        }
    }
}

