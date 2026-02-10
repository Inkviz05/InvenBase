package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.ReportCategoryItem;

import java.util.ArrayList;
import java.util.List;

public class ReportCategoryAdapter extends RecyclerView.Adapter<ReportCategoryAdapter.ViewHolder> {

    private List<ReportCategoryItem> items = new ArrayList<>();

    public void setItems(List<ReportCategoryItem> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    @NonNull
    @Override
    public ViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_report_category, parent, false);
        return new ViewHolder(view);
    }

    @Override
    public void onBindViewHolder(@NonNull ViewHolder holder, int position) {
        ReportCategoryItem item = items.get(position);
        holder.textCategoryName.setText(item.getCategoryName().isEmpty() ? holder.itemView.getContext().getString(R.string.no_category) : item.getCategoryName());
        holder.textCategoryTotal.setText(holder.itemView.getContext().getString(R.string.report_category_total, item.getTotal()));
        holder.textCategoryAvailable.setText(String.valueOf(item.getAvailable()));
        holder.textCategoryBooked.setText(String.valueOf(item.getBooked()));
    }

    @Override
    public int getItemCount() {
        return items.size();
    }

    static class ViewHolder extends RecyclerView.ViewHolder {
        final TextView textCategoryName;
        final TextView textCategoryTotal;
        final TextView textCategoryAvailable;
        final TextView textCategoryBooked;

        ViewHolder(View itemView) {
            super(itemView);
            textCategoryName = itemView.findViewById(R.id.text_category_name);
            textCategoryTotal = itemView.findViewById(R.id.text_category_total);
            textCategoryAvailable = itemView.findViewById(R.id.text_category_available);
            textCategoryBooked = itemView.findViewById(R.id.text_category_booked);
        }
    }
}
