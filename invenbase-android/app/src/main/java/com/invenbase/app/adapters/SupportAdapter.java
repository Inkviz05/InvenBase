package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.LinearLayout;
import android.widget.EditText;
import android.widget.Spinner;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.core.content.ContextCompat;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.google.android.material.button.MaterialButton;

import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;
import java.util.Locale;
import java.util.Map;

public class SupportAdapter extends RecyclerView.Adapter<SupportAdapter.RequestViewHolder> {

    public interface SupportListener {
        // Метод onAddMessageClick: обрабатывает соответствующее событие приложения.
        void onAddMessageClick(String requestId);
        // Метод onSendReply: обрабатывает соответствующее событие приложения.
        void onSendReply(String requestId, String status, String comment);
        // Метод onDeleteRequest: обрабатывает соответствующее событие приложения.
        void onDeleteRequest(String requestId);
    }

    private final SupportListener listener;
    private final boolean isAdmin;
    private final String currentUserId;
    private List<Map<String, Object>> items = new ArrayList<>();
    private int expandedReplyPosition = -1;

    private static final String[] STATUS_VALUES = {"open", "in_progress", "answered", "closed"};
    private static final int[] STATUS_LABELS = {
            R.string.support_status_open,
            R.string.support_status_in_progress,
            R.string.support_status_answered,
            R.string.support_status_closed
    };

    // Конструктор SupportAdapter: инициализирует объект и его зависимости.
    public SupportAdapter(SupportListener listener, boolean isAdmin, String currentUserId) {
        this.listener = listener;
        this.isAdmin = isAdmin;
        this.currentUserId = currentUserId;
    }

    // Метод setItems: устанавливает или обновляет значение данных.
    public void setItems(List<Map<String, Object>> items) {
        this.items = items != null ? items : new ArrayList<>();
        expandedReplyPosition = -1;
        notifyDataSetChanged();
    }

