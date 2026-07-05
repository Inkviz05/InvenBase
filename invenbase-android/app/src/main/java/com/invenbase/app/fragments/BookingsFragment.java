package com.invenbase.app.fragments;

import android.app.AlertDialog;
import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.Button;
import android.widget.ProgressBar;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
import androidx.core.content.ContextCompat;
import androidx.fragment.app.Fragment;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;

import com.invenbase.app.R;
import com.invenbase.app.adapters.BookingAdapter;
import com.invenbase.app.api.ApiClient;
import com.invenbase.app.api.ApiService;
import com.invenbase.app.models.Booking;
import com.invenbase.app.utils.ApiErrorParser;
import com.invenbase.app.utils.AuthManager;

import java.util.ArrayList;
import java.util.List;

import retrofit2.Call;
import retrofit2.Callback;
import retrofit2.Response;

public class BookingsFragment extends Fragment {
    private RecyclerView recyclerBookings;
    private ProgressBar progressBar;
    private BookingAdapter adapter;
    private ApiService apiService;
    private AuthManager authManager;
    private final List<Booking> allBookings = new ArrayList<>();
    private String currentFilter = "all";
    private Button filterAll;
    private Button filterPending;
    private Button filterApproved;
    private Button filterAwaitingReturn;
    private Button filterReturned;
    private Button filterRejected;
    private Button filterCancelled;

