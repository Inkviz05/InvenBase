import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { notificationsAPI } from '../api/notifications';
import { bookingsAPI } from '../api/bookings';
import { useAuth } from '../context/AuthContext';
import { format } from 'date-fns';

const Notifications = () => {
  const { isAdmin, isResponsible } = useAuth();
  const [notifications, setNotifications] = useState([]);
  const [bookingsData, setBookingsData] = useState({});
  const [loading, setLoading] = useState(true);
  const [processingIds, setProcessingIds] = useState(new Set());

  useEffect(() => {
    fetchNotifications();
  }, []);

  const fetchNotifications = async () => {
    try {
      const data = await notificationsAPI.getAll();
      setNotifications(data);

      // Загружаем данные о бронированиях для уведомлений
      const bookingIds = data
        .filter(n => n.booking_id)
        .map(n => n.booking_id);

      if (bookingIds.length > 0) {
        await fetchBookingsData(bookingIds);
      }
    } catch (error) {
      console.error('Failed to fetch notifications:', error);
    } finally {
      setLoading(false);
    }
  };

  const fetchBookingsData = async (bookingIds) => {
    try {
      const bookings = await bookingsAPI.getAll();
      const bookingsMap = {};

      bookingIds.forEach(id => {
        const booking = bookings.find(b => b.id === id);
        if (booking) {
          bookingsMap[id] = booking;
        }
      });

      setBookingsData(bookingsMap);
    } catch (error) {
      console.error('Failed to fetch bookings data:', error);
    }
  };

  const handleMarkAsRead = async (id) => {
    try {
      await notificationsAPI.markAsRead(id);
      setNotifications(notifications.map(n =>
        n.id === id ? { ...n, is_read: true } : n
      ));
    } catch (error) {
      console.error('Failed to mark as read:', error);
    }
  };

  const handleMarkAllAsRead = async () => {
    try {
      await notificationsAPI.markAllAsRead();
      setNotifications(notifications.map(n => ({ ...n, is_read: true })));
    } catch (error) {
      console.error('Failed to mark all as read:', error);
    }
  };

  const handleApprove = async (notificationId, bookingId) => {
    if (processingIds.has(notificationId)) return;

    setProcessingIds(prev => new Set([...prev, notificationId]));
    try {
      await bookingsAPI.approve(bookingId);
      await handleMarkAsRead(notificationId);
      // Обновляем данные о бронировании
      await fetchBookingsData([bookingId]);
      // Обновляем список уведомлений
      await fetchNotifications();
    } catch (error) {
      console.error('Failed to approve booking:', error);
      alert('Ошибка при одобрении бронирования');
    } finally {
      setProcessingIds(prev => {
        const newSet = new Set(prev);
        newSet.delete(notificationId);
        return newSet;
      });
    }
  };

  const handleReject = async (notificationId, bookingId) => {
    if (processingIds.has(notificationId)) return;

    if (!window.confirm('Вы уверены, что хотите отклонить это бронирование?')) {
      return;
    }

    setProcessingIds(prev => new Set([...prev, notificationId]));
    try {
      await bookingsAPI.reject(bookingId);
      await handleMarkAsRead(notificationId);
      // Обновляем данные о бронировании
      await fetchBookingsData([bookingId]);
      // Обновляем список уведомлений
      await fetchNotifications();
    } catch (error) {
      console.error('Failed to reject booking:', error);
      alert('Ошибка при отклонении бронирования');
    } finally {
      setProcessingIds(prev => {
        const newSet = new Set(prev);
        newSet.delete(notificationId);
        return newSet;
      });
    }
  };

  const unreadCount = notifications.filter(n => !n.is_read).length;
  const canManageBookings = isAdmin() || isResponsible();

  if (loading) {
    return <div className="loading"><div className="spinner"></div></div>;
  }

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h1>Уведомления</h1>
        {unreadCount > 0 && (
          <button onClick={handleMarkAllAsRead} className="btn btn-secondary">
            <span className="material-icons">done_all</span>
            Отметить все как прочитанные
          </button>
        )}
      </div>

      {notifications.length === 0 ? (
        <div className="card" style={{ textAlign: 'center', padding: '40px' }}>
          <span className="material-icons" style={{ fontSize: '64px', color: 'var(--text-secondary)', marginBottom: '16px' }}>
            notifications_none
          </span>
          <p style={{ color: 'var(--text-secondary)' }}>Нет уведомлений</p>
        </div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {notifications.map((notification) => {
            const booking = notification.booking_id ? bookingsData[notification.booking_id] : null;
            const isProcessing = processingIds.has(notification.id);
            const isBookingNotification = booking && canManageBookings && booking.status === 'pending';
            const isSupportNotification = notification.notification_type === 'support_new' || notification.notification_type === 'support_reply';

            return (
              <div
                key={notification.id}
                className="card notification-card"
                style={{
                  borderLeft: notification.is_read ? 'none' : '4px solid var(--primary-color)',
                  opacity: notification.is_read ? 0.7 : 1,
                  background: isBookingNotification && !notification.is_read
                    ? 'var(--notification-booking-bg)'
                    : isSupportNotification && !notification.is_read
                    ? 'var(--notification-support-bg)'
                    : 'var(--card-bg)'
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', gap: '16px' }}>
                  <div style={{ flex: 1 }}>
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '8px' }}>
                      <span className="material-icons" style={{
                        fontSize: '20px',
                        color: notification.is_read ? 'var(--text-secondary)' : 'var(--primary-color)'
                      }}>
                        {booking ? 'event' : isSupportNotification ? 'support_agent' : 'notifications'}
                      </span>
                      <h3 style={{ margin: 0, fontSize: '18px' }}>{notification.title}</h3>
                      {!notification.is_read && (
                        <span style={{
                          width: '8px',
                          height: '8px',
                          borderRadius: '50%',
                          background: 'var(--primary-color)'
                        }}></span>
                      )}
                    </div>
                    <p style={{ marginBottom: '12px', color: 'var(--text-secondary)' }}>
                      {notification.message}
                    </p>
                    {isSupportNotification && (
                      <Link
                        to="/support"
                        style={{
                          display: 'inline-flex',
                          alignItems: 'center',
                          gap: '6px',
                          fontSize: '14px',
                          color: 'var(--primary-color)',
                          textDecoration: 'none',
                          marginBottom: '12px',
                        }}
                      >
                        <span className="material-icons" style={{ fontSize: '18px' }}>open_in_new</span>
                        Перейти в Поддержку
                      </Link>
                    )}
                    {/* Детали бронирования */}
                    {booking && (
                      <div className="booking-details" style={{
                        background: 'var(--background)',
                        padding: '12px',
                        borderRadius: '8px',
                        marginBottom: '12px',
                        border: '1px solid var(--divider)'
                      }}>
                        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px', marginBottom: '8px' }}>
                          <div>
                            <strong style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>
                              Оборудование:
                            </strong>
                            <span style={{ fontSize: '14px', fontWeight: 500 }}>
                              {booking.equipment_name || booking.group_name || 'Не указано'}
                            </span>
                          </div>
                          <div>
                            <strong style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>
                              Пользователь:
                            </strong>
                            <span style={{ fontSize: '14px' }}>
                              {booking.username || booking.full_name || 'Неизвестно'}
                            </span>
                          </div>
                          <div>
                            <strong style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>
                              Количество:
                            </strong>
                            <span style={{ fontSize: '14px' }}>{booking.quantity}</span>
                          </div>
                        </div>
                        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px', marginBottom: '8px' }}>
                          <div>
                            <strong style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>
                              Начало:
                            </strong>
                            <span style={{ fontSize: '14px' }}>
                              {format(new Date(booking.start_date), 'dd.MM.yyyy HH:mm')}
                            </span>
                          </div>
                          <div>
                            <strong style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>
                              Конец:
                            </strong>
                            <span style={{ fontSize: '14px' }}>
                              {format(new Date(booking.end_date), 'dd.MM.yyyy HH:mm')}
                            </span>
                          </div>
                        </div>
                        {booking.purpose && (
                          <div>
                            <strong style={{ fontSize: '12px', color: 'var(--text-secondary)', display: 'block', marginBottom: '4px' }}>
                              Цель:
                            </strong>
                            <span style={{ fontSize: '14px' }}>{booking.purpose}</span>
                          </div>
                        )}
                        <div style={{ marginTop: '8px', paddingTop: '8px', borderTop: '1px solid var(--divider)' }}>
                          <span style={{
                            padding: '6px 12px',
                            borderRadius: '8px',
                            background: booking.status === 'pending' ? 'rgba(245, 158, 11, 0.2)' :
                                       booking.status === 'approved' ? 'rgba(16, 185, 129, 0.2)' : 'rgba(239, 68, 68, 0.2)',
                            color: booking.status === 'pending' ? 'var(--warning)' :
                                   booking.status === 'approved' ? 'var(--success)' : 'var(--error)',
                            fontSize: '13px',
                            fontWeight: 600,
                            border: `1px solid ${booking.status === 'pending' ? 'rgba(245, 158, 11, 0.3)' :
                            booking.status === 'approved' ? 'rgba(16, 185, 129, 0.3)' : 'rgba(239, 68, 68, 0.3)'}`
                          }}>
                            {booking.status === 'pending' ? 'Ожидает одобрения' :
                             booking.status === 'approved' ? 'Одобрено' :
                             booking.status === 'rejected' ? 'Отклонено' : booking.status}
                          </span>
                        </div>
                      </div>
                    )}

                    <div style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                      {format(new Date(notification.created_at), 'dd.MM.yyyy HH:mm')}
                    </div>
                  </div>

                  <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', alignItems: 'flex-end' }}>
                    {isBookingNotification && (
                      <>
                        <button
                          onClick={() => handleApprove(notification.id, booking.id)}
                          className="btn btn-primary"
                          disabled={isProcessing || notification.is_read}
                          style={{
                            fontSize: '12px',
                            padding: '8px 16px',
                            minWidth: '120px'
                          }}
                        >
                          {isProcessing ? (
                            <>
                              <span className="spinner" style={{ width: '12px', height: '12px', borderWidth: '2px' }}></span>
                              Обработка...
                            </>
                          ) : (
                            <>
                              <span className="material-icons" style={{ fontSize: '16px' }}>check</span>
                              Принять
                            </>
                          )}
                        </button>
                        <button
                          onClick={() => handleReject(notification.id, booking.id)}
                          className="btn btn-danger"
                          disabled={isProcessing || notification.is_read}
                          style={{
                            fontSize: '12px',
                            padding: '8px 16px',
                            minWidth: '120px'
                          }}
                        >
                          {isProcessing ? (
                            <>
                              <span className="spinner" style={{ width: '12px', height: '12px', borderWidth: '2px' }}></span>
                              Обработка...
                            </>
                          ) : (
                            <>
                              <span className="material-icons" style={{ fontSize: '16px' }}>close</span>
                              Отклонить
                            </>
                          )}
                        </button>
                      </>
                    )}
                    {!notification.is_read && !isBookingNotification && (
                      <button
                        onClick={() => handleMarkAsRead(notification.id)}
                        className="btn btn-secondary"
                        style={{ fontSize: '12px', padding: '6px 12px' }}
                      >
                        Отметить прочитанным
                      </button>
                    )}
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      )}
    </div>
  );
};

export default Notifications;

