package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.core.content.ContextCompat;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.Booking;

import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.ArrayList;
import java.util.Date;
import java.util.List;
import java.util.Locale;

// ООП: наследование от RecyclerView.Adapter + композиция (список Booking и callback-интерфейсы).
public class BookingAdapter extends RecyclerView.Adapter<BookingAdapter.BookingViewHolder> {
    private List<Booking> bookingList;
    private OnBookingClickListener listener;
    private OnBookingActionListener actionListener;
    private boolean isAdminOrResponsible;

    public interface OnBookingClickListener {
        // ООП (интерфейс): контракт обратного вызова при клике по элементу.
        void onBookingClick(Booking booking);
    }

    public interface OnBookingActionListener {
        // ООП (интерфейс): контракты действий над бронированием, реализуются внешним слоем (Activity/Fragment).
        void onApprove(Booking booking);
        // ООП (интерфейс): контракт отклонения бронирования.
        void onReject(Booking booking);
        void onConfirmReturn(Booking booking);
        // ООП (интерфейс): контракт удаления бронирования.
        void onDelete(Booking booking);
    }

    // Конструктор BookingAdapter: инициализирует объект и его зависимости.
    public BookingAdapter() {
        this.bookingList = new ArrayList<>();
    }

    // Метод setBookingList: устанавливает или обновляет значение данных.
    public void setBookingList(List<Booking> bookingList) {
        this.bookingList = bookingList;
        notifyDataSetChanged();
    }

    // Метод setOnBookingClickListener: устанавливает или обновляет значение данных.
    public void setOnBookingClickListener(OnBookingClickListener listener) {
        this.listener = listener;
    }

    // Метод setOnBookingActionListener: устанавливает или обновляет значение данных.
    public void setOnBookingActionListener(OnBookingActionListener listener) {
        this.actionListener = listener;
    }

    // Метод setAdminOrResponsible: устанавливает или обновляет значение данных.
    public void setAdminOrResponsible(boolean isAdminOrResponsible) {
        this.isAdminOrResponsible = isAdminOrResponsible;
    }

