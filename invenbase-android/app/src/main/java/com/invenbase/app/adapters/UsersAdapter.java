package com.invenbase.app.adapters;

import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.TextView;

import androidx.annotation.NonNull;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.models.User;

import java.util.ArrayList;
import java.util.List;

public class UsersAdapter extends RecyclerView.Adapter<UsersAdapter.UserViewHolder> {

    public interface OnUserActionListener {
        void onEdit(User user);
        void onDelete(User user);
    }

    private final OnUserActionListener listener;
    private final User currentUser;
    private List<User> items = new ArrayList<>();

    public UsersAdapter(OnUserActionListener listener, User currentUser) {
        this.listener = listener;
        this.currentUser = currentUser;
    }

    public void setItems(List<User> items) {
        this.items = items != null ? items : new ArrayList<>();
        notifyDataSetChanged();
    }

    @NonNull
    @Override
    public UserViewHolder onCreateViewHolder(@NonNull ViewGroup parent, int viewType) {
        View view = LayoutInflater.from(parent.getContext())
                .inflate(R.layout.item_user, parent, false);
        return new UserViewHolder(view);
    }

    @Override
    public void onBindViewHolder(@NonNull UserViewHolder holder, int position) {
        holder.bind(items.get(position));
    }

    @Override
    public int getItemCount() {
        return items.size();
    }

    class UserViewHolder extends RecyclerView.ViewHolder {
        private final TextView textUsername;
        private final TextView textFullName;
        private final TextView textEmail;
        private final TextView textRole;
        private final Button buttonEdit;
        private final Button buttonDelete;
        private final TextView textYou;

        UserViewHolder(@NonNull View itemView) {
            super(itemView);
            textUsername = itemView.findViewById(R.id.text_username);
            textFullName = itemView.findViewById(R.id.text_full_name);
            textEmail = itemView.findViewById(R.id.text_email);
            textRole = itemView.findViewById(R.id.text_role);
            buttonEdit = itemView.findViewById(R.id.button_edit);
            buttonDelete = itemView.findViewById(R.id.button_delete);
            textYou = itemView.findViewById(R.id.text_you);
        }

        void bind(User user) {
            textUsername.setText(user.getUsername());
            textFullName.setText(user.getFullName() == null || user.getFullName().isEmpty()
                    ? itemView.getContext().getString(R.string.no_data)
                    : user.getFullName());
            textEmail.setText(user.getEmail() == null || user.getEmail().isEmpty()
                    ? itemView.getContext().getString(R.string.no_data)
                    : user.getEmail());

            String roleText = user.getRole().equals("admin") ? "Администратор" :
                    user.getRole().equals("responsible") ? "Ответственный" : "Пользователь";
            textRole.setText(roleText);

            boolean isCurrentUser = currentUser != null && user.getId().equals(currentUser.getId());
            textYou.setVisibility(isCurrentUser ? View.VISIBLE : View.GONE);
            buttonDelete.setVisibility(isCurrentUser ? View.GONE : View.VISIBLE);

            buttonEdit.setOnClickListener(v -> {
                if (listener != null) listener.onEdit(user);
            });

            buttonDelete.setOnClickListener(v -> {
                if (listener != null) listener.onDelete(user);
            });
        }
    }
}