    @Nullable
    @Override
    // Метод onCreateView: обрабатывает соответствующее событие приложения.
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container, @Nullable Bundle savedInstanceState) {
        View view = inflater.inflate(R.layout.fragment_bookings, container, false);
        
        recyclerBookings = view.findViewById(R.id.recycler_bookings);
        progressBar = view.findViewById(R.id.progress_bar);
        filterAll = view.findViewById(R.id.filter_all);
        filterPending = view.findViewById(R.id.filter_pending);
        filterApproved = view.findViewById(R.id.filter_approved);
        filterAwaitingReturn = view.findViewById(R.id.filter_awaiting_return);
        filterReturned = view.findViewById(R.id.filter_returned);
        filterRejected = view.findViewById(R.id.filter_rejected);
        filterCancelled = view.findViewById(R.id.filter_cancelled);
        
        recyclerBookings.setLayoutManager(new LinearLayoutManager(requireContext()));
        adapter = new BookingAdapter();
        recyclerBookings.setAdapter(adapter);
        
        apiService = ApiClient.getInstance(requireContext()).getApiService();
        authManager = AuthManager.getInstance(requireContext());
        setupFilters();
        
        // Настраиваем адаптер
        boolean isAdminOrResponsible = authManager.isAdmin() || authManager.isResponsible();
        adapter.setAdminOrResponsible(isAdminOrResponsible);
        adapter.setOnBookingActionListener(new BookingAdapter.OnBookingActionListener() {
            @Override
            // Метод onApprove: обрабатывает соответствующее событие приложения.
            public void onApprove(Booking booking) {
                approveBooking(booking);
            }

            @Override
            // Метод onReject: обрабатывает соответствующее событие приложения.
            public void onReject(Booking booking) {
                rejectBooking(booking);
            }

            @Override
            public void onConfirmReturn(Booking booking) {
                confirmBookingReturn(booking);
            }

            @Override
            // Метод onDelete: обрабатывает соответствующее событие приложения.
            public void onDelete(Booking booking) {
                cancelBooking(booking);
            }
        });
        
        loadBookings();
        
        return view;
    }

    // Метод loadBookings: выполняет основную бизнес- или UI-логику данного участка кода.
    private void loadBookings() {
        progressBar.setVisibility(View.VISIBLE);
        
        Call<List<Booking>> call = apiService.getBookings();
        call.enqueue(new Callback<List<Booking>>() {
            @Override
            // Метод onResponse: обрабатывает соответствующее событие приложения.
            public void onResponse(Call<List<Booking>> call, Response<List<Booking>> response) {
                progressBar.setVisibility(View.GONE);
                
                if (response.isSuccessful() && response.body() != null) {
                    allBookings.clear();
                    allBookings.addAll(response.body());
                    applyCurrentFilter();
                } else {
                    Toast.makeText(requireContext(), ApiErrorParser.fromResponse(requireContext(), response), Toast.LENGTH_LONG).show();
                }
            }

            @Override
            // Метод onFailure: обрабатывает соответствующее событие приложения.
            public void onFailure(Call<List<Booking>> call, Throwable t) {
                progressBar.setVisibility(View.GONE);
                Toast.makeText(requireContext(), ApiErrorParser.fromThrowable(requireContext(), t), Toast.LENGTH_LONG).show();
            }
        });
    }

    private void setupFilters() {
        bindFilter(filterAll, "all");
        bindFilter(filterPending, "pending");
        bindFilter(filterApproved, "approved");
        bindFilter(filterAwaitingReturn, "awaiting_return");
        bindFilter(filterReturned, "returned");
        bindFilter(filterRejected, "rejected");
        bindFilter(filterCancelled, "cancelled");
        updateFilterButtons();
    }

    private void bindFilter(Button button, String status) {
        if (button == null) return;
        button.setOnClickListener(v -> {
            currentFilter = status;
            applyCurrentFilter();
        });
    }

    private void applyCurrentFilter() {
        if ("all".equals(currentFilter)) {
            adapter.setBookingList(new ArrayList<>(allBookings));
        } else {
            List<Booking> filtered = new ArrayList<>();
            for (Booking booking : allBookings) {
                if (booking != null && currentFilter.equals(booking.getStatus())) {
                    filtered.add(booking);
                }
            }
            adapter.setBookingList(filtered);
        }
        updateFilterButtons();
    }

    private void updateFilterButtons() {
        updateFilterButton(filterAll, "all");
        updateFilterButton(filterPending, "pending");
        updateFilterButton(filterApproved, "approved");
        updateFilterButton(filterAwaitingReturn, "awaiting_return");
        updateFilterButton(filterReturned, "returned");
        updateFilterButton(filterRejected, "rejected");
        updateFilterButton(filterCancelled, "cancelled");
    }

    private void updateFilterButton(Button button, String status) {
        if (button == null || getContext() == null) return;
        boolean active = status.equals(currentFilter);
        button.setSelected(active);
        button.setTextColor(ContextCompat.getColor(requireContext(), active ? R.color.surface : R.color.text_primary));
        button.setBackgroundTintList(ContextCompat.getColorStateList(requireContext(), active ? R.color.primary : R.color.surface));
    }

    // Метод approveBooking: выполняет основную бизнес- или UI-логику данного участка кода.
    private void approveBooking(Booking booking) {
        new AlertDialog.Builder(requireContext())
                .setMessage("Одобрить бронирование?")
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.approve, (dialog, which) -> {
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.approveBooking(booking.getId()).enqueue(new Callback<Booking>() {
                        @Override
                        // Метод onResponse: обрабатывает соответствующее событие приложения.
                        public void onResponse(Call<Booking> call, Response<Booking> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful()) {
                                Toast.makeText(requireContext(), R.string.booking_approved, Toast.LENGTH_SHORT).show();
                                loadBookings();
                            } else {
                                Toast.makeText(requireContext(), ApiErrorParser.fromResponse(requireContext(), response), Toast.LENGTH_LONG).show();
                            }
                        }

                        @Override
                        // Метод onFailure: обрабатывает соответствующее событие приложения.
                        public void onFailure(Call<Booking> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(requireContext(), ApiErrorParser.fromThrowable(requireContext(), t), Toast.LENGTH_LONG).show();
                        }
                    });
                })
                .show();
    }

    // Метод rejectBooking: выполняет основную бизнес- или UI-логику данного участка кода.
    private void rejectBooking(Booking booking) {
        new AlertDialog.Builder(requireContext())
                .setMessage("Отклонить бронирование?")
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.reject, (dialog, which) -> {
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.rejectBooking(booking.getId()).enqueue(new Callback<Booking>() {
                        @Override
                        // Метод onResponse: обрабатывает соответствующее событие приложения.
                        public void onResponse(Call<Booking> call, Response<Booking> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful()) {
                                Toast.makeText(requireContext(), R.string.booking_rejected, Toast.LENGTH_SHORT).show();
                                loadBookings();
                            } else {
                                Toast.makeText(requireContext(), ApiErrorParser.fromResponse(requireContext(), response), Toast.LENGTH_LONG).show();
                            }
                        }

                        @Override
                        // Метод onFailure: обрабатывает соответствующее событие приложения.
                        public void onFailure(Call<Booking> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(requireContext(), ApiErrorParser.fromThrowable(requireContext(), t), Toast.LENGTH_LONG).show();
                        }
                    });
                })
                .show();
    }

    // Confirms actual equipment return.
    private void confirmBookingReturn(Booking booking) {
        new AlertDialog.Builder(requireContext())
                .setMessage(R.string.confirm_return_booking)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.confirm_return, (dialog, which) -> {
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.confirmBookingReturn(booking.getId()).enqueue(new Callback<Booking>() {
                        @Override
                        public void onResponse(Call<Booking> call, Response<Booking> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful()) {
                                Toast.makeText(requireContext(), R.string.booking_returned, Toast.LENGTH_SHORT).show();
                                loadBookings();
                            } else {
                                Toast.makeText(requireContext(), ApiErrorParser.fromResponse(requireContext(), response), Toast.LENGTH_LONG).show();
                            }
                        }

                        @Override
                        public void onFailure(Call<Booking> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(requireContext(), ApiErrorParser.fromThrowable(requireContext(), t), Toast.LENGTH_LONG).show();
                        }
                    });
                })
                .show();
    }

    private void cancelBooking(Booking booking) {
        new AlertDialog.Builder(requireContext())
                .setMessage(R.string.confirm_delete_booking)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.cancel_booking_action, (dialog, which) -> {
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.cancelBooking(booking.getId()).enqueue(new Callback<Booking>() {
                        @Override
                        // Метод onResponse: обрабатывает соответствующее событие приложения.
                        public void onResponse(Call<Booking> call, Response<Booking> response) {
                            progressBar.setVisibility(View.GONE);
                            if (response.isSuccessful()) {
                                Toast.makeText(requireContext(), R.string.booking_deleted, Toast.LENGTH_SHORT).show();
                                loadBookings();
                            } else {
                                Toast.makeText(requireContext(), ApiErrorParser.fromResponse(requireContext(), response), Toast.LENGTH_LONG).show();
                            }
                        }

                        @Override
                        // Метод onFailure: обрабатывает соответствующее событие приложения.
                        public void onFailure(Call<Booking> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(requireContext(), ApiErrorParser.fromThrowable(requireContext(), t), Toast.LENGTH_LONG).show();
                        }
                    });
                })
                .show();
    }
}