    @NonNull
    @Override
    // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
    public RequestViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_support_request, parent, false);
        return new RequestViewHolder(view);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull RequestViewHolder holder, int position) {
        holder.bind(items.get(position), position);
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return items.size();
    }

    class RequestViewHolder extends RecyclerView.ViewHolder {
        private final TextView textSubject;
        private final TextView textStatus;
        private final TextView textUser;
        private final LinearLayout containerThread;
        private final TextView textDate;
        private final MaterialButton buttonAddMessage;
        private final LinearLayout adminReplyBlock;
        private final Spinner spinnerStatus;
        private final EditText editAdminReply;
        private final MaterialButton buttonSendReply;
        private final MaterialButton buttonCloseReply;
        private final MaterialButton buttonReply;
        private final MaterialButton buttonDeleteRequest;

        RequestViewHolder(@NonNull View itemView) {
            super(itemView);
            textSubject = itemView.findViewById(R.id.text_request_subject);
            textStatus = itemView.findViewById(R.id.text_request_status);
            textUser = itemView.findViewById(R.id.text_request_user);
            containerThread = itemView.findViewById(R.id.container_thread);
            textDate = itemView.findViewById(R.id.text_request_date);
            buttonAddMessage = itemView.findViewById(R.id.button_add_message);
            adminReplyBlock = itemView.findViewById(R.id.admin_reply_block);
            spinnerStatus = itemView.findViewById(R.id.spinner_status);
            editAdminReply = itemView.findViewById(R.id.edit_admin_reply);
            buttonSendReply = itemView.findViewById(R.id.button_send_reply);
            buttonCloseReply = itemView.findViewById(R.id.button_close_reply);
            buttonReply = itemView.findViewById(R.id.button_reply);
            buttonDeleteRequest = itemView.findViewById(R.id.button_delete_request);
        }

        // Метод bind: выполняет основную бизнес- или UI-логику данного участка кода.
        void bind(Map<String, Object> req, int position) {
            String id = safeStr(req.get("id"));
            String subject = safeStr(req.get("subject"));
            String status = safeStr(req.get("status"));
            String userId = safeStr(req.get("user_id"));
            String userName = safeStr(req.get("user_name"));
            String message = safeStr(req.get("message"));
            String createdAt = safeStr(req.get("created_at"));
            String updatedAt = safeStr(req.get("updated_at"));
            String adminComment = req.get("admin_comment") != null ? safeStr(req.get("admin_comment")) : null;

            textSubject.setText(subject);
            textStatus.setText(getStatusLabel(status));
            textDate.setText(formatDate(createdAt));

            if (isAdmin && userName != null && !userName.isEmpty()) {
                textUser.setVisibility(View.VISIBLE);
                textUser.setText(itemView.getContext().getString(R.string.support_from, userName));
            } else {
                textUser.setVisibility(View.GONE);
            }

            // Переписка: первое сообщение + messages или admin_comment
            containerThread.removeAllViews();
            List<Map<String, Object>> messages = getMessagesList(req);
            boolean hasMessages = messages != null && !messages.isEmpty();
            if (hasMessages) {
                addMessageBubble(containerThread, message, createdAt, false);
                for (Map<String, Object> m : messages) {
                    boolean isStaff = Boolean.TRUE.equals(m.get("is_staff"));
                    String msgText = safeStr(m.get("message"));
                    String msgDate = safeStr(m.get("created_at"));
                    addMessageBubble(containerThread, msgText, msgDate, isStaff);
                }
            } else {
                addMessageBubble(containerThread, message, createdAt, false);
                if (adminComment != null && !adminComment.isEmpty()) {
                    addMessageBubble(containerThread, adminComment, updatedAt, true);
                }
            }

            // Пользователь: кнопка "Добавить сообщение" если не закрыта
            if (!isAdmin && "closed".equals(status)) {
                buttonAddMessage.setVisibility(View.GONE);
            } else if (!isAdmin && userId.equals(currentUserId)) {
                buttonAddMessage.setVisibility(View.VISIBLE);
                buttonAddMessage.setOnClickListener(v -> listener.onAddMessageClick(id));
            } else {
                buttonAddMessage.setVisibility(View.GONE);
            }

            // Админ: для закрытой заявки — только «Удалить», иначе — «Ответить» и блок формы
            if (isAdmin) {
                if ("closed".equals(status)) {
                    buttonReply.setVisibility(View.GONE);
                    adminReplyBlock.setVisibility(View.GONE);
                    buttonDeleteRequest.setVisibility(View.VISIBLE);
                    buttonDeleteRequest.setOnClickListener(v -> listener.onDeleteRequest(id));
                } else {
                    buttonDeleteRequest.setVisibility(View.GONE);
                    buttonReply.setVisibility(View.VISIBLE);
                    boolean showForm = (expandedReplyPosition == position);
                    adminReplyBlock.setVisibility(showForm ? View.VISIBLE : View.GONE);
                    if (showForm) {
                        setupSpinner(status);
                        editAdminReply.setText("");
                        buttonSendReply.setOnClickListener(v -> {
                            String comment = editAdminReply.getText() != null ? editAdminReply.getText().toString().trim() : "";
                            String selStatus = STATUS_VALUES[spinnerStatus.getSelectedItemPosition()];
                            listener.onSendReply(id, selStatus, comment.isEmpty() ? null : comment);
                            expandedReplyPosition = -1;
                            editAdminReply.setText("");
                        });
                        buttonCloseReply.setOnClickListener(v -> {
                            expandedReplyPosition = -1;
                            notifyItemChanged(position);
                        });
                    }
                    buttonReply.setOnClickListener(v -> {
                        if (expandedReplyPosition == position) {
                            expandedReplyPosition = -1;
                        } else {
                            expandedReplyPosition = position;
                        }
                        notifyDataSetChanged();
                    });
                }
            } else {
                buttonReply.setVisibility(View.GONE);
                buttonDeleteRequest.setVisibility(View.GONE);
                adminReplyBlock.setVisibility(View.GONE);
            }
        }

        // Метод addMessageBubble: выполняет основную бизнес- или UI-логику данного участка кода.
        private void addMessageBubble(LinearLayout parent, String text, String date, boolean isStaff) {
            View bubble = LayoutInflater.from(itemView.getContext())
                    .inflate(android.R.layout.simple_list_item_2, parent, false);
            TextView t1 = bubble.findViewById(android.R.id.text1);
            TextView t2 = bubble.findViewById(android.R.id.text2);
            t1.setText(text);
            t2.setText(formatDate(date));
            t1.setTextSize(14);
            t2.setTextSize(12);
            float density = itemView.getContext().getResources().getDisplayMetrics().density;
            int pad = (int) (14 * density);
            int marginTop = (int) (10 * density);
            bubble.setPadding(pad, pad, pad, pad);
            LinearLayout.LayoutParams lp = new LinearLayout.LayoutParams(
                    ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT);
            if (parent.getChildCount() > 0) {
                lp.topMargin = marginTop;
            }
            bubble.setLayoutParams(lp);
            if (isStaff) {
                bubble.setBackgroundColor(ContextCompat.getColor(itemView.getContext(), R.color.primary));
                t1.setTextColor(ContextCompat.getColor(itemView.getContext(), android.R.color.white));
                t2.setTextColor(ContextCompat.getColor(itemView.getContext(), android.R.color.white));
            } else {
                bubble.setBackgroundColor(ContextCompat.getColor(itemView.getContext(), R.color.background));
            }
            parent.addView(bubble);
        }

        // Метод setupSpinner: устанавливает или обновляет значение данных.
        private void setupSpinner(String currentStatus) {
            if (spinnerStatus.getAdapter() == null) {
                String[] labels = new String[STATUS_LABELS.length];
                for (int i = 0; i < STATUS_LABELS.length; i++) {
                    labels[i] = itemView.getContext().getString(STATUS_LABELS[i]);
                }
                android.widget.ArrayAdapter<String> ad = new android.widget.ArrayAdapter<>(
                        itemView.getContext(), android.R.layout.simple_spinner_dropdown_item, labels);
                spinnerStatus.setAdapter(ad);
            }
            for (int i = 0; i < STATUS_VALUES.length; i++) {
                if (STATUS_VALUES[i].equals(currentStatus)) {
                    spinnerStatus.setSelection(i);
                    break;
                }
            }
        }

        // Метод getStatusLabel: возвращает нужное значение для текущего контекста.
        private String getStatusLabel(String status) {
            if (status == null) return "";
            for (int i = 0; i < STATUS_VALUES.length; i++) {
                if (STATUS_VALUES[i].equals(status)) {
                    return itemView.getContext().getString(STATUS_LABELS[i]);
                }
            }
            return status;
        }

        @SuppressWarnings("unchecked")
        // Метод getMessagesList: возвращает нужное значение для текущего контекста.
        private List<Map<String, Object>> getMessagesList(Map<String, Object> req) {
            Object m = req.get("messages");
            if (m instanceof List) {
                return (List<Map<String, Object>>) m;
            }
            return null;
        }

        // Метод safeStr: выполняет основную бизнес- или UI-логику данного участка кода.
        private String safeStr(Object o) {
            return o == null ? "" : String.valueOf(o);
        }

        // Метод formatDate: выполняет основную бизнес- или UI-логику данного участка кода.
        private String formatDate(String iso) {
            if (iso == null || iso.isEmpty()) return "";
            try {
                SimpleDateFormat in = new SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss", Locale.US);
                Date d = in.parse(iso);
                if (d == null) return iso;
                SimpleDateFormat out = new SimpleDateFormat("dd.MM.yyyy HH:mm", Locale.getDefault());
                return out.format(d);
            } catch (ParseException e) {
                return iso;
            }
        }
    }
}
