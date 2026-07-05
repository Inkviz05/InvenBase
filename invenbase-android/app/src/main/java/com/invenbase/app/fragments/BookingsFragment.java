package com.invenbase.app.fragments;

import android.app.AlertDialog;
import android.os.Bundle;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ProgressBar;
import android.widget.Toast;

import androidx.annotation.NonNull;
import androidx.annotation.Nullable;
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

    @Nullable
    @Override
    // Метод onCreateView: обрабатывает соответствующее событие приложения.
    public View onCreateView(@NonNull LayoutInflater inflater, @Nullable ViewGroup container, @Nullable Bundle savedInstanceState) {
        View view = inflater.inflate(R.layout.fragment_bookings, container, false);
        
        recyclerBookings = view.findViewById(R.id.recycler_bookings);
        progressBar = view.findViewById(R.id.progress_bar);
        
        recyclerBookings.setLayoutManager(new LinearLayoutManager(requireContext()));
        adapter = new BookingAdapter();
        recyclerBookings.setAdapter(adapter);
        
        apiService = ApiClient.getInstance(requireContext()).getApiService();
        authManager = AuthManager.getInstance(requireContext());
        
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
                deleteBooking(booking);
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
                    adapter.setBookingList(response.body());
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

    // Метод deleteBooking: выполняет основную бизнес- или UI-логику данного участка кода.
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

    private void deleteBooking(Booking booking) {
        new AlertDialog.Builder(requireContext())
                .setMessage(R.string.confirm_delete_booking)
                .setNegativeButton(R.string.cancel, null)
                .setPositiveButton(R.string.delete, (dialog, which) -> {
                    progressBar.setVisibility(View.VISIBLE);
                    apiService.deleteBooking(booking.getId()).enqueue(new Callback<Void>() {
                        @Override
                        // Метод onResponse: обрабатывает соответствующее событие приложения.
                        public void onResponse(Call<Void> call, Response<Void> response) {
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
                        public void onFailure(Call<Void> call, Throwable t) {
                            progressBar.setVisibility(View.GONE);
                            Toast.makeText(requireContext(), ApiErrorParser.fromThrowable(requireContext(), t), Toast.LENGTH_LONG).show();
                        }
                    });
                })
                .show();
    }
}
