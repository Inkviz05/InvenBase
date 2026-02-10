import React, { useState, useEffect, useRef } from 'react';
import { useNavigate } from 'react-router-dom';
import { equipmentAPI } from '../api/equipment';

const QRScanner = () => {
  const [scanning, setScanning] = useState(false);
  const [result, setResult] = useState(null);
  const [error, setError] = useState('');
  const [cameraPermission, setCameraPermission] = useState(null);
  const scannerRef = useRef(null);
  const navigate = useNavigate();

  useEffect(() => {
    checkCameraPermission();
    return () => {
      stopScanning();
    };
  }, []);

  const checkCameraPermission = async () => {
    try {
      const stream = await navigator.mediaDevices.getUserMedia({ video: true });
      stream.getTracks().forEach(track => track.stop());
      setCameraPermission(true);
    } catch (err) {
      setCameraPermission(false);
      setError('Доступ к камере запрещён. Разрешите доступ в настройках браузера.');
    }
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
      setError('');
      setResult(null);
      
      // Останавливаем предыдущий экземпляр, если он существует
      if (scannerRef.current) {
        await stopScanning();
      }
      
      // Принудительно освобождаем все потоки камеры
      await releaseAllCameraStreams();
      
      // Даём больше времени на освобождение камеры
      await new Promise(resolve => setTimeout(resolve, 500));
      
      setScanning(true);

      // Динамически загружаем библиотеку html5-qrcode
      let Html5Qrcode;
      if (window.Html5Qrcode) {
        Html5Qrcode = window.Html5Qrcode;
      } else {
        // Пытаемся загрузить библиотеку
        await loadHtml5QrcodeLibrary();
        Html5Qrcode = window.Html5Qrcode;
      }

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
      setResult(equipment);
      navigate(`/equipment/${equipment.id}`);
    } catch (err) {
      setError('Оборудование с таким QR-кодом не найдено');
    }
  };

  const handleManualInput = async () => {
    const qrCode = prompt('Введите QR-код вручную:');
    if (qrCode) {
      await handleQRCode(qrCode);
    }
  };

  return (
    <div>
      <h1 style={{ marginBottom: '24px' }}>Сканер QR-кодов</h1>

      {error && <div className="error-message">{error}</div>}

      {cameraPermission === false && (
        <div className="card" style={{ marginBottom: '24px' }}>
          <p>Для работы сканера необходим доступ к камере устройства.</p>
          <button onClick={checkCameraPermission} className="btn btn-primary">
            Проверить разрешения
          </button>
        </div>
      )}

      <div className="card">
        {!scanning ? (
          <div style={{ textAlign: 'center', padding: '40px' }}>
            <span className="material-icons" style={{ fontSize: '64px', color: 'var(--primary-color)', marginBottom: '16px' }}>
              qr_code_scanner
            </span>
            <p style={{ marginBottom: '24px', color: 'var(--text-secondary)' }}>
              Нажмите кнопку для начала сканирования QR-кода
            </p>
            <button onClick={startScanning} className="btn btn-primary" disabled={cameraPermission === false}>
              <span className="material-icons">camera_alt</span>
              Начать сканирование
            </button>
            <button onClick={handleManualInput} className="btn btn-secondary" style={{ marginLeft: '12px' }}>
              <span className="material-icons">keyboard</span>
              Ввести вручную
            </button>
          </div>
        ) : (
          <div>
            <div id="qr-reader" style={{ width: '100%', marginBottom: '16px' }}></div>
            <button onClick={stopScanning} className="btn btn-danger">
              <span className="material-icons">stop</span>
              Остановить сканирование
            </button>
          </div>
        )}
      </div>

      {result && (
        <div className="card" style={{ background: 'rgba(16, 185, 129, 0.1)', border: '1px solid rgba(16, 185, 129, 0.3)' }}>
          <h3>Найдено оборудование:</h3>
          <p><strong>{result.name}</strong></p>
          <button
            onClick={() => navigate(`/equipment/${result.id}`)}
            className="btn btn-primary"
          >
            Перейти к оборудованию
          </button>
        </div>
      )}
    </div>
  );
};

export default QRScanner;

