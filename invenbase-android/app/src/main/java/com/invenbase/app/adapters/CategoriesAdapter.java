package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.Category;

import java.util.ArrayList;
import java.util.List;

public class CategoriesAdapter extends RecyclerView.Adapter<CategoriesAdapter.CategoryViewHolder> {

    public interface OnCategoryActionListener {
        // Метод onEdit: обрабатывает соответствующее событие приложения.
        void onEdit(Category category);
        // Метод onDelete: обрабатывает соответствующее событие приложения.
        void onDelete(Category category);
    }

    private final OnCategoryActionListener listener;
    private final boolean canDelete;
    private List<Category> items = new ArrayList<>();

    // Конструктор CategoriesAdapter: инициализирует объект и его зависимости.
    public CategoriesAdapter(OnCategoryActionListener listener, boolean canDelete) {
        this.listener = listener;
        this.canDelete = canDelete;
    }

    // Метод setItems: устанавливает или обновляет значение данных.
    public void setItems(List<Category> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    @NonNull
    @Override
    // Метод onCreateViewHolder: обрабатывает соответствующее событие приложения.
    public CategoryViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_category, parent, false);
        return new CategoryViewHolder(view);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull CategoryViewHolder holder, int position) {
        holder.bind(items.get(position));
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return items.size();
    }

    class CategoryViewHolder extends RecyclerView.ViewHolder {
        private final TextView textName;
        private final TextView textDescription;
        private final Button buttonEdit;
        private final Button buttonDelete;

        CategoryViewHolder(@NonNull View itemView) {
            super(itemView);
            textName = itemView.findViewById(R.id.text_name);
            textDescription = itemView.findViewById(R.id.text_description);
            buttonEdit = itemView.findViewById(R.id.button_edit);
            buttonDelete = itemView.findViewById(R.id.button_delete);
        }

        // Метод bind: выполняет основную бизнес- или UI-логику данного участка кода.
        void bind(Category category) {
            textName.setText(category.getName());
            textDescription.setText(category.getDescription() == null || category.getDescription().isEmpty()
                    ? itemView.getContext().getString(R.string.no_data)
                    : category.getDescription());

            buttonEdit.setOnClickListener(v -> {
                if (listener != null) listener.onEdit(category);
            });

            buttonDelete.setVisibility(canDelete ? View.VISIBLE : View.GONE);
            buttonDelete.setOnClickListener(v -> {
                if (listener != null) listener.onDelete(category);
            });
        }
    }
}

