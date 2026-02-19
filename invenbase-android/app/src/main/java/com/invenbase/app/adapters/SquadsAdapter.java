package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.google.android.material.button.MaterialButton;
import com.invenbase.app.R;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

public class SquadsAdapter extends RecyclerView.Adapter<SquadsAdapter.SquadViewHolder> {

    public interface OnSquadClickListener {
        void onSquadClick(Map<String, Object> squad);
        void onEditClick(Map<String, Object> squad);
        void onDeleteClick(Map<String, Object> squad);
    }

    private final OnSquadClickListener listener;
    private final boolean canEdit;
    private List<Map<String, Object>> items = new ArrayList<>();

    public SquadsAdapter(OnSquadClickListener listener, boolean canEdit) {
        this.listener = listener;
        this.canEdit = canEdit;
    }

    public void setItems(List<Map<String, Object>> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    private static String str(Object o) {
        return o == null ? "" : String.valueOf(o);
    }

    @NonNull
    @Override
    public SquadViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext()).inflate(R.layout.item_squad, parent, false);
        return new SquadViewHolder(view);
    }

    @Override
    public void onBindViewHolder(@NonNull SquadViewHolder holder, int position) {
        Map<String, Object> squad = items.get(position);
        holder.textName.setText(str(squad.get("name")));
        holder.textLocation.setText(str(squad.get("location")).isEmpty() ? "—" : str(squad.get("location")));
        holder.textResponsible.setText(str(squad.get("responsible_name")).isEmpty() ? "—" : str(squad.get("responsible_name")));
        String desc = str(squad.get("description"));
        holder.textDescription.setText(desc.isEmpty() ? "—" : desc);
        holder.textDescription.setVisibility(desc.isEmpty() ? View.GONE : View.VISIBLE);

        holder.itemView.setOnClickListener(v -> listener.onSquadClick(squad));
        holder.buttonEdit.setVisibility(canEdit ? View.VISIBLE : View.GONE);
        holder.buttonDelete.setVisibility(canEdit ? View.VISIBLE : View.GONE);
        if (canEdit) {
            holder.buttonEdit.setOnClickListener(v -> listener.onEditClick(squad));
            holder.buttonDelete.setOnClickListener(v -> listener.onDeleteClick(squad));
        }
    }

    @Override
    public int getItemCount() {
        return items.size();
    }

    static class SquadViewHolder extends RecyclerView.ViewHolder {
        final TextView textName;
        final TextView textLocation;
        final TextView textResponsible;
        final TextView textDescription;
        final MaterialButton buttonEdit;
        final MaterialButton buttonDelete;

        SquadViewHolder(@NonNull View itemView) {
            super(itemView);
            textName = itemView.findViewById(R.id.text_squad_name);
            textLocation = itemView.findViewById(R.id.text_squad_location);
            textResponsible = itemView.findViewById(R.id.text_squad_responsible);
            textDescription = itemView.findViewById(R.id.text_squad_description);
            buttonEdit = itemView.findViewById(R.id.button_edit_squad);
            buttonDelete = itemView.findViewById(R.id.button_delete_squad);
        }
    }
}
