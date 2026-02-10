import React, { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { equipmentAPI } from '../api/equipment';
import { qrAPI } from '../api/qr';
import { useAuth } from '../context/AuthContext';
import { useCart } from '../context/CartContext';
import { format } from 'date-fns';

const EquipmentDetail = () => {
  const { id } = useParams();
  const navigate = useNavigate();
  const { isAdmin, isResponsible } = useAuth();
  const { addToCart, isInCart } = useCart();
  const [equipment, setEquipment] = useState(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState('');
  const [showQRModal, setShowQRModal] = useState(false);
  const [qrCodeUrl, setQrCodeUrl] = useState(null);
  const [qrData, setQrData] = useState('');

  useEffect(() => {
    fetchEquipment();
  }, [id]);

  const fetchEquipment = async () => {
    try {
      const data = await equipmentAPI.getById(id);
      setEquipment(data);
    } catch (err) {
      setError('Оборудование не найдено');
    } finally {
      setLoading(false);
    }
  };

  const handleDelete = async () => {
    if (!window.confirm('Вы уверены, что хотите удалить это оборудование?')) {
      return;
    }

    try {
      await equipmentAPI.delete(id);
      navigate('/equipment');
    } catch (err) {
      setError('Ошибка при удалении оборудования');
    }
  };

  const handleGenerateQR = async () => {
    try {
      // Получаем данные QR-кода
      const dataResponse = await qrAPI.getData(id);
      setQrData(dataResponse.qr_code || '');

      // Генерируем QR-код
      const blob = await qrAPI.generate(id);
      const url = URL.createObjectURL(blob);
      setQrCodeUrl(url);
      setShowQRModal(true);
    } catch (error) {
      console.error('Failed to generate QR code:', error);
      alert('Ошибка генерации QR-кода');
    }
  };

  const handleDownloadQR = () => {
    if (qrCodeUrl) {
      const link = document.createElement('a');
      link.href = qrCodeUrl;
      link.download = `qr-code-${id}.svg`;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
    }
  };

  if (loading) {
    return <div className="loading"><div className="spinner"></div></div>;
  }

  if (error || !equipment) {
    return (
      <div>
        <div className="error-message">{error || 'Оборудование не найдено'}</div>
        <Link to="/equipment" className="btn btn-secondary">Вернуться к списку</Link>
      </div>
    );
  }

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '24px' }}>
        <Link to="/equipment" className="btn btn-secondary" style={{ textDecoration: 'none' }}>
          <span className="material-icons">arrow_back</span>
          Назад
        </Link>
        <h1 style={{ margin: 0 }}>{equipment.name}</h1>
      </div>

      <div className="card">
        <div style={{ display: 'grid', gap: '16px' }}>
          <div>
            <label className="label">Описание</label>
            <p>{equipment.description || 'Нет описания'}</p>
          </div>

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '16px' }}>
            <div>
              <label className="label">Категория</label>
              <p>{equipment.category_name || 'Не указана'}</p>
            </div>
            <div>
              <label className="label">Всего единиц</label>
              <p style={{ fontSize: '24px', fontWeight: 'bold', color: 'var(--primary-color)' }}>{equipment.quantity}</p>
            </div>
            <div>
              <label className="label">Доступно</label>
              <p style={{ fontSize: '24px', fontWeight: 'bold', color: 'var(--success)' }}>{equipment.available_quantity}</p>
            </div>
            <div>
              <label className="label">Статус</label>
              <p>
                <span style={{
                  padding: '6px 12px',
                  borderRadius: '8px',
                  background: equipment.status === 'available' ? 'rgba(16, 185, 129, 0.2)' : 'rgba(239, 68, 68, 0.2)',
                  color: equipment.status === 'available' ? 'var(--success)' : 'var(--error)',
                  fontSize: '13px',
                  fontWeight: 600,
                  border: `1px solid ${equipment.status === 'available' ? 'rgba(16, 185, 129, 0.3)' : 'rgba(239, 68, 68, 0.3)'}`
                }}>
                  {equipment.status === 'available' ? 'Доступно' : equipment.status === 'maintenance' ? 'На обслуживании' : 'Недоступно'}
                </span>
              </p>
            </div>
          </div>

          {equipment.location && (
            <div>
              <label className="label">Местоположение</label>
              <p>
                <span className="material-icons" style={{ verticalAlign: 'middle', fontSize: '20px' }}>location_on</span>
                {' '}{equipment.location}
              </p>
            </div>
          )}

          {equipment.responsible_name && (
            <div>
              <label className="label">Ответственный</label>
              <p>{equipment.responsible_name}</p>
            </div>
          )}

          {equipment.qr_code && (
            <div>
              <label className="label">QR-код</label>
              <p style={{ fontFamily: 'monospace', background: 'var(--surface)', padding: '12px', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)', color: 'var(--text-primary)' }}>
                {equipment.qr_code}
              </p>
            </div>
          )}

          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '16px' }}>
            <div>
              <label className="label">Создано</label>
              <p>{format(new Date(equipment.created_at), 'dd.MM.yyyy HH:mm')}</p>
            </div>
            <div>
              <label className="label">Обновлено</label>
              <p>{format(new Date(equipment.updated_at), 'dd.MM.yyyy HH:mm')}</p>
            </div>
          </div>
        </div>
      </div>

      <div style={{ display: 'flex', gap: '12px', marginTop: '16px', flexWrap: 'wrap' }}>
        {equipment.available_quantity > 0 && (
          <>
            <Link
              to={`/bookings/create?equipment_id=${equipment.id}`}
              className="btn btn-primary"
              style={{ textDecoration: 'none' }}
            >
              <span className="material-icons">event</span>
              Забронировать
            </Link>
            <button
              onClick={() => {
                addToCart(equipment, 1);
              }}
              className={`btn ${isInCart(equipment.id) ? 'btn-secondary' : 'btn-primary'}`}
              disabled={equipment.available_quantity === 0}
            >
              <span className="material-icons">
                {isInCart(equipment.id) ? 'check_circle' : 'add_shopping_cart'}
              </span>
              {isInCart(equipment.id) ? 'В корзине' : 'Добавить в корзину'}
            </button>
          </>
        )}
        {(isAdmin() || isResponsible()) && (
          <>
            <Link
              to={`/equipment/${id}/edit`}
              className="btn btn-secondary"
              style={{ textDecoration: 'none' }}
            >
              <span className="material-icons">edit</span>
              Редактировать
            </Link>
            <button onClick={handleGenerateQR} className="btn btn-secondary">
              <span className="material-icons">qr_code</span>
              QR-код
            </button>
            {isAdmin() && (
              <button onClick={handleDelete} className="btn btn-danger">
                <span className="material-icons">delete</span>
                Удалить
              </button>
            )}
          </>
        )}
      </div>

      {/* Модальное окно QR-кода */}
      {showQRModal && (
        <div
          style={{
            position: 'fixed',
            top: 0,
            left: 0,
            right: 0,
            bottom: 0,
            background: 'rgba(0, 0, 0, 0.5)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            zIndex: 1000,
          }}
          onClick={() => {
            setShowQRModal(false);
            if (qrCodeUrl) {
              URL.revokeObjectURL(qrCodeUrl);
              setQrCodeUrl(null);
            }
          }}
        >
          <div
            className="card"
            style={{
              maxWidth: '500px',
              width: '90%',
              textAlign: 'center',
              boxShadow: '0 8px 32px rgba(0,0,0,0.5)',
              border: '1px solid rgba(255,255,255,0.1)',
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
              <h2 style={{ margin: 0 }}>QR-код оборудования</h2>
              <button
                onClick={() => {
                  setShowQRModal(false);
                  if (qrCodeUrl) {
                    URL.revokeObjectURL(qrCodeUrl);
                    setQrCodeUrl(null);
                  }
                }}
                style={{
                  background: 'none',
                  border: 'none',
                  cursor: 'pointer',
                  fontSize: '24px',
                  color: 'var(--text-secondary)',
                }}
              >
                ×
              </button>
            </div>

            {qrCodeUrl && (
              <div>
                <img
                  src={qrCodeUrl}
                  alt="QR Code"
                  style={{
                    display: 'block',
                    width: '400px',
                    height: '400px',
                    maxWidth: '100%',
                    margin: '0 auto 20px',
                  }}
                />
                {qrData && (
                  <div style={{ marginBottom: '20px', padding: '12px', background: 'var(--surface)', borderRadius: '8px', border: '1px solid rgba(255,255,255,0.05)' }}>
                    <strong style={{ color: 'var(--text-primary)' }}>Код:</strong> <span style={{ color: 'var(--text-secondary)', fontFamily: 'monospace' }}>{qrData}</span>
                  </div>
                )}
                <button onClick={handleDownloadQR} className="btn btn-primary">
                  <span className="material-icons">download</span>
                  Скачать QR-код
                </button>
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default EquipmentDetail;

