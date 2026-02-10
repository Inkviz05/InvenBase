package com.invenbase.app.api;

import com.invenbase.app.models.Booking;
import com.invenbase.app.models.Category;
import com.invenbase.app.models.Equipment;
import com.invenbase.app.models.LoginRequest;
import com.invenbase.app.models.LoginResponse;
import com.invenbase.app.models.User;

import java.util.List;
import java.util.Map;

import okhttp3.ResponseBody;
import retrofit2.Call;
import retrofit2.http.Body;
import retrofit2.http.DELETE;
import retrofit2.http.GET;
import retrofit2.http.POST;
import retrofit2.http.PUT;
import retrofit2.http.Path;
import retrofit2.http.Query;

public interface ApiService {
    // Auth
    @POST("/api/auth/login")
    Call<LoginResponse> login(@Body LoginRequest request);
    
    @GET("/api/users/me")
    Call<User> getCurrentUser();
    
    // Equipment
    @GET("/api/equipment")
    Call<List<Equipment>> getEquipment();
    
    @GET("/api/equipment/{id}")
    Call<Equipment> getEquipmentById(@Path("id") String id);
    
    @GET("/api/equipment/qr/{qrCode}")
    Call<Equipment> getEquipmentByQR(@Path("qrCode") String qrCode);
    
    @POST("/api/equipment")
    Call<Equipment> createEquipment(@Body Map<String, Object> data);
    
    @PUT("/api/equipment/{id}")
    Call<Equipment> updateEquipment(@Path("id") String id, @Body Map<String, Object> data);
    
    @DELETE("/api/equipment/{id}")
    Call<Void> deleteEquipment(@Path("id") String id);
    
    // Bookings
    @GET("/api/bookings")
    Call<List<Booking>> getBookings();
    
    @GET("/api/bookings/{id}")
    Call<Booking> getBookingById(@Path("id") String id);
    
    @POST("/api/bookings")
    Call<Booking> createBooking(@Body Map<String, Object> data);
    
    @PUT("/api/bookings/{id}")
    Call<Booking> updateBooking(@Path("id") String id, @Body Map<String, Object> data);
    
    @POST("/api/bookings/{id}/approve")
    Call<Booking> approveBooking(@Path("id") String id);
    
    @POST("/api/bookings/{id}/reject")
    Call<Booking> rejectBooking(@Path("id") String id);
    
    @DELETE("/api/bookings/{id}")
    Call<Void> deleteBooking(@Path("id") String id);
    
    // Categories
    @GET("/api/categories")
    Call<List<Category>> getCategories();
    
    @GET("/api/categories/{id}")
    Call<Category> getCategoryById(@Path("id") String id);
    
    @POST("/api/categories")
    Call<Category> createCategory(@Body Map<String, Object> data);
    
    @PUT("/api/categories/{id}")
    Call<Category> updateCategory(@Path("id") String id, @Body Map<String, Object> data);
    
    @DELETE("/api/categories/{id}")
    Call<Void> deleteCategory(@Path("id") String id);
    
    // Users (Admin only)
    @GET("/api/users")
    Call<List<User>> getUsers();
    
    @GET("/api/users/{id}")
    Call<User> getUserById(@Path("id") String id);
    
    @POST("/api/users")
    Call<User> createUser(@Body Map<String, Object> data);
    
    @PUT("/api/users/{id}")
    Call<User> updateUser(@Path("id") String id, @Body Map<String, Object> data);
    
    @DELETE("/api/users/{id}")
    Call<Void> deleteUser(@Path("id") String id);
    
    // Notifications
    @GET("/api/notifications")
    Call<List<Map<String, Object>>> getNotifications();
    
    @GET("/api/notifications/unread-count")
    Call<Map<String, Integer>> getUnreadCount();
    
    @POST("/api/notifications/{id}/read")
    Call<Void> markAsRead(@Path("id") String id);
    
    // Reports (Admin/Responsible only)
    @GET("/api/reports/equipment")
    Call<Map<String, Object>> getEquipmentReport();
    
    @GET("/api/reports/bookings")
    Call<Map<String, Object>> getBookingReport();
    
    // Logs (Admin/Responsible only)
    @GET("/api/logs")
    Call<List<Map<String, Object>>> getLogs();

    // Push notifications
    @POST("/api/push/register-token")
    Call<Void> registerPushToken(@Body Map<String, String> data);
    
    // QR Code
    @GET("/api/qr/{id}")
    Call<ResponseBody> generateQRCode(@Path("id") String id);
    
    @GET("/api/qr/{id}/data")
    Call<Map<String, Object>> getQRCodeData(@Path("id") String id);
}
