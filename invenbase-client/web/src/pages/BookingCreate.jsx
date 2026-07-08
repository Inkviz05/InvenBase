import React, { useState, useEffect, useRef } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { bookingsAPI } from '../api/bookings';
import { equipmentAPI } from '../api/equipment';
import { useAuth } from '../context/AuthContext';

const BookingCreate = () => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const equipmentIdParam = searchParams.get('equipment_id');
  const { isAdmin, isResponsible } = useAuth();
  const isRegularUser = !isAdmin() && !isResponsible();

  const [formData, setFormData] = useState({
    equipment_id: equipmentIdParam || '',
    group_id: '',
    quantity: 1,
    start_date: '',
    end_date: '',
    purpose: '',
    permission_type: 'internal',
  });

  const [equipment, setEquipment] = useState([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const [scanning, setScanning] = useState(false);
  const [cameraPermission, setCameraPermission] = useState(null);
  const [showManualInput, setShowManualInput] = useState(false);
  const [manualQRCode, setManualQRCode] = useState('');
  const [selectedEquipmentData, setSelectedEquipmentData] = useState(null);
  const [loadingFromUrl, setLoadingFromUrl] = useState(!!(isRegularUser && equipmentIdParam));
  const scannerRef = useRef(null);

  useEffect(() => {
    if (!isRegularUser) {
      fetchEquipment();
    } else {
      checkCameraPermission();
    }
    return () => {
      if (scannerRef.current) {
        stopScanning();
      }
    };
  }, [isRegularUser]);

  // Для обычного пользователя: при переходе с equipment_id в URL сразу подставляем оборудование (без повторного сканирования)
  useEffect(() => {
    if (!isRegularUser || !equipmentIdParam) {
      setLoadingFromUrl(false);
      return;
    }
    let cancelled = false;
    (async () => {
      try {
        const data = await equipmentAPI.getById(equipmentIdParam);
        if (!cancelled) {
          setSelectedEquipmentData(data);
          setFormData(prev => ({ ...prev, equipment_id: equipmentIdParam }));
          setError('');
        }
      } catch (err) {
        if (!cancelled) setError('Оборудование не найдено');
      } finally {
        if (!cancelled) setLoadingFromUrl(false);
      }
    })();
    return () => { cancelled = true; };
  }, [equipmentIdParam, isRegularUser]);

  const checkCameraPermission = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video: true });
      stream.getTracks().forEach(track => track.stop());
      setCameraPermission(true);
    } catch (err) {
      setCameraPermission(false);
    }
  };

  const fetchEquipment = async () => {
    try {
      const data = await equipmentAPI.getAll();
      setEquipment(data);
    } catch (error) {
      console.error('Failed to fetch equipment:', error);
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
      const streams = await navigator.mediaDevices.enumerateDevices();
      // Останавливаем все активные видеопотоки
      if (navigator.mediaDevices && navigator.mediaDevices.getUserMedia) {
        const tracks = document.querySelectorAll('video').forEach(video => {
          if (video.srcObject) {
            const stream = video.srcObject;
            stream.getTracks().forEach(track => {
              track.stop();
              track.enabled = false;
            });
            video.srcObject = null;
          }
        });
      }
    } catch (err) {
      console.warn('Error releasing camera streams:', err);
    }
  };

  const startScanning = async () => {
    try {
      setError('');
      
      // Останавливаем предыдущий экземпляр, если он существует
      if (scannerRef.current) {
        await stopScanning();
      }
      
      // Принудительно освобождаем все потоки камеры
      await releaseAllCameraStreams();
      
      // Даём больше времени на освобождение камеры
      await new Promise(resolve => setTimeout(resolve, 500));
      
      setScanning(true);

      let Html5Qrcode;
      if (window.Html5Qrcode) {
        Html5Qrcode = window.Html5Qrcode;
      } else {
        await loadHtml5QrcodeLibrary();
        Html5Qrcode = window.Html5Qrcode;
      }

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
      console.error('Scanner error:', err);
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
      
      setError('Не удалось получить доступ к камере: ' + errorMessage);
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
      const equipmentData = await equipmentAPI.getByQR(qrCode);
      setSelectedEquipmentData(equipmentData);
      setFormData({ ...formData, equipment_id: equipmentData.id });
      setShowManualInput(false);
      setError('');
    } catch (err) {
      setError('Оборудование с таким QR-кодом не найдено');
    }
  };

  const handleManualSubmit = async () => {
    if (!manualQRCode.trim()) {
      setError('Введите QR-код');
      return;
    }
    await handleQRCode(manualQRCode.trim());
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      const bookingData = {
        equipment_id: formData.equipment_id || null,
        group_id: formData.group_id || null,
        quantity: parseInt(formData.quantity),
        start_date: new Date(formData.start_date).toISOString(),
        end_date: new Date(formData.end_date).toISOString(),
        purpose: formData.purpose || null,
        permission_type: formData.permission_type,
      };

      await bookingsAPI.create(bookingData);
      navigate('/bookings');
    } catch (err) {
      setError(err.response?.data?.error || 'Ошибка при создании бронирования');
    } finally {
      setLoading(false);
    }
  };

  const selectedEquipment = isRegularUser 
    ? selectedEquipmentData 
    : equipment.find(eq => eq.id === formData.equipment_id);

  // Для обычных пользователей: показываем QR-сканер или ручной ввод (или загрузку при переходе по ссылке с equipment_id)
  if (isRegularUser && !selectedEquipment) {
    if (loadingFromUrl) {
      return (
        <div className="booking-create-container">
          <h1 className="page-title">Создать бронирование</h1>
          <div className="card"><div className="loading"><div className="spinner"></div></div><p style={{ marginTop: '12px' }}>Загрузка...</p></div>
        </div>
      );
    }
    return (
      <div className="booking-create-container">
        <h1 className="page-title">Создать бронирование</h1>

        {error && <div className="error-message">{error}</div>}

        <div className="card">
          <div className="qr-section">
            <h2 style={{ marginBottom: '16px', fontSize: '18px', fontWeight: 500 }}>
              Отсканируйте QR-код оборудования
            </h2>
            <p style={{ marginBottom: '24px', color: 'var(--text-secondary)' }}>
              Используйте камеру для сканирования QR-кода или введите код вручную
            </p>

            {cameraPermission === false && (
              <div className="card" style={{ marginBottom: '24px', background: 'rgba(245, 158, 11, 0.1)', border: '1px solid rgba(245, 158, 11, 0.3)' }}>
                <p style={{ marginBottom: '12px' }}>Для работы сканера необходим доступ к камере устройства.</p>
                <button onClick={checkCameraPermission} className="btn btn-primary">
                  Проверить разрешения
                </button>
              </div>
            )}

            {!scanning && !showManualInput && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: '12px', alignItems: 'stretch' }}>
                <button 
                  onClick={startScanning} 
                  className="btn btn-primary" 
                  disabled={cameraPermission === false}
                  style={{ width: '100%' }}
                >
                  <span className="material-icons">qr_code_scanner</span>
                  Сканировать QR-код
                </button>
                <button 
                  onClick={() => setShowManualInput(true)} 
                  className="btn btn-secondary"
                  style={{ width: '100%' }}
                >
                  <span className="material-icons">keyboard</span>
                  Ввести QR-код вручную
                </button>
              </div>
            )}

            {scanning && (
              <div>
                <div id="qr-reader-booking" style={{ width: '100%', marginBottom: '16px' }}></div>
                <button onClick={stopScanning} className="btn btn-danger" style={{ width: '100%' }}>
                  <span className="material-icons">stop</span>
                  Остановить сканирование
                </button>
              </div>
            )}

            {showManualInput && !scanning && (
              <div>
                <label className="label">Введите QR-код</label>
                <input
                  type="text"
                  className="input"
                  value={manualQRCode}
                  onChange={(e) => setManualQRCode(e.target.value)}
                  placeholder="Введите QR-код оборудования"
                  autoFocus
                  aria-label="Manual QR code"
                />
                <div style={{ display: 'flex', gap: '12px', marginTop: '16px' }}>
                  <button 
                    onClick={handleManualSubmit} 
                    className="btn btn-primary"
                    style={{ flex: 1 }}
                  >
                    Найти оборудование
                  </button>
                  <button 
                    onClick={() => {
                      setShowManualInput(false);
                      setManualQRCode('');
                      setError('');
                    }} 
                    className="btn btn-secondary"
                  >
                    Отмена
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="booking-create-container">
      <h1 className="page-title">Создать бронирование</h1>

      {error && <div className="error-message">{error}</div>}

      <form onSubmit={handleSubmit} className="card">
        {!isRegularUser && (
          <>
            <label className="label">Оборудование *</label>
            <select
              className="input"
              value={formData.equipment_id}
              onChange={(e) => setFormData({ ...formData, equipment_id: e.target.value })}
              required
              aria-label="Equipment"
            >
              <option value="">Выберите оборудование</option>
              {equipment.map((eq) => (
                <option key={eq.id} value={eq.id}>
                  {eq.name} (Доступно: {eq.available_quantity})
                </option>
              ))}
            </select>
          </>
        )}

        {selectedEquipment && (
          <div className="equipment-info">
            <strong>Выбрано:</strong> {selectedEquipment.name}<br />
            <small>Доступно: {selectedEquipment.available_quantity} из {selectedEquipment.quantity}</small>
            {isRegularUser && (
              <button
                type="button"
                onClick={() => {
                  setSelectedEquipmentData(null);
                  setFormData({ ...formData, equipment_id: '' });
                }}
                className="btn btn-secondary"
                style={{ marginTop: '12px', width: '100%' }}
              >
                Выбрать другое оборудование
              </button>
            )}
          </div>
        )}

        <label className="label">Количество *</label>
        <input
          type="number"
          className="input"
          min="1"
          max={selectedEquipment?.available_quantity || 999}
          value={formData.quantity}
          onChange={(e) => setFormData({ ...formData, quantity: e.target.value })}
          required
          aria-label="Quantity"
        />

        <label className="label">Дата начала *</label>
        <input
          type="datetime-local"
          className="input"
          value={formData.start_date}
          onChange={(e) => setFormData({ ...formData, start_date: e.target.value })}
          required
          aria-label="Start date"
        />

        <label className="label">Дата окончания *</label>
        <input
          type="datetime-local"
          className="input"
          value={formData.end_date}
          onChange={(e) => setFormData({ ...formData, end_date: e.target.value })}
          required
          aria-label="End date"
        />

        <label className="label">Тип разрешения *</label>
        <select
          className="input"
          value={formData.permission_type}
          onChange={(e) => setFormData({ ...formData, permission_type: e.target.value })}
          required
          aria-label="Permission type"
        >
          <option value="internal">Внутреннее использование</option>
          <option value="external">Вынос за пределы учреждения</option>
        </select>

        <label className="label">Цель использования</label>
        <textarea
          className="input"
          rows="4"
          value={formData.purpose}
          onChange={(e) => setFormData({ ...formData, purpose: e.target.value })}
          placeholder="Опишите цель использования оборудования..."
          aria-label="Purpose"
        />

        <div className="form-actions">
          <button type="submit" className="btn btn-primary" disabled={loading || !selectedEquipment}>
            {loading ? 'Создание...' : 'Создать бронирование'}
          </button>
          <button
            type="button"
            className="btn btn-secondary"
            onClick={() => navigate('/bookings')}
          >
            Отмена
          </button>
        </div>
      </form>
    </div>
  );
};

export default BookingCreate;

