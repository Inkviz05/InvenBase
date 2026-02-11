import React, { useState, useEffect } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { useCart } from '../context/CartContext';
import { equipmentAPI } from '../api/equipment';
import { bookingsAPI } from '../api/bookings';
import { notificationsAPI } from '../api/notifications';
import { reportsAPI } from '../api/reports';

const Dashboard = () => {
  const { user, isAdmin, isResponsible } = useAuth();
  const { getCartItemsCount, cartItems } = useCart();
  const navigate = useNavigate();
  const [stats, setStats] = useState({
    totalEquipment: 0,
    availableEquipment: 0,
    bookedEquipment: 0,
    totalBookings: 0,
    pendingBookings: 0,
    approvedBookings: 0,
    expiredBookings: 0,
    unreadNotifications: 0,
  });
  const [equipmentReport, setEquipmentReport] = useState(null);
  const [bookingReport, setBookingReport] = useState(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        if (isAdmin() || isResponsible()) {
          // Для админов и ответственных - используем отчёты
          const [equipmentReportData, bookingReportData, notifications] = await Promise.all([
            reportsAPI.getEquipmentReport(),
            reportsAPI.getBookingReport(),
            notificationsAPI.getUnreadCount(),
          ]);

          setEquipmentReport(equipmentReportData);
          setBookingReport(bookingReportData);

          setStats({
            totalEquipment: equipmentReportData.total_equipment || 0,
            availableEquipment: equipmentReportData.available_equipment || 0,
            bookedEquipment: equipmentReportData.booked_equipment || 0,
            totalBookings: (bookingReportData.approved || 0) + (bookingReportData.pending || 0),
            pendingBookings: bookingReportData.pending || 0,
            approvedBookings: bookingReportData.approved || 0,
            expiredBookings: bookingReportData.expired || 0,
            unreadNotifications: notifications.count || 0,
          });
        } else {
          // Для обычных пользователей - упрощённая статистика
          const [equipment, bookings, notifications] = await Promise.all([
            equipmentAPI.getAll(),
            bookingsAPI.getAll(),
            notificationsAPI.getUnreadCount(),
          ]);

          const myBookings = bookings.filter(b => b.user_id === user?.id);
          const totalEquipment = equipment.length;
          const availableEquipment = equipment.reduce((sum, eq) => sum + eq.available_quantity, 0);
          const totalBookings = myBookings.length;
          const pendingBookings = myBookings.filter(b => b.status === 'pending').length;
          const approvedBookings = myBookings.filter(b => b.status === 'approved').length;

          setStats({
            totalEquipment,
            availableEquipment,
            bookedEquipment: 0,
            totalBookings,
            pendingBookings,
            approvedBookings,
            expiredBookings: 0,
            unreadNotifications: notifications.count || 0,
          });
        }
      } catch (error) {
        console.error('Failed to fetch stats:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchStats();
  }, [isAdmin, isResponsible, user]);

  if (loading) {
    return <div className="loading"><div className="spinner"></div></div>;
  }

  return (
    <div>
      <h1 style={{ marginBottom: '24px' }}>Добро пожаловать, {user?.full_name || user?.username}!</h1>

      <div style={{
        display: 'grid',
        gridTemplateColumns: 'repeat(auto-fit, minmax(250px, 1fr))',
        gap: '16px',
        marginBottom: '32px'
      }}>
        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
            <div style={{ 
              width: '60px', 
              height: '60px', 
              borderRadius: '12px', 
              background: 'linear-gradient(135deg, var(--primary-color) 0%, var(--secondary-color) 100%)', 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              color: 'white',
              boxShadow: '0 4px 12px rgba(168, 85, 247, 0.4)'
            }}>
              <span className="material-icons" style={{ fontSize: '32px' }}>inventory</span>
            </div>
            <div>
              <div style={{ fontSize: '12px', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>Всего оборудования</div>
              <div style={{ fontSize: '32px', fontWeight: 'bold', color: 'var(--text-primary)' }}>{stats.totalEquipment}</div>
            </div>
          </div>
        </div>

        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
            <div style={{ 
              width: '60px', 
              height: '60px', 
              borderRadius: '12px', 
              background: 'linear-gradient(135deg, var(--success) 0%, #059669 100%)', 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              color: 'white',
              boxShadow: '0 4px 12px rgba(16, 185, 129, 0.4)'
            }}>
              <span className="material-icons" style={{ fontSize: '32px' }}>check_circle</span>
            </div>
            <div>
              <div style={{ fontSize: '12px', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>Доступно</div>
              <div style={{ fontSize: '32px', fontWeight: 'bold', color: 'var(--text-primary)' }}>{stats.availableEquipment}</div>
            </div>
          </div>
        </div>

        <div className="card">
          <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
            <div style={{ 
              width: '60px', 
              height: '60px', 
              borderRadius: '12px', 
              background: 'linear-gradient(135deg, var(--accent-cyan) 0%, #06B6D4 100%)', 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              color: 'white',
              boxShadow: '0 4px 12px rgba(34, 211, 238, 0.4)'
            }}>
              <span className="material-icons" style={{ fontSize: '32px' }}>event</span>
            </div>
            <div>
              <div style={{ fontSize: '12px', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>Бронирований</div>
              <div style={{ fontSize: '32px', fontWeight: 'bold', color: 'var(--text-primary)' }}>{stats.totalBookings}</div>
            </div>
          </div>
        </div>

        {(isAdmin() || isResponsible()) && (
          <>
            <div className="card">
              <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                <div style={{ 
                  width: '60px', 
                  height: '60px', 
                  borderRadius: '12px', 
                  background: 'linear-gradient(135deg, var(--warning) 0%, #D97706 100%)', 
                  display: 'flex', 
                  alignItems: 'center', 
                  justifyContent: 'center',
                  color: 'white',
                  boxShadow: '0 4px 12px rgba(245, 158, 11, 0.4)'
                }}>
                  <span className="material-icons" style={{ fontSize: '32px' }}>pending</span>
                </div>
                <div>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>Ожидают одобрения</div>
                  <div style={{ fontSize: '32px', fontWeight: 'bold', color: 'var(--text-primary)' }}>{stats.pendingBookings}</div>
                </div>
              </div>
            </div>
            <div className="card">
              <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                <div style={{ 
                  width: '60px', 
                  height: '60px', 
                  borderRadius: '12px', 
                  background: 'linear-gradient(135deg, var(--primary-color) 0%, var(--secondary-color) 100%)', 
                  display: 'flex', 
                  alignItems: 'center', 
                  justifyContent: 'center',
                  color: 'white',
                  boxShadow: '0 4px 12px rgba(168, 85, 247, 0.4)'
                }}>
                  <span className="material-icons" style={{ fontSize: '32px' }}>inventory</span>
                </div>
                <div>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.5px', marginBottom: '4px' }}>Забронировано</div>
                  <div style={{ fontSize: '32px', fontWeight: 'bold', color: 'var(--text-primary)' }}>{stats.bookedEquipment}</div>
                </div>
              </div>
            </div>
          </>
        )}
      </div>

      {/* Расширенная статистика для админов и ответственных */}
      {(isAdmin() || isResponsible()) && equipmentReport && (
        <div style={{ marginTop: '32px' }}>
          {equipmentReport.by_category && equipmentReport.by_category.length > 0 && (
            <div className="card" style={{ marginBottom: '24px' }}>
              <h2 style={{ marginBottom: '20px' }}>📊 Статистика по категориям</h2>
              <div style={{ display: 'grid', gap: '20px' }}>
                {equipmentReport.by_category.map((cat, index) => (
                  <div key={index}>
                    <div style={{ 
                      fontSize: '14px', 
                      color: 'var(--text-primary)', 
                      marginBottom: '12px', 
                      fontWeight: 600, 
                      textTransform: 'uppercase', 
                      letterSpacing: '0.5px' 
                    }}>
                      {cat.category_name || 'Без категории'}
                    </div>
                    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '20px' }}>
                      <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--primary-color)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                        <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Всего</div>
                        <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--primary-color)' }}>
                          {cat.total || 0}
                        </div>
                      </div>
                      <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--success)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                        <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Доступно</div>
                        <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--success)' }}>
                          {cat.available || 0}
                        </div>
                      </div>
                      <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--warning)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                        <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Забронировано</div>
                        <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--warning)' }}>
                          {cat.booked || 0}
                        </div>
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          )}

          {bookingReport && (
            <div className="card">
              <h2 style={{ marginBottom: '20px' }}>📅 Статистика бронирований</h2>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '20px' }}>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--warning)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Ожидают одобрения</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--warning)' }}>
                    {bookingReport.pending || 0}
                  </div>
                </div>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--success)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Одобрены</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--success)' }}>
                    {bookingReport.approved || 0}
                  </div>
                </div>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--error)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Истекли</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--error)' }}>
                    {bookingReport.expired || 0}
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '16px' }}>
      <div className="card">
      <h2 style={{ marginBottom: '16px' }}>Быстрые действия</h2>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
      {cartItems.length > 0 && (
      <button 
      onClick={() => navigate('/bookings/bulk')} 
      className="btn btn-primary"
      style={{ 
      position: 'relative'
      }}
      >
      <span className="material-icons">shopping_cart</span>
      Забронировать из корзины
      <span style={{
      position: 'absolute',
      top: '-8px',
      right: '-8px',
      background: 'var(--error)',
      borderRadius: '50%',
      width: '24px',
      height: '24px',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      fontSize: '12px',
      fontWeight: 'bold'
      }}>
      {getCartItemsCount()}
      </span>
      </button>
      )}

      {/* Администратор / ответственный */}
      {(isAdmin() || isResponsible()) && (
        <>
          <Link
            to="/equipment"
            className="btn btn-primary"
            style={{ textDecoration: 'none', justifyContent: 'center' }}
          >
            <span className="material-icons">inventory</span>
            Выбрать оборудование
          </Link>
          <Link
            to="/equipment/create"
            className="btn btn-primary"
            style={{ textDecoration: 'none', justifyContent: 'center' }}
          >
            <span className="material-icons">add</span>
            Добавить оборудование
          </Link>
          <Link
            to="/scanner"
            className="btn btn-secondary"
            style={{ textDecoration: 'none', justifyContent: 'center' }}
          >
            <span className="material-icons">qr_code_scanner</span>
            Сканировать QR-код
          </Link>
        </>
      )}

      {/* Обычный пользователь */}
      {!isAdmin() && !isResponsible() && (
        <>
          <Link
            to="/scanner"
            className="btn btn-primary"
            style={{ textDecoration: 'none', justifyContent: 'center' }}
          >
            <span className="material-icons">qr_code_scanner</span>
            Сканировать QR-код
          </Link>
          <Link
            to="/scanner?mode=manual"
            className="btn btn-secondary"
            style={{ textDecoration: 'none', justifyContent: 'center' }}
          >
            <span className="material-icons">keyboard</span>
            Ввести код вручную
          </Link>
        </>
      )}
      </div>
      </div>

        <div className="card">
          <h2 style={{ marginBottom: '16px' }}>Уведомления</h2>
          {stats.unreadNotifications > 0 ? (
            <div>
              <p>У вас {stats.unreadNotifications} непрочитанных уведомлений</p>
              <Link to="/notifications" className="btn btn-primary" style={{ textDecoration: 'none', marginTop: '12px' }}>
                Просмотреть
              </Link>
            </div>
          ) : (
            <p style={{ color: 'var(--text-secondary)' }}>Нет новых уведомлений</p>
          )}
        </div>
      </div>
    </div>
  );
};

export default Dashboard;

