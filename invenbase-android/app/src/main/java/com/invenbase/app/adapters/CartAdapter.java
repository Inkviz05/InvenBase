package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.CartItem;

import java.util.ArrayList;
import java.util.List;

public class CartAdapter extends RecyclerView.Adapter<CartAdapter.CartViewHolder> {
    private List<CartItem> items = new ArrayList<>();
    private OnCartActionListener listener;

    public interface OnCartActionListener {
        void onRemove(CartItem item);
    }

    public void setItems(List<CartItem> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    public void setOnCartActionListener(OnCartActionListener listener) {
        this.listener = listener;
    }

    @NonNull
    @Override
    public CartViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
            .inflate(R.layout.item_cart, parent, false);
        return new CartViewHolder(view);
    }

    @Override
    public void onBindViewHolder(@NonNull CartViewHolder holder, int position) {
        holder.bind(items.get(position));
    }

    @Override
    public int getItemCount() {
        return items.size();
    }

    class CartViewHolder extends RecyclerView.ViewHolder {
        private TextView textName;
        private TextView textQuantity;
        private View buttonRemove;

        CartViewHolder(@NonNull View itemView) {
            super(itemView);
            textName = itemView.findViewById(R.id.text_cart_name);
            textQuantity = itemView.findViewById(R.id.text_cart_quantity);
            buttonRemove = itemView.findViewById(R.id.button_cart_remove);
        }

        void bind(CartItem item) {
            textName.setText(item.getEquipmentName());
            textQuantity.setText(itemView.getContext().getString(R.string.cart_quantity, item.getQuantity()));
            buttonRemove.setOnClickListener(v -> {
                if (listener != null) {
                    listener.onRemove(item);
                }
            });
        }
    }
}
