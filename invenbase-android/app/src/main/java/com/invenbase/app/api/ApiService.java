package com.invenbase.app.api;

import com.invenbase.app.models.Booking;
import com.invenbase.app.models.BulkBookingsRequest;
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

// ООП (абстракция): интерфейс описывает контракт API без привязки к конкретной реализации клиента.
public interface ApiService {
    // Auth
    @POST("/api/auth/login")
    // ООП (контракт): авторизация как обязательная операция для любого API-клиента.
    Call<LoginResponse> login(@Body LoginRequest request);
    
    @GET("/api/users/me")
    // Метод getCurrentUser: возвращает нужное значение для текущего контекста.
    Call<User> getCurrentUser();
    
    // Equipment
    @GET("/api/equipment")
    // ООП (полиморфизм через Retrofit): реальная реализация метода генерируется динамически.
    Call<List<Equipment>> getEquipment();
    
    @GET("/api/equipment/{id}")
    // Метод getEquipmentById: возвращает нужное значение для текущего контекста.
    Call<Equipment> getEquipmentById(@Path("id") String id);

    @GET("/api/equipment/{id}/movements")
    // Метод getEquipmentMovements: возвращает нужное значение для текущего контекста.
    Call<List<Map<String, Object>>> getEquipmentMovements(@Path("id") String id);

    @POST("/api/equipment/{id}/move")
    // Метод moveEquipment: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Map<String, Object>> moveEquipment(@Path("id") String id, @Body Map<String, Object> data);
    
    @GET("/api/equipment/qr/{qrCode}")
    // Метод getEquipmentByQR: возвращает нужное значение для текущего контекста.
    Call<Equipment> getEquipmentByQR(@Path("qrCode") String qrCode);
    
    @POST("/api/equipment")
    // Метод createEquipment: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Equipment> createEquipment(@Body Map<String, Object> data);
    
    @PUT("/api/equipment/{id}")
    // Метод updateEquipment: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Equipment> updateEquipment(@Path("id") String id, @Body Map<String, Object> data);
    
    @DELETE("/api/equipment/{id}")
    // Метод deleteEquipment: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Void> deleteEquipment(@Path("id") String id);
    
    // Bookings
    @GET("/api/bookings")
    // Метод getBookings: возвращает нужное значение для текущего контекста.
    Call<List<Booking>> getBookings();
    
    @GET("/api/bookings/{id}")
    // Метод getBookingById: возвращает нужное значение для текущего контекста.
    Call<Booking> getBookingById(@Path("id") String id);
    
    @POST("/api/bookings")
    // Метод createBooking: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Booking> createBooking(@Body Map<String, Object> data);

    /** Массовое создание бронирований (из корзины). Одно уведомление администраторам вместо N. */
    @POST("/api/bookings/bulk")
    // Метод createBulkBookings: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<List<Booking>> createBulkBookings(@Body BulkBookingsRequest data);
    
    @PUT("/api/bookings/{id}")
    // Метод updateBooking: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Booking> updateBooking(@Path("id") String id, @Body Map<String, Object> data);
    
    @POST("/api/bookings/{id}/approve")
    // Метод approveBooking: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Booking> approveBooking(@Path("id") String id);
    
    @POST("/api/bookings/{id}/reject")
    // Метод rejectBooking: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Booking> rejectBooking(@Path("id") String id);

    @POST("/api/bookings/{id}/return")
    Call<Booking> confirmBookingReturn(@Path("id") String id);
    
    @DELETE("/api/bookings/{id}")
    // Метод deleteBooking: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Void> deleteBooking(@Path("id") String id);
    
    // Categories
    @GET("/api/categories")
    // Метод getCategories: возвращает нужное значение для текущего контекста.
    Call<List<Category>> getCategories();

    @GET("/api/categories")
    // Метод getCategoriesBySquad: возвращает нужное значение для текущего контекста.
    Call<List<Category>> getCategoriesBySquad(@Query("squad_id") String squadId);
    
    @GET("/api/categories/{id}")
    // Метод getCategoryById: возвращает нужное значение для текущего контекста.
    Call<Category> getCategoryById(@Path("id") String id);
    
    @POST("/api/categories")
    // Метод createCategory: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Category> createCategory(@Body Map<String, Object> data);
    
    @PUT("/api/categories/{id}")
    // Метод updateCategory: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Category> updateCategory(@Path("id") String id, @Body Map<String, Object> data);
    
