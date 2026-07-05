import React, { useState, useEffect, useRef } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { bookingsAPI } from '../api/bookings';
import { equipmentAPI } from '../api/equipment';
import { useAuth } from '../context/AuthContext';
import { format } from 'date-fns';

const BookingList = () => {
  const { isAdmin, isResponsible, user } = useAuth();
  const navigate = useNavigate();
  const [bookings, setBookings] = useState([]);
  const [loading, setLoading] = useState(true);
  const [filter, setFilter] = useState('all');
  const [showQRScanner, setShowQRScanner] = useState(false);
  const [scanning, setScanning] = useState(false);
  const [manualCode, setManualCode] = useState('');
  const [showManualInput, setShowManualInput] = useState(false);
  const scannerRef = useRef(null);
  
  const isRegularUser = !isAdmin() && !isResponsible();
  const canCancelBooking = (booking) => {
    if (!booking) return false;
    if (isAdmin() || isResponsible()) {
      return ['pending', 'approved', 'awaiting_return'].includes(booking.status);
    }
    return booking.status === 'pending';
  };

  useEffect(() => {
    fetchBookings();
    return () => {
      if (scannerRef.current) {
        stopScanning();
      }
    };
  }, []);

  const fetchBookings = async () => {
    try {
      const data = await bookingsAPI.getAll();
      setBookings(data);
    } catch (error) {
      console.error('Failed to fetch bookings:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleApprove = async (id) => {
    try {
      await bookingsAPI.approve(id);
      fetchBookings();
    } catch (error) {
      alert('Ошибка при одобрении бронирования');
    }
  };

  const handleReject = async (id) => {
    if (!window.confirm('Вы уверены, что хотите отклонить это бронирование?')) {
      return;
    }
    try {
      await bookingsAPI.reject(id);
      fetchBookings();
    } catch (error) {
      alert('Ошибка при отклонении бронирования');
    }
  };

  const handleConfirmReturn = async (id) => {
    if (!window.confirm('Подтвердить фактический возврат оборудования?')) {
      return;
    }
    try {
      await bookingsAPI.confirmReturn(id);
      fetchBookings();
    } catch (error) {
      alert('Ошибка при подтверждении возврата оборудования');
    }
  };

  const handleCancel = async (id) => {
    if (!window.confirm('\u041e\u0442\u043c\u0435\u043d\u0438\u0442\u044c \u044d\u0442\u043e \u0431\u0440\u043e\u043d\u0438\u0440\u043e\u0432\u0430\u043d\u0438\u0435? \u0418\u0441\u0442\u043e\u0440\u0438\u044f \u0431\u0440\u043e\u043d\u0438\u0440\u043e\u0432\u0430\u043d\u0438\u044f \u0441\u043e\u0445\u0440\u0430\u043d\u0438\u0442\u0441\u044f.')) {
      return;
    }
    try {
      await bookingsAPI.cancel(id);
      fetchBookings();
    } catch (error) {
      alert('\u041e\u0448\u0438\u0431\u043a\u0430 \u043f\u0440\u0438 \u043e\u0442\u043c\u0435\u043d\u0435 \u0431\u0440\u043e\u043d\u0438\u0440\u043e\u0432\u0430\u043d\u0438\u044f');
    }
  };

  const getStatusColor = (status) => {
    switch (status) {
      case 'approved': return 'var(--success)';
      case 'rejected': return 'var(--error)';
      case 'pending': return 'var(--warning)';
      case 'awaiting_return': return 'var(--warning)';
      case 'returned': return 'var(--success)';
      case 'expired': return 'var(--text-secondary)';
      case 'cancelled': return 'var(--text-secondary)';
      default: return 'var(--text-secondary)';
    }
  };

  const getStatusText = (status) => {
    switch (status) {
      case 'approved': return 'Одобрено';
      case 'rejected': return 'Отклонено';
      case 'pending': return 'Ожидает';
      case 'awaiting_return': return 'Ожидает возврата';
      case 'returned': return 'Возвращено';
      case 'expired': return 'Истекло';
      case 'cancelled': return 'Отменено';
      default: return status;
    }
  };

  const loadHtml5QrcodeLibrary = () => {
    return new Promise((resolve, reject) => {
      if (window.Html5Qrcode) {
        resolve();
        return;
      }
      const script = document.createElement('script');
      script.src = 'https://unpkg.com/html5-qrcode@2.3.8/html5-qrcode.min.js';
      script.onload = () => resolve();
      script.onerror = () => reject(new Error('Failed to load html5-qrcode library'));
      document.head.appendChild(script);
    });
  };

  // Функция для принудительного освобождения всех потоков камеры
  const releaseAllCameraStreams = async () => {
    try {
      // Останавливаем все активные видеопотоки
      document.querySelectorAll('video').forEach(video => {
        if (video.srcObject) {
          const stream = video.srcObject;
          stream.getTracks().forEach(track => {
            track.stop();
            track.enabled = false;
          });
          video.srcObject = null;
        }
      });
    } catch (err) {
      console.warn('Error releasing camera streams:', err);
    }
  };

  const startScanning = async () => {
    try {
      // Останавливаем предыдущий экземпляр, если он существует
      if (scannerRef.current) {
        await stopScanning();
      }
      
      // Принудительно освобождаем все потоки камеры
      await releaseAllCameraStreams();
      
      // Даём больше времени на освобождение камеры
      await new Promise(resolve => setTimeout(resolve, 500));
      
      setScanning(true);
      await loadHtml5QrcodeLibrary();
      const Html5Qrcode = window.Html5Qrcode;
      
      // Ждём появления элемента в DOM
      let container = document.getElementById("qr-reader-booking");
      let attempts = 0;
      while (!container && attempts < 10) {
        await new Promise(resolve => setTimeout(resolve, 100));
        container = document.getElementById("qr-reader-booking");
        attempts++;
      }

      if (!container) {
        throw new Error('Элемент для сканера не найден. Попробуйте обновить страницу.');
      }
      
      // Очищаем контейнер перед созданием нового экземпляра
      container.innerHTML = '';
      
      const html5QrCode = new Html5Qrcode("qr-reader-booking");
      scannerRef.current = html5QrCode;

      await html5QrCode.start(
        { facingMode: "environment" },
        {
          fps: 10,
          qrbox: { width: 250, height: 250 }
        },
        (decodedText) => {
          handleQRCode(decodedText);
          stopScanning();
        },
        (errorMessage) => {
          // Игнорируем ошибки сканирования
        }
      );
    } catch (err) {
      console.error('Error starting scanner:', err);
      // Более детальная обработка ошибок
      let errorMessage = 'Неизвестная ошибка';
      if (err) {
        if (err.message) {
          errorMessage = err.message;
        } else if (err.name) {
          errorMessage = err.name;
        } else if (typeof err === 'string') {
          errorMessage = err;
        } else if (err.toString && err.toString() !== '[object Object]') {
          errorMessage = err.toString();
        }
      }
      alert('Не удалось получить доступ к камере: ' + errorMessage);
      setScanning(false);
      scannerRef.current = null;
      
      // Освобождаем камеру при ошибке
      await releaseAllCameraStreams();
    }
  };

  const stopScanning = async () => {
    if (scannerRef.current) {
      try {
        // Проверяем, запущен ли сканер, перед остановкой
        const isScanning = scannerRef.current.getState && scannerRef.current.getState() === 2;
        if (isScanning) {
          await scannerRef.current.stop();
        }
        await scannerRef.current.clear();
      } catch (err) {
        // Игнорируем ошибки при остановке (возможно, уже остановлен)
        console.warn('Error stopping scanner:', err);
      }
      scannerRef.current = null;
    }
    
    // Очищаем контейнер
    const container = document.getElementById("qr-reader-booking");
    if (container) {
      container.innerHTML = '';
    }
    
    // Освобождаем камеру
    await releaseAllCameraStreams();
    
    setScanning(false);
  };

  const handleQRCode = async (qrCode) => {
    try {
      const equipment = await equipmentAPI.getByQR(qrCode);
      // Переходим на создание бронирования с выбранным оборудованием
      navigate(`/bookings/create?equipment_id=${equipment.id}`);
      setShowQRScanner(false);
    } catch (err) {
      alert('Оборудование с таким QR-кодом не найдено');
    }
  };

  const handleManualSubmit = async (e) => {
    e.preventDefault();
    if (!manualCode.trim()) {
      alert('Введите код оборудования');
      return;
    }
    try {
      await handleQRCode(manualCode.trim());
      setManualCode('');
      setShowManualInput(false);
      setShowQRScanner(false);
    } catch (err) {
      // Ошибка уже обработана в handleQRCode
    }
  };

  const handleCreateBookingClick = () => {
    if (isRegularUser) {
      setShowQRScanner(true);
    } else {
      navigate('/bookings/create');
    }
  };

  const filteredBookings = filter === 'all' 
    ? bookings 
    : bookings.filter(b => b.status === filter);

  if (loading) {
    return <div className="loading"><div className="spinner"></div></div>;
  }

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h1>Бронирования</h1>
        {!isRegularUser ? (
          <Link to="/bookings/create" className="btn btn-primary" style={{ textDecoration: 'none' }}>
            <span className="material-icons">add</span>
            Создать бронирование
          </Link>
        ) : (
          <button onClick={handleCreateBookingClick} className="btn btn-primary">
            <span className="material-icons">add</span>
            Создать бронирование
          </button>
        )}
      </div>

      {/* Модальное окно для QR-сканера и ручного ввода (только для обычных пользователей) */}
      {isRegularUser && showQRScanner && (
        <div style={{
          position: 'fixed',
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          background: 'var(--modal-overlay-bg)',
          zIndex: 2000,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          padding: '20px'
        }} onClick={() => {
          setShowQRScanner(false);
          stopScanning();
        }}>
          <div className="card" style={{
            maxWidth: '500px',
            width: '100%',
            maxHeight: '90vh',
            overflow: 'auto',
            boxShadow: 'var(--modal-shadow)',
            border: '1px solid var(--dropdown-border)',
          }} onClick={(e) => e.stopPropagation()}>
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
              <h2 style={{ margin: 0 }}>Выбор оборудования</h2>
              <button
                onClick={() => {
                  setShowQRScanner(false);
                  stopScanning();
                }}
                style={{
                  background: 'none',
                  border: 'none',
                  cursor: 'pointer',
                  fontSize: '24px',
                  color: 'var(--text-secondary)'
                }}
              >
                ×
              </button>
            </div>

            <div style={{ marginBottom: '24px' }}>
              <h3 style={{ marginBottom: '16px', fontSize: '18px' }}>Сканирование QR-кода</h3>
              {!scanning ? (
                <div style={{ textAlign: 'center', padding: '20px' }}>
                  <span className="material-icons" style={{ fontSize: '64px', color: 'var(--primary-color)', marginBottom: '16px', display: 'block' }}>
                    qr_code_scanner
                  </span>
                  <button onClick={startScanning} className="btn btn-primary" style={{ width: '100%', maxWidth: '300px' }}>
                    <span className="material-icons">camera_alt</span>
                    Начать сканирование
                  </button>
                </div>
              ) : (
                <div>
                  <div id="qr-reader-booking" style={{ width: '100%', marginBottom: '16px' }}></div>
                  <button onClick={stopScanning} className="btn btn-danger" style={{ width: '100%' }}>
                    <span className="material-icons">stop</span>
                    Остановить сканирование
                  </button>
                </div>
              )}
            </div>

            <div style={{ borderTop: '1px solid var(--divider)', paddingTop: '24px' }}>
              <h3 style={{ marginBottom: '16px', fontSize: '18px' }}>Ввод кода вручную</h3>
              {!showManualInput ? (
                <button 
                  onClick={() => setShowManualInput(true)} 
                  className="btn btn-secondary" 
                  style={{ width: '100%', maxWidth: '300px' }}
                >
                  <span className="material-icons">keyboard</span>
                  Ввести код вручную
                </button>
              ) : (
                <form onSubmit={handleManualSubmit}>
                  <input
                    type="text"
                    className="input"
                    placeholder="Введите код оборудования"
                    value={manualCode}
                    onChange={(e) => setManualCode(e.target.value)}
                    autoFocus
                    style={{ marginBottom: '16px' }}
                  />
                  <div style={{ display: 'flex', gap: '12px' }}>
                    <button type="submit" className="btn btn-primary" style={{ flex: 1 }}>
                      <span className="material-icons">search</span>
                      Найти
                    </button>
                    <button 
                      type="button" 
                      onClick={() => {
                        setShowManualInput(false);
                        setManualCode('');
                      }} 
                      className="btn btn-secondary"
                    >
                      Отмена
                    </button>
                  </div>
                </form>
              )}
            </div>
          </div>
        </div>
      )}

      <div style={{ display: 'flex', gap: '8px', marginBottom: '24px', flexWrap: 'wrap' }}>
        <button
          className={`btn ${filter === 'all' ? 'btn-primary' : 'btn-secondary'}`}
          onClick={() => setFilter('all')}
        >
          Все
        </button>
        <button
          className={`btn ${filter === 'pending' ? 'btn-primary' : 'btn-secondary'}`}
          onClick={() => setFilter('pending')}
        >
          Ожидают
        </button>
        <button
          className={`btn ${filter === 'approved' ? 'btn-primary' : 'btn-secondary'}`}
          onClick={() => setFilter('approved')}
        >
          Одобрены
        </button>
        <button
          className={`btn ${filter === 'awaiting_return' ? 'btn-primary' : 'btn-secondary'}`}
          onClick={() => setFilter('awaiting_return')}
        >
          Ожидают возврата
        </button>
        <button
          className={`btn ${filter === 'rejected' ? 'btn-primary' : 'btn-secondary'}`}
          onClick={() => setFilter('rejected')}
        >
          Отклонены
        </button>
      </div>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '16px' }}>
        {filteredBookings.map((booking) => (
          <div key={booking.id} className="card">
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '12px' }}>
              <div>
                <h3 style={{ margin: 0, marginBottom: '8px' }}>
                  {booking.equipment_name || booking.group_name || 'Не указано'}
                </h3>
                <div style={{ fontSize: '14px', color: 'var(--text-secondary)' }}>
                  Пользователь: {booking.username || 'Неизвестно'}
                </div>
              </div>
              <span style={{
                padding: '6px 12px',
                borderRadius: '8px',
                background: getStatusColor(booking.status) === 'var(--success)' 
                  ? 'rgba(16, 185, 129, 0.2)' 
                  : getStatusColor(booking.status) === 'var(--error)'
                  ? 'rgba(239, 68, 68, 0.2)'
                  : getStatusColor(booking.status) === 'var(--warning)'
                  ? 'rgba(245, 158, 11, 0.2)'
                  : 'var(--inline-neutral-bg)',
                color: getStatusColor(booking.status),
                fontSize: '13px',
                fontWeight: 600,
                border: `1px solid ${getStatusColor(booking.status)}40`
              }}>
                {getStatusText(booking.status)}
              </span>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '12px', marginBottom: '12px' }}>
              <div>
                <span className="material-icons" style={{ fontSize: '18px', verticalAlign: 'middle' }}>event</span>
                {' '}Начало: {format(new Date(booking.start_date), 'dd.MM.yyyy HH:mm')}
              </div>
              <div>
                <span className="material-icons" style={{ fontSize: '18px', verticalAlign: 'middle' }}>event</span>
                {' '}Конец: {format(new Date(booking.end_date), 'dd.MM.yyyy HH:mm')}
              </div>
              <div>
                <span className="material-icons" style={{ fontSize: '18px', verticalAlign: 'middle' }}>inventory</span>
                {' '}Количество: {booking.quantity}
              </div>
            </div>

            {booking.purpose && (
              <div style={{ marginBottom: '12px', padding: '12px', background: 'var(--inline-panel-bg)', borderRadius: '8px', border: '1px solid var(--divider)' }}>
                <strong style={{ color: 'var(--text-primary)' }}>Цель:</strong> <span style={{ color: 'var(--text-secondary)' }}>{booking.purpose}</span>
              </div>
            )}

            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
              {(isAdmin() || isResponsible()) && booking.status === 'pending' && (
                <>
                  <button
                    onClick={() => handleApprove(booking.id)}
                    className="btn btn-primary"
                    style={{ fontSize: '12px', padding: '6px 12px' }}
                  >
                    Одобрить
                  </button>
                  <button
                    onClick={() => handleReject(booking.id)}
                    className="btn btn-danger"
                    style={{ fontSize: '12px', padding: '6px 12px' }}
                  >
                    Отклонить
                  </button>
                </>
              )}
              {(isAdmin() || isResponsible()) && booking.status === 'awaiting_return' && (
                <button
                  onClick={() => handleConfirmReturn(booking.id)}
                  className="btn btn-primary"
                  style={{ fontSize: '12px', padding: '6px 12px' }}
                >
                  Подтвердить возврат
                </button>
              )}
              {canCancelBooking(booking) && (
                <button
                  onClick={() => handleCancel(booking.id)}
                  className="btn btn-secondary"
                  style={{ fontSize: '12px', padding: '6px 12px' }}
                >
                  {'\u041e\u0442\u043c\u0435\u043d\u0438\u0442\u044c'}
                </button>
              )}
            </div>
          </div>
        ))}
      </div>

      {filteredBookings.length === 0 && (
        <div className="card" style={{ textAlign: 'center', padding: '40px' }}>
          <span className="material-icons" style={{ fontSize: '64px', color: 'var(--text-secondary)', marginBottom: '16px' }}>event_busy</span>
          <p style={{ color: 'var(--text-secondary)' }}>Нет бронирований</p>
        </div>
      )}
    </div>
  );
};

export default BookingList;

