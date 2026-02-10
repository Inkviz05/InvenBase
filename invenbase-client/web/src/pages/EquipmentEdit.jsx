import React, { useState, useEffect } from 'react';
import { useParams, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { equipmentAPI } from '../api/equipment';
import { categoriesAPI } from '../api/categories';

const EquipmentEdit = () => {
  const { id } = useParams();
  const { isAdmin, isResponsible } = useAuth();
  const navigate = useNavigate();
  const [categories, setCategories] = useState([]);
  const [formData, setFormData] = useState({
    name: '',
    description: '',
    category_id: '',
    quantity: 1,
    available_quantity: 1,
    location: '',
    status: 'available',
  });
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!isAdmin() && !isResponsible()) {
      navigate('/equipment');
      return;
    }
    fetchData();
  }, [id, isAdmin, isResponsible, navigate]);

  const fetchData = async () => {
    try {
      const [equipment, categoriesData] = await Promise.all([
        equipmentAPI.getById(id),
        categoriesAPI.getAll(),
      ]);
      setCategories(categoriesData);
      setFormData({
        name: equipment.name || '',
        description: equipment.description || '',
        category_id: equipment.category_id || '',
        quantity: equipment.quantity || 1,
        available_quantity: equipment.available_quantity || 0,
        location: equipment.location || '',
        status: equipment.status || 'available',
      });
    } catch (error) {
      console.error('Failed to fetch data:', error);
      alert('Ошибка загрузки данных');
      navigate('/equipment');
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setSaving(true);

    try {
      const dataToSend = { ...formData };
      if (!dataToSend.category_id) {
        delete dataToSend.category_id;
      }
      dataToSend.quantity = parseInt(dataToSend.quantity);
      dataToSend.available_quantity = parseInt(dataToSend.available_quantity);

      await equipmentAPI.update(id, dataToSend);
      alert('Оборудование обновлено');
      navigate(`/equipment/${id}`);
    } catch (error) {
      console.error('Failed to update equipment:', error);
      alert(error.response?.data?.message || 'Ошибка обновления оборудования');
    } finally {
      setSaving(false);
    }
  };

  if (loading) {
    return <div className="loading"><div className="spinner"></div></div>;
  }

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '24px' }}>
        <button onClick={() => navigate(`/equipment/${id}`)} className="btn btn-secondary">
          <span className="material-icons">arrow_back</span>
          Назад
        </button>
        <h1 style={{ margin: 0 }}>Редактировать оборудование</h1>
      </div>

      <div className="card">
        <form onSubmit={handleSubmit}>
          <div style={{ display: 'grid', gap: '16px' }}>
            <div>
              <label className="label">Название *</label>
              <input
                type="text"
                className="input"
                value={formData.name}
                onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                required
              />
            </div>

            <div>
              <label className="label">Описание</label>
              <textarea
                className="input"
                value={formData.description}
                onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                rows={4}
              />
            </div>

            <div>
              <label className="label">Категория</label>
              <select
                className="input"
                value={formData.category_id}
                onChange={(e) => setFormData({ ...formData, category_id: e.target.value })}
              >
                <option value="">Выберите категорию</option>
                {categories.map((cat) => (
                  <option key={cat.id} value={cat.id}>
                    {cat.name}
                  </option>
                ))}
              </select>
            </div>

            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(200px, 1fr))', gap: '16px' }}>
              <div>
                <label className="label">Количество *</label>
                <input
                  type="number"
                  className="input"
                  value={formData.quantity}
                  onChange={(e) => setFormData({ ...formData, quantity: e.target.value })}
                  min="1"
                  required
                />
              </div>

              <div>
                <label className="label">Доступно</label>
                <input
                  type="number"
                  className="input"
                  value={formData.available_quantity}
                  onChange={(e) => setFormData({ ...formData, available_quantity: e.target.value })}
                  min="0"
                />
              </div>
            </div>

            <div>
              <label className="label">Местоположение</label>
              <input
                type="text"
                className="input"
                value={formData.location}
                onChange={(e) => setFormData({ ...formData, location: e.target.value })}
              />
            </div>

            <div>
              <label className="label">Статус</label>
              <select
                className="input"
                value={formData.status}
                onChange={(e) => setFormData({ ...formData, status: e.target.value })}
              >
                <option value="available">Доступно</option>
                <option value="maintenance">На обслуживании</option>
                <option value="unavailable">Недоступно</option>
              </select>
            </div>

            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end', marginTop: '16px' }}>
              <button type="button" onClick={() => navigate(`/equipment/${id}`)} className="btn btn-secondary">
                Отмена
              </button>
              <button type="submit" className="btn btn-primary" disabled={saving}>
                {saving ? 'Сохранение...' : 'Сохранить'}
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
};

export default EquipmentEdit;