    @DELETE("/api/categories/{id}")
    // Метод deleteCategory: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Void> deleteCategory(@Path("id") String id);
    
    // Users (Admin only)
    @GET("/api/users")
    // Метод getUsers: возвращает нужное значение для текущего контекста.
    Call<List<User>> getUsers();
    
    @GET("/api/users/{id}")
    // Метод getUserById: возвращает нужное значение для текущего контекста.
    Call<User> getUserById(@Path("id") String id);
    
    @POST("/api/users")
    // Метод createUser: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<User> createUser(@Body Map<String, Object> data);
    
    @PUT("/api/users/{id}")
    // Метод updateUser: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<User> updateUser(@Path("id") String id, @Body Map<String, Object> data);
    
    @DELETE("/api/users/{id}")
    // Метод deleteUser: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Void> deleteUser(@Path("id") String id);
    
    // Notifications
    @GET("/api/notifications")
    // Метод getNotifications: возвращает нужное значение для текущего контекста.
    Call<List<Map<String, Object>>> getNotifications();
    
    @GET("/api/notifications/unread-count")
    // Метод getUnreadCount: возвращает нужное значение для текущего контекста.
    Call<Map<String, Integer>> getUnreadCount();
    
    @POST("/api/notifications/{id}/read")
    // Метод markAsRead: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Void> markAsRead(@Path("id") String id);
    
    // Reports (Admin/Responsible only)
    @GET("/api/reports/equipment")
    // Метод getEquipmentReport: возвращает нужное значение для текущего контекста.
    Call<Map<String, Object>> getEquipmentReport();
    
    @GET("/api/reports/bookings")
    // Метод getBookingReport: возвращает нужное значение для текущего контекста.
    Call<Map<String, Object>> getBookingReport();
    
    // Logs (Admin/Responsible only)
    @GET("/api/logs")
    // Метод getLogs: возвращает нужное значение для текущего контекста.
    Call<List<Map<String, Object>>> getLogs();

    // Push notifications
    @POST("/api/push/register-token")
    // Метод registerPushToken: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Void> registerPushToken(@Body Map<String, String> data);
    
    // QR Code
    @GET("/api/qr/{id}")
    // Метод generateQRCode: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<ResponseBody> generateQRCode(@Path("id") String id);
    
    @GET("/api/qr/{id}/data")
    // Метод getQRCodeData: возвращает нужное значение для текущего контекста.
    Call<Map<String, Object>> getQRCodeData(@Path("id") String id);

    // Support (техподдержка)
    @GET("/api/support/requests")
    // Метод getSupportRequests: возвращает нужное значение для текущего контекста.
    Call<List<Map<String, Object>>> getSupportRequests();

    @POST("/api/support/requests")
    // Метод createSupportRequest: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Map<String, Object>> createSupportRequest(@Body Map<String, String> data);

    @POST("/api/support/requests/{id}/messages")
    // Метод addSupportMessage: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Map<String, Object>> addSupportMessage(@Path("id") String id, @Body Map<String, String> data);

    @PUT("/api/support/requests/{id}")
    // Метод updateSupportRequest: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Map<String, Object>> updateSupportRequest(@Path("id") String id, @Body Map<String, Object> data);

    @DELETE("/api/support/requests/{id}")
    // Метод deleteSupportRequest: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Void> deleteSupportRequest(@Path("id") String id);

    // Squads (сквады)
    @GET("/api/squads")
    // Метод getSquads: возвращает нужное значение для текущего контекста.
    Call<List<Map<String, Object>>> getSquads();

    @GET("/api/squads/{id}")
    // Метод getSquad: возвращает нужное значение для текущего контекста.
    Call<Map<String, Object>> getSquad(@Path("id") String id);

    @GET("/api/squads/{id}/equipment")
    // Метод getSquadEquipment: возвращает нужное значение для текущего контекста.
    Call<List<Equipment>> getSquadEquipment(@Path("id") String id);

    @POST("/api/squads")
    // Метод createSquad: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Map<String, Object>> createSquad(@Body Map<String, Object> data);

    @PUT("/api/squads/{id}")
    // Метод updateSquad: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Map<String, Object>> updateSquad(@Path("id") String id, @Body Map<String, Object> data);

    @DELETE("/api/squads/{id}")
    // Метод deleteSquad: выполняет основную бизнес- или UI-логику данного участка кода.
    Call<Void> deleteSquad(@Path("id") String id);
}
