import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { bookingsAPI } from '../api/bookings';
import { useCart } from '../context/CartContext';

const BookingBulkCreate = () => {
  const navigate = useNavigate();
  const { cartItems, clearCart, updateQuantity, removeFromCart } = useCart();
  const [formData, setFormData] = useState({
    start_date: '',
    end_date: '',
    purpose: '',
    permission_type: 'internal',
  });
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    
    if (cartItems.length === 0) {
      setError('Корзина пуста. Добавьте оборудование в корзину.');
      return;
    }

    setLoading(true);

    try {
      // Создаем бронирования для каждого элемента корзины
      const bookingPromises = cartItems.map(item =>
        bookingsAPI.create({
          equipment_id: item.equipment.id,
          group_id: null,
          quantity: item.quantity,
          start_date: new Date(formData.start_date).toISOString(),
          end_date: new Date(formData.end_date).toISOString(),
          purpose: formData.purpose || null,
          permission_type: formData.permission_type,
        })
      );

      await Promise.all(bookingPromises);
      clearCart();
      navigate('/bookings');
    } catch (err) {
      setError(err.response?.data?.error || 'Ошибка при создании бронирований');
    } finally {
      setLoading(false);
    }
  };

  const totalItems = cartItems.reduce((sum, item) => sum + item.quantity, 0);

  if (cartItems.length === 0) {
    return (
      <div className="booking-create-container">
        <h1 className="page-title">Массовое бронирование</h1>
        <div className="card">
          <div style={{ textAlign: 'center', padding: '40px' }}>
            <span className="material-icons" style={{ fontSize: '64px', color: 'var(--text-secondary)', marginBottom: '16px', display: 'block' }}>
              shopping_cart
            </span>
            <p style={{ color: 'var(--text-secondary)', marginBottom: '24px' }}>
              Корзина пуста
            </p>
            <button
              onClick={() => navigate('/equipment')}
              className="btn btn-primary"
            >
              <span className="material-icons">inventory</span>
              Выбрать оборудование
            </button>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="booking-create-container">
      <h1 className="page-title">Массовое бронирование</h1>

      {error && <div className="error-message">{error}</div>}

      <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '24px', marginBottom: '24px' }}>
        {/* Список оборудования в корзине */}
        <div className="card">
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
            <h2 style={{ margin: 0, fontSize: '18px' }}>Оборудование в корзине</h2>
            <span style={{ 
              background: 'var(--primary-color)', 
              color: 'white', 
              borderRadius: '50%', 
              width: '28px', 
              height: '28px', 
              display: 'flex', 
              alignItems: 'center', 
              justifyContent: 'center',
              fontSize: '14px',
              fontWeight: 'bold'
            }}>
              {cartItems.length}
            </span>
          </div>
          
          <div style={{ maxHeight: '400px', overflowY: 'auto' }}>
            {cartItems.map((item) => (
              <div 
                key={item.equipment.id} 
                className="cart-item"
                style={{
                  padding: '12px',
                  border: '1px solid var(--divider)',
                  borderRadius: '8px',
                  marginBottom: '12px',
                  background: 'var(--surface)'
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'start', marginBottom: '8px' }}>
                  <div style={{ flex: 1 }}>
                    <h4 style={{ margin: 0, marginBottom: '4px', color: 'var(--text-primary)' }}>
                      {item.equipment.name}
                    </h4>
                    <div style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                      Доступно: {item.equipment.available_quantity} из {item.equipment.quantity}
                    </div>
                  </div>
                  <button
                    onClick={() => removeFromCart(item.equipment.id)}
                    style={{
                      background: 'none',
                      border: 'none',
                      color: 'var(--error)',
                      cursor: 'pointer',
                      padding: '4px',
                      display: 'flex',
                      alignItems: 'center'
                    }}
                    title="Удалить"
                  >
                    <span className="material-icons" style={{ fontSize: '20px' }}>close</span>
                  </button>
                </div>
                <div style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                  <label style={{ fontSize: '14px', color: 'var(--text-secondary)' }}>Количество:</label>
                  <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                    <button
                      onClick={() => updateQuantity(item.equipment.id, item.quantity - 1)}
                      className="btn btn-secondary"
                      style={{ 
                        padding: '4px 8px', 
                        minWidth: '32px',
                        fontSize: '16px'
                      }}
                      disabled={item.quantity <= 1}
                    >
                      -
                    </button>
                    <input
                      type="number"
                      value={item.quantity}
                      onChange={(e) => {
                        const newQuantity = parseInt(e.target.value) || 1;
                        updateQuantity(item.equipment.id, newQuantity);
                      }}
                      min="1"
                      max={item.equipment.available_quantity}
                      style={{
                        width: '60px',
                        padding: '4px 8px',
                        border: '1px solid var(--divider)',
                        borderRadius: '4px',
                        textAlign: 'center',
                        fontSize: '14px'
                      }}
                    />
                    <button
                      onClick={() => updateQuantity(item.equipment.id, item.quantity + 1)}
                      className="btn btn-secondary"
                      style={{ 
                        padding: '4px 8px', 
                        minWidth: '32px',
                        fontSize: '16px'
                      }}
                      disabled={item.quantity >= item.equipment.available_quantity}
                    >
                      +
                    </button>
                  </div>
                </div>
              </div>
            ))}
          </div>

          <div style={{ 
            marginTop: '16px', 
            padding: '12px', 
            background: 'rgba(168, 85, 247, 0.1)',
            border: '1px solid rgba(168, 85, 247, 0.3)', 
            borderRadius: '8px',
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center'
          }}>
            <strong>Всего единиц:</strong>
            <strong style={{ fontSize: '18px', color: 'var(--text-primary)' }}>{totalItems}</strong>
          </div>
        </div>

        {/* Форма бронирования */}
        <form onSubmit={handleSubmit} className="card">
          <h2 style={{ margin: 0, marginBottom: '16px', fontSize: '18px' }}>Параметры бронирования</h2>

          <label className="label">Дата начала *</label>
          <input
            type="datetime-local"
            className="input"
            value={formData.start_date}
            onChange={(e) => setFormData({ ...formData, start_date: e.target.value })}
            required
          />

          <label className="label">Дата окончания *</label>
          <input
            type="datetime-local"
            className="input"
            value={formData.end_date}
            onChange={(e) => setFormData({ ...formData, end_date: e.target.value })}
            required
          />

          <label className="label">Тип разрешения *</label>
          <select
            className="input"
            value={formData.permission_type}
            onChange={(e) => setFormData({ ...formData, permission_type: e.target.value })}
            required
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
          />

          <div className="form-actions">
            <button type="submit" className="btn btn-primary" disabled={loading}>
              {loading ? 'Создание...' : `Забронировать всё (${cartItems.length} ${cartItems.length === 1 ? 'позиция' : 'позиций'})`}
            </button>
            <button
              type="button"
              className="btn btn-secondary"
              onClick={() => navigate('/equipment')}
            >
              Добавить ещё
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
    </div>
  );
};

export default BookingBulkCreate;

