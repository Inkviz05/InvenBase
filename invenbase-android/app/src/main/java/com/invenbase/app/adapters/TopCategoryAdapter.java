package com.invenbase.app.adapters;

import android.content.Context;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.TopCategoryRow;

import java.util.ArrayList;
import java.util.List;

public class TopCategoryAdapter extends RecyclerView.Adapter<RecyclerView.ViewHolder> {
    private static final int TYPE_HEADER = 0;
    private static final int TYPE_ROW = 1;

    private final Context context;
    private List<TopCategoryRow> items = new ArrayList<>();

    // Конструктор TopCategoryAdapter: инициализирует объект и его зависимости.
    public TopCategoryAdapter(Context context) {
        this.context = context;
    }

    // Метод setItems: устанавливает или обновляет значение данных.
    public void setItems(List<TopCategoryRow> items) {
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
            View v = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_top_category_header, parent, false);
            return new HeaderVH(v);
        }
        View v = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_top_category_row, parent, false);
        return new VH(v);
    }

    @Override
    // Метод onBindViewHolder: обрабатывает соответствующее событие приложения.
    public void onBindViewHolder(@NonNull RecyclerView.ViewHolder holder, int position) {
        if (holder instanceof HeaderVH) {
            HeaderVH h = (HeaderVH) holder;
            h.textNum.setText(context.getString(R.string.report_col_num));
            h.textCategory.setText(context.getString(R.string.category));
            h.textCount.setText(context.getString(R.string.report_col_bookings));
            h.textPercent.setText(context.getString(R.string.report_percent));
        } else {
            TopCategoryRow row = items.get(position - 1);
            VH vh = (VH) holder;
            vh.textNum.setText(String.valueOf(row.index + 1));
            vh.textCategory.setText(row.categoryName);
            vh.textCount.setText(String.valueOf(row.bookingsCount));
            vh.textPercent.setText(row.percent + "%");
        }
    }

    @Override
    // Метод getItemCount: возвращает нужное значение для текущего контекста.
    public int getItemCount() {
        return 1 + items.size();
    }

    static class HeaderVH extends RecyclerView.ViewHolder {
        final TextView textNum, textCategory, textCount, textPercent;

        HeaderVH(@NonNull View itemView) {
            super(itemView);
            textNum = itemView.findViewById(R.id.text_top_num);
            textCategory = itemView.findViewById(R.id.text_top_category);
            textCount = itemView.findViewById(R.id.text_top_count);
            textPercent = itemView.findViewById(R.id.text_top_percent);
        }
    }

    static class VH extends RecyclerView.ViewHolder {
        final TextView textNum, textCategory, textCount, textPercent;

        VH(@NonNull View itemView) {
            super(itemView);
            textNum = itemView.findViewById(R.id.text_top_num);
            textCategory = itemView.findViewById(R.id.text_top_category);
            textCount = itemView.findViewById(R.id.text_top_count);
            textPercent = itemView.findViewById(R.id.text_top_percent);
        }
    }
}
