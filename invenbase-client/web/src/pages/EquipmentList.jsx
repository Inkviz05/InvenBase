import React, { useState, useEffect, useRef } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { equipmentAPI } from '../api/equipment';
import { squadsAPI } from '../api/squads';
import { useAuth } from '../context/AuthContext';
import { useCart } from '../context/CartContext';

const EquipmentList = () => {
  const { isAdmin, isResponsible, user } = useAuth();
  const { addToCart, isInCart } = useCart();
  const navigate = useNavigate();
  const [equipment, setEquipment] = useState([]);
  const [squads, setSquads] = useState([]);
  const [squadFilter, setSquadFilter] = useState('');
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState('');
  const [scanning, setScanning] = useState(false);
  const [manualCode, setManualCode] = useState('');
  const [showManualInput, setShowManualInput] = useState(false);
  const scannerRef = useRef(null);

  const isRegularUser = !isAdmin() && !isResponsible();

  useEffect(() => {
    if (!isRegularUser) {
      fetchSquads();
    }
  }, [isRegularUser]);

  useEffect(() => {
    if (!isRegularUser) {
      fetchEquipment();
    } else {
      setLoading(false);
    }
    return () => {
      if (scannerRef.current) {
        stopScanning();
      }
    };
  }, [isRegularUser, squadFilter]);

  const fetchSquads = async () => {
    try {
      const data = await squadsAPI.getAll();
      setSquads(data);
    } catch (error) {
      console.error('Failed to fetch squads:', error);
    }
  };

  const fetchEquipment = async () => {
    setLoading(true);
    try {
      const params = squadFilter ? { squad_id: squadFilter } : {};
      const data = await equipmentAPI.getAll(params);
      setEquipment(data);
    } catch (error) {
      console.error('Failed to fetch equipment:', error);
    } finally {
      setLoading(false);
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
      let container = document.getElementById("qr-reader");
      let attempts = 0;
      while (!container && attempts < 10) {
        await new Promise(resolve => setTimeout(resolve, 100));
        container = document.getElementById("qr-reader");
        attempts++;
      }

      if (!container) {
        throw new Error('Элемент для сканера не найден. Попробуйте обновить страницу.');
      }
      
      // Очищаем контейнер перед созданием нового экземпляра
      container.innerHTML = '';
      
      const html5QrCode = new Html5Qrcode("qr-reader");
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
    const container = document.getElementById("qr-reader");
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
      navigate(`/equipment/${equipment.id}`);
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
    } catch (err) {
      // Ошибка уже обработана в handleQRCode
    }
  };

  const filteredEquipment = equipment.filter(eq =>
    eq.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
    eq.description?.toLowerCase().includes(searchTerm.toLowerCase())
  );

  if (loading && !isRegularUser) {
    return <div className="loading"><div className="spinner"></div></div>;
  }

  // Для обычных пользователей показываем интерфейс выбора через QR или код
  if (isRegularUser) {
    return (
      <div>
        <h1 style={{ marginBottom: '24px' }}>Выбор оборудования</h1>
        
        <div className="card" style={{ marginBottom: '24px' }}>
          <h2 style={{ marginBottom: '16px', fontSize: '18px' }}>Сканирование QR-кода</h2>
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
              <div id="qr-reader" style={{ width: '100%', marginBottom: '16px' }}></div>
              <button onClick={stopScanning} className="btn btn-danger" style={{ width: '100%' }}>
                <span className="material-icons">stop</span>
                Остановить сканирование
              </button>
            </div>
          )}
        </div>

        <div className="card">
          <h2 style={{ marginBottom: '16px', fontSize: '18px' }}>Ввод кода вручную</h2>
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
    );
  }

  // Для админов и ответственных - полный список
  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h1>Оборудование</h1>
        <Link to="/equipment/create" className="btn btn-primary" style={{ textDecoration: 'none' }}>
          <span className="material-icons">add</span>
          Добавить
        </Link>
      </div>

      <div style={{ display: 'flex', flexWrap: 'wrap', gap: '12px', marginBottom: '24px', alignItems: 'center' }}>
        <input
          type="text"
          className="input"
          placeholder="Поиск оборудования..."
          value={searchTerm}
          onChange={(e) => setSearchTerm(e.target.value)}
          style={{ flex: '1', minWidth: '200px' }}
        />
        <select
          className="input"
          value={squadFilter}
          onChange={(e) => setSquadFilter(e.target.value)}
          style={{ minWidth: '200px' }}
        >
          <option value="">Все сквады</option>
          {squads.map((s) => (
            <option key={s.id} value={s.id}>{s.name}</option>
          ))}
        </select>
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(300px, 1fr))', gap: '16px' }}>
        {filteredEquipment.map((eq) => (
          <div key={eq.id} className="card" style={{ transition: 'transform 0.2s', display: 'flex', flexDirection: 'column' }}
            onMouseEnter={(e) => e.currentTarget.style.transform = 'translateY(-4px)'}
            onMouseLeave={(e) => e.currentTarget.style.transform = 'translateY(0)'}
          >
            <Link
              to={`/equipment/${eq.id}`}
              style={{ textDecoration: 'none', color: 'inherit', flex: 1 }}
            >
              <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '12px' }}>
                <h3 style={{ margin: 0, color: 'var(--text-primary)' }}>{eq.name}</h3>
                {eq.qr_code && (
                  <span className="material-icons" style={{ color: 'var(--text-secondary)' }}>qr_code</span>
                )}
              </div>
              {eq.description && (
                <p style={{ color: 'var(--text-secondary)', marginBottom: '12px', fontSize: '14px' }}>
                  {eq.description.length > 100 ? `${eq.description.substring(0, 100)}...` : eq.description}
                </p>
              )}
              <div style={{ display: 'flex', gap: '16px', fontSize: '14px', color: 'var(--text-secondary)' }}>
                <div>
                  <span className="material-icons" style={{ fontSize: '18px', verticalAlign: 'middle' }}>inventory</span>
                  {' '}Всего: {eq.quantity}
                </div>
                <div>
                  <span className="material-icons" style={{ fontSize: '18px', verticalAlign: 'middle' }}>check_circle</span>
                  {' '}Доступно: {eq.available_quantity}
                </div>
              </div>
              {eq.location && (
                <div style={{ marginTop: '8px', fontSize: '14px', color: 'var(--text-secondary)' }}>
                  <span className="material-icons" style={{ fontSize: '18px', verticalAlign: 'middle' }}>location_on</span>
                  {' '}{eq.location}
                </div>
              )}
              {eq.squad_name && (
                <div style={{ marginTop: '8px', fontSize: '14px', color: 'var(--text-secondary)' }}>
                  <span className="material-icons" style={{ fontSize: '18px', verticalAlign: 'middle' }}>groups</span>
                  {' '}
                  <span
                    role="link"
                    tabIndex={0}
                    onClick={(e) => { e.preventDefault(); e.stopPropagation(); navigate(`/squads/${eq.squad_id}`); }}
                    onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); navigate(`/squads/${eq.squad_id}`); } }}
                    style={{ color: 'var(--primary-color)', cursor: 'pointer', textDecoration: 'underline' }}
                  >
                    {eq.squad_name}
                  </span>
                </div>
              )}
            </Link>
            {eq.available_quantity > 0 && (
              <button
                onClick={(e) => {
                  e.preventDefault();
                  e.stopPropagation();
                  addToCart(eq, 1);
                }}
                className={`btn ${isInCart(eq.id) ? 'btn-secondary' : 'btn-primary'}`}
                style={{ 
                  marginTop: '12px', 
                  width: '100%',
                  opacity: isInCart(eq.id) ? 0.7 : 1
                }}
              >
                <span className="material-icons">
                  {isInCart(eq.id) ? 'check_circle' : 'add_shopping_cart'}
                </span>
                {isInCart(eq.id) ? 'В корзине' : 'Добавить в корзину'}
              </button>
            )}
          </div>
        ))}
      </div>

      {filteredEquipment.length === 0 && (
        <div className="card" style={{ textAlign: 'center', padding: '40px' }}>
          <span className="material-icons" style={{ fontSize: '64px', color: 'var(--text-secondary)', marginBottom: '16px' }}>inventory_2</span>
          <p style={{ color: 'var(--text-secondary)' }}>
            {searchTerm ? 'Оборудование не найдено' : 'Нет оборудования'}
          </p>
        </div>
      )}
    </div>
  );
};

export default EquipmentList;