    @NonNull
    @Override
    // ООП (полиморфизм): реализация обязательного метода базового Adapter для создания ViewHolder.
    public BookingViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
            .inflate(R.layout.item_booking, parent, false);
        return new BookingViewHolder(view);
    }

    @Override
    // ООП (полиморфизм): связываем модель Booking с конкретным представлением ViewHolder.
    public void onBindViewHolder(@NonNull BookingViewHolder holder, int position) {
        Booking booking = bookingList.get(position);
        holder.bind(booking);
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return bookingList.size();
    }

    // ООП: внутренний класс инкапсулирует привязку UI-компонентов одного элемента списка.
    class BookingViewHolder extends RecyclerView.ViewHolder {
        private TextView textEquipmentName;
        private TextView textUser;
        private TextView textDates;
        private TextView textStatus;
        private Button buttonApprove;
        private Button buttonReject;
        private Button buttonConfirmReturn;
        private Button buttonDelete;
        private View actionContainer;

        // Конструктор BookingViewHolder: инициализирует объект и его зависимости.
        public BookingViewHolder(@NonNull View itemView) {
            super(itemView);
            textEquipmentName = itemView.findViewById(R.id.text_equipment_name);
            textUser = itemView.findViewById(R.id.text_user);
            textDates = itemView.findViewById(R.id.text_dates);
            textStatus = itemView.findViewById(R.id.text_status);
            buttonApprove = itemView.findViewById(R.id.button_approve);
            buttonReject = itemView.findViewById(R.id.button_reject);
            buttonConfirmReturn = itemView.findViewById(R.id.button_confirm_return);
            buttonDelete = itemView.findViewById(R.id.button_delete);
            actionContainer = itemView.findViewById(R.id.action_container);

            itemView.setOnClickListener(v -> {
                if (listener != null) {
                    listener.onBookingClick(bookingList.get(getAdapterPosition()));
                }
            });

            if (buttonApprove != null) {
                buttonApprove.setOnClickListener(v -> {
                    if (actionListener != null) {
                        actionListener.onApprove(bookingList.get(getAdapterPosition()));
                    }
                });
            }

            if (buttonReject != null) {
                buttonReject.setOnClickListener(v -> {
                    if (actionListener != null) {
                        actionListener.onReject(bookingList.get(getAdapterPosition()));
                    }
                });
            }

            if (buttonConfirmReturn != null) {
                buttonConfirmReturn.setOnClickListener(v -> {
                    if (actionListener != null) {
                        actionListener.onConfirmReturn(bookingList.get(getAdapterPosition()));
                    }
                });
            }

            if (buttonDelete != null) {
                buttonDelete.setOnClickListener(v -> {
                    if (actionListener != null) {
                        actionListener.onDelete(bookingList.get(getAdapterPosition()));
                    }
                });
            }
        }

        // Метод bind: выполняет основную бизнес- или UI-логику данного участка кода.
        public void bind(Booking booking) {
            String equipmentName = booking.getEquipmentName() != null ? 
                booking.getEquipmentName() : 
                (booking.getGroupName() != null ? booking.getGroupName() : "Не указано");
            textEquipmentName.setText(equipmentName);
            textUser.setText("Пользователь: " + (booking.getUsername() != null ? booking.getUsername() : "Неизвестно"));
            
            // Форматируем даты
            String dates = formatDate(booking.getStartDate()) + " - " + formatDate(booking.getEndDate());
            textDates.setText(dates);
            
            // Устанавливаем статус с цветом
            String statusText = getStatusText(booking.getStatus());
            int statusColor = getStatusColor(booking.getStatus());
            textStatus.setText(statusText);
            textStatus.setTextColor(ContextCompat.getColor(itemView.getContext(), statusColor));

            // Показываем кнопки действий только для админов/ответственных
            if (actionContainer != null) {
                if (isAdminOrResponsible) {
                    actionContainer.setVisibility(View.VISIBLE);
                    if (buttonApprove != null) {
                        buttonApprove.setVisibility("pending".equals(booking.getStatus()) ? View.VISIBLE : View.GONE);
                    }
                    if (buttonReject != null) {
                        buttonReject.setVisibility("pending".equals(booking.getStatus()) ? View.VISIBLE : View.GONE);
                    }
                    if (buttonConfirmReturn != null) {
                        buttonConfirmReturn.setVisibility("awaiting_return".equals(booking.getStatus()) ? View.VISIBLE : View.GONE);
                    }
                    if (buttonDelete != null) {
                        buttonDelete.setVisibility(canCancelBooking(booking.getStatus()) ? View.VISIBLE : View.GONE);
                    }
                } else {
                    actionContainer.setVisibility(View.GONE);
                }
            }
        }

        // Метод formatDate: выполняет основную бизнес- или UI-логику данного участка кода.
        private String formatDate(String dateString) {
            if (dateString == null) return "";
            try {
                SimpleDateFormat inputFormat = new SimpleDateFormat("yyyy-MM-dd'T'HH:mm:ss", Locale.getDefault());
                SimpleDateFormat outputFormat = new SimpleDateFormat("dd.MM.yyyy HH:mm", Locale.getDefault());
                Date date = inputFormat.parse(dateString);
                return date != null ? outputFormat.format(date) : dateString;
            } catch (ParseException e) {
                return dateString;
            }
        }

        // Метод getStatusText: возвращает нужное значение для текущего контекста.
        private String getStatusText(String status) {
            if (status == null) return "";
            switch (status) {
                case "pending": return "Ожидает одобрения";
                case "approved": return "Одобрено";
                case "rejected": return "Отклонено";
                case "awaiting_return": return "Ожидает возврата";
                case "returned": return "Возвращено";
                case "expired": return "Истекло";
                case "cancelled": return "Отменено";
                default: return status;
            }
        }

        // Метод getStatusColor: возвращает нужное значение для текущего контекста.
        private int getStatusColor(String status) {
            if (status == null) return R.color.text_secondary;
            switch (status) {
                case "pending": return R.color.warning;
                case "approved": return R.color.success;
                case "rejected": return R.color.error;
                case "awaiting_return": return R.color.warning;
                case "returned": return R.color.success;
                case "expired": return R.color.text_secondary;
                case "cancelled": return R.color.text_secondary;
                default: return R.color.text_secondary;
            }
        }

        private boolean canCancelBooking(String status) {
            return "pending".equals(status) || "approved".equals(status) || "awaiting_return".equals(status);
        }
    }
}
