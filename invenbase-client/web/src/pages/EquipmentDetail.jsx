import React, { useState, useEffect } from 'react';
import { useParams, useNavigate, Link } from 'react-router-dom';
import { equipmentAPI } from '../api/equipment';
import { squadsAPI } from '../api/squads';
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
  const [showDeleteModal, setShowDeleteModal] = useState(false);
  const [deleteConfirmName, setDeleteConfirmName] = useState('');
  const [movements, setMovements] = useState([]);
  const [movementsLoading, setMovementsLoading] = useState(false);
  const [showMoveModal, setShowMoveModal] = useState(false);
  const [moveForm, setMoveForm] = useState({
    to_squad_id: '',
    to_location: '',
    comment: '',
  });
  const [squads, setSquads] = useState([]);

  useEffect(() => {
    fetchEquipment();
    fetchSquads();
    fetchMovements();
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

  const fetchSquads = async () => {
    try {
      const data = await squadsAPI.getAll();
      setSquads(data);
    } catch (err) {
      console.error('Failed to fetch squads:', err);
    }
  };

  const fetchMovements = async () => {
    setMovementsLoading(true);
    try {
      const data = await equipmentAPI.getMovements(id);
      setMovements(data);
    } catch (err) {
      console.error('Failed to fetch movements:', err);
    } finally {
      setMovementsLoading(false);
    }
  };

  const openDeleteModal = () => {
    if (equipment) {
      setDeleteConfirmName('');
      setShowDeleteModal(true);
    }
  };

  const handleDeleteConfirm = async () => {
    if (!equipment) return;
    if (deleteConfirmName.trim() !== equipment.name.trim()) {
      alert('Название оборудования не совпадает. Удаление отменено.');
      return;
    }
    setShowDeleteModal(false);
    try {
      await equipmentAPI.delete(id);
      navigate('/equipment');
    } catch (err) {
      setError('Ошибка при удалении оборудования');
    }
  };

  const openMoveModal = () => {
    if (!equipment) return;
    setMoveForm({
      to_squad_id: equipment.squad_id || '',
      to_location: equipment.location || '',
      comment: '',
    });
    setShowMoveModal(true);
  };

  const handleMoveSubmit = async (e) => {
    e.preventDefault();
    try {
      const payload = {
        to_squad_id: moveForm.to_squad_id || null,
        to_location: moveForm.to_location || null,
        comment: moveForm.comment || null,
      };
      await equipmentAPI.move(id, payload);
      setShowMoveModal(false);
      await Promise.all([fetchEquipment(), fetchMovements()]);
    } catch (err) {
      console.error('Failed to move equipment:', err);
      alert(err.response?.data?.message || 'Ошибка переноса оборудования');
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

  const buildQRLabelCanvas = (qrImageUrl, callback) => {
    if (!qrImageUrl || !equipment) return;
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.onload = () => {
      const qrSize = 280;
      const padding = 24;
      const lineHeight = 22;
      const codeText = `Код для ручного ввода: ${qrData || equipment.qr_code || ''}`;
      const nameText = `Название: ${equipment.name || ''}`;
      const descText = `Описание: ${(equipment.description && equipment.description.trim()) ? equipment.description.trim() : '—'}`;
      const maxTextWidth = 320;
      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      ctx.font = '14px sans-serif';
      const wrap = (t) => {
        const lines = [];
        let rest = t;
        while (rest.length) {
          let chunk = rest.slice(0, 28);
          if (chunk.length < rest.length) {
            const lastSpace = chunk.lastIndexOf(' ');
            if (lastSpace > 18) chunk = chunk.slice(0, lastSpace + 1);
          }
          lines.push(chunk);
          rest = rest.slice(chunk.length).trim();
        }
        return lines;
      };
      const descLines = wrap(descText.length > 80 ? descText.slice(0, 80) + '…' : descText);
      const textLines = [codeText, nameText, ...descLines];
      canvas.width = Math.max(qrSize + padding * 2, maxTextWidth + padding * 2);
      canvas.height = padding + qrSize + padding + textLines.length * lineHeight + padding;
      ctx.fillStyle = '#fff';
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      const qrX = (canvas.width - qrSize) / 2;
      ctx.drawImage(img, 0, 0, img.width, img.height, qrX, padding, qrSize, qrSize);
      ctx.fillStyle = '#000';
      ctx.font = '14px sans-serif';
      let y = padding + qrSize + padding + lineHeight;
      [codeText, nameText].forEach((t) => {
        ctx.fillText(t.length > 42 ? t.slice(0, 42) + '…' : t, padding, y);
        y += lineHeight;
      });
      descLines.forEach((line) => {
        ctx.fillText(line, padding, y);
        y += lineHeight;
      });
      callback(canvas);
    };
    img.onerror = () => callback(null);
    img.src = qrCodeUrl;
  };

  const handleDownloadQR = () => {
    buildQRLabelCanvas(qrCodeUrl, (canvas) => {
      if (!canvas) return;
      canvas.toBlob((blob) => {
        if (!blob) return;
        const url = URL.createObjectURL(blob);
        const link = document.createElement('a');
        link.href = url;
        const rawName = (equipment?.name || id || '').trim();
        const safeName = rawName.replace(/[\x00-\x1f\\/:*?"<>|]/g, '_').replace(/\s+/g, ' ').trim() || `qr-${(id || '').slice(0, 8)}`;
        link.download = `qr-${safeName}.png`;
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
        URL.revokeObjectURL(url);
      }, 'image/png');
    });
  };

  const handlePrintQR = () => {
    buildQRLabelCanvas(qrCodeUrl, (canvas) => {
      if (!canvas) return;
      const dataUrl = canvas.toDataURL('image/png');
      const w = window.open('', '_blank');
      if (!w) {
        alert('Разрешите всплывающие окна для печати');
        return;
      }
      w.document.write(`
        <!DOCTYPE html><html><head><title>QR-код: ${equipment?.name || ''}</title></head>
        <body style="margin:0;display:flex;justify-content:center;align-items:center;min-height:100vh;">
          <img src="${dataUrl}" alt="QR" style="max-width:100%;height:auto;" />
        </body></html>
      `);
      w.document.close();
      w.onload = () => {
        w.focus();
        w.setTimeout(() => {
          w.print();
          w.onafterprint = () => w.close();
        }, 250);
      };
    });
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
              <p style={{ fontFamily: 'monospace', background: 'var(--inline-panel-bg)', padding: '12px', borderRadius: '8px', border: '1px solid var(--divider)', color: 'var(--text-primary)' }}>
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

      {(isAdmin() || isResponsible()) && (
        <div className="card" style={{ marginTop: '24px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
            <h2 style={{ margin: 0, fontSize: '18px' }}>Перемещение оборудования</h2>
            <button onClick={openMoveModal} className="btn btn-secondary">
              <span className="material-icons">sync_alt</span>
              Перенести
            </button>
          </div>
          <p style={{ color: 'var(--text-secondary)', fontSize: '14px' }}>
            Можно изменить сквад (кабинет) и/или местоположение. Все изменения сохраняются в истории перемещений.
          </p>
        </div>
      )}

      <div className="card" style={{ marginTop: '24px' }}>
        <h2 style={{ marginTop: 0, marginBottom: '16px', fontSize: '18px' }}>История перемещений</h2>
        {movementsLoading ? (
          <div className="loading"><div className="spinner"></div></div>
        ) : movements.length === 0 ? (
          <p style={{ color: 'var(--text-secondary)' }}>Перемещения ещё не зарегистрированы.</p>
        ) : (
          <>
            <div className="table-responsive">
              <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '14px' }}>
            <thead>
              <tr style={{ borderBottom: '1px solid var(--divider)', color: 'var(--text-secondary)' }}>
                <th style={{ padding: '8px', textAlign: 'left' }}>Когда</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Откуда</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Куда</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Кто</th>
                <th style={{ padding: '8px', textAlign: 'left' }}>Комментарий</th>
              </tr>
            </thead>
            <tbody>
              {movements.map((m) => (
                <tr key={m.id} style={{ borderBottom: '1px solid var(--divider)' }}>
                  <td style={{ padding: '8px', whiteSpace: 'nowrap' }}>
                    {m.moved_at ? format(new Date(m.moved_at), 'dd.MM.yyyy HH:mm') : '-'}
                  </td>
                  <td style={{ padding: '8px' }}>
                    {m.from_location || '—'}
                  </td>
                  <td style={{ padding: '8px' }}>
                    {m.to_location || '—'}
                  </td>
                  <td style={{ padding: '8px' }}>
                    {m.moved_by_name || '—'}
                  </td>
                  <td style={{ padding: '8px' }}>
                    {m.comment || '—'}
                  </td>
                </tr>
              ))}
            </tbody>
              </table>
            </div>
          </>
        )}
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
            {(isAdmin() || isResponsible()) && (
              <button onClick={openDeleteModal} className="btn btn-danger">
                <span className="material-icons">delete</span>
                Удалить
              </button>
            )}
          </>
        )}
      </div>

      {/* Модальное окно подтверждения удаления */}
      {showDeleteModal && equipment && (
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
          onClick={() => setShowDeleteModal(false)}
        >
          <div
            className="card"
            style={{
              maxWidth: '400px',
              width: '90%',
              boxShadow: 'var(--modal-shadow)',
              border: '1px solid var(--dropdown-border)',
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ marginTop: 0 }}>Удаление оборудования</h3>
            <p style={{ color: 'var(--text-secondary)', marginBottom: '16px' }}>
              Для подтверждения введите точное название: <strong style={{ color: 'var(--text-primary)' }}>{equipment.name}</strong>
            </p>
            <input
              type="text"
              className="input"
              placeholder={equipment.name}
              value={deleteConfirmName}
              onChange={(e) => setDeleteConfirmName(e.target.value)}
              style={{ marginBottom: '16px' }}
              autoFocus
            />
            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
              <button type="button" onClick={() => setShowDeleteModal(false)} className="btn btn-secondary">
                Отмена
              </button>
              <button type="button" onClick={handleDeleteConfirm} className="btn btn-danger">
                Удалить
              </button>
            </div>
          </div>
        </div>
      )}

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
              boxShadow: 'var(--modal-shadow)',
              border: '1px solid var(--dropdown-border)',
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
                  <div style={{ marginBottom: '20px', padding: '12px', background: 'var(--inline-panel-bg)', borderRadius: '8px', border: '1px solid var(--divider)' }}>
                    <strong style={{ color: 'var(--text-primary)' }}>Код:</strong> <span style={{ color: 'var(--text-secondary)', fontFamily: 'monospace' }}>{qrData}</span>
                  </div>
                )}
                <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap', justifyContent: 'center' }}>
                  <button onClick={handleDownloadQR} className="btn btn-primary">
                    <span className="material-icons">download</span>
                    Скачать PNG
                  </button>
                  <button onClick={handlePrintQR} className="btn btn-secondary">
                    <span className="material-icons">print</span>
                    Печать
                  </button>
                </div>
              </div>
            )}
          </div>
        </div>
      )}

      {/* Модальное окно переноса оборудования */}
      {showMoveModal && equipment && (
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
          onClick={() => setShowMoveModal(false)}
        >
          <div
            className="card"
            style={{
              maxWidth: '500px',
              width: '90%',
              boxShadow: 'var(--modal-shadow)',
              border: '1px solid var(--dropdown-border)',
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ marginTop: 0, marginBottom: '8px' }}>Перенос оборудования</h3>
            <p style={{ color: 'var(--text-secondary)', fontSize: '12px', marginBottom: '12px' }}>
              <strong style={{ color: 'var(--text-primary)' }}>{equipment.name}</strong>
            </p>
            <p style={{ color: 'var(--text-secondary)', fontSize: '13px', marginBottom: '16px', lineHeight: 1.4 }}>
              Укажите новое местоположение. Запись о перемещении сохранится в истории.
            </p>
            <div style={{ background: 'var(--inline-panel-bg)', padding: '12px', borderRadius: '8px', marginBottom: '16px', border: '1px solid var(--divider)' }}>
              <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '6px', fontWeight: 600 }}>Сейчас:</div>
              <div style={{ fontSize: '13px', color: 'var(--text-primary)' }}>Сквад: {equipment.squad_name || 'Без сквада'}</div>
              <div style={{ fontSize: '13px', color: 'var(--text-primary)' }}>Местоположение: {equipment.location || '—'}</div>
            </div>
            <form onSubmit={handleMoveSubmit}>
              <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 600 }}>Куда переносим:</div>
              <div style={{ marginBottom: '12px' }}>
                <label className="label">Сквад (кабинет / подразделение)</label>
                <select
                  className="input"
                  value={moveForm.to_squad_id}
                  onChange={(e) => setMoveForm({ ...moveForm, to_squad_id: e.target.value })}
                >
                  <option value="">Без сквада</option>
                  {squads.map((s) => (
                    <option key={s.id} value={s.id}>{s.name}</option>
                  ))}
                </select>
              </div>
              <div style={{ marginBottom: '12px' }}>
                <label className="label">Местоположение (адрес в помещении)</label>
                <input
                  type="text"
                  className="input"
                  value={moveForm.to_location}
                  onChange={(e) => setMoveForm({ ...moveForm, to_location: e.target.value })}
                  placeholder="Например: Кабинет 227, полка 3, стол 5"
                />
              </div>
              <div style={{ marginBottom: '16px' }}>
                <label className="label">Комментарий (необязательно)</label>
                <textarea
                  className="input"
                  rows={2}
                  value={moveForm.comment}
                  onChange={(e) => setMoveForm({ ...moveForm, comment: e.target.value })}
                  placeholder="Причина или детали перемещения"
                />
              </div>
              <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
                <button
                  type="button"
                  className="btn btn-secondary"
                  onClick={() => setShowMoveModal(false)}
                >
                  Отмена
                </button>
                <button type="submit" className="btn btn-primary">
                  Перенести
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};

export default EquipmentDetail;

