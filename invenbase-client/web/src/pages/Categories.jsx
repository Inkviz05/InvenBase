import React, { useState, useEffect } from 'react';
import { useAuth } from '../context/AuthContext';
import { categoriesAPI } from '../api/categories';

const Categories = () => {
  const { isAdmin, isResponsible } = useAuth();
  const [categories, setCategories] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [editingCategory, setEditingCategory] = useState(null);
  const [formData, setFormData] = useState({ name: '', description: '' });
  const [deleteTarget, setDeleteTarget] = useState(null);
  const [deleteConfirmName, setDeleteConfirmName] = useState('');

  useEffect(() => {
    if (isAdmin() || isResponsible()) {
      fetchCategories();
    } else {
      setLoading(false);
    }
  }, [isAdmin, isResponsible]);

  const fetchCategories = async () => {
    try {
      const data = await categoriesAPI.getAll();
      setCategories(data);
    } catch (error) {
      console.error('Failed to fetch categories:', error);
      alert('Ошибка загрузки категорий');
    } finally {
      setLoading(false);
    }
  };

  const handleOpenModal = (category = null) => {
    if (category) {
      setEditingCategory(category);
      setFormData({ name: category.name || '', description: category.description || '' });
    } else {
      setEditingCategory(null);
      setFormData({ name: '', description: '' });
    }
    setShowModal(true);
  };

  const handleCloseModal = () => {
    setShowModal(false);
    setEditingCategory(null);
    setFormData({ name: '', description: '' });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      if (editingCategory) {
        await categoriesAPI.update(editingCategory.id, formData);
        alert('Категория обновлена');
      } else {
        await categoriesAPI.create(formData);
        alert('Категория создана');
      }
      handleCloseModal();
      fetchCategories();
    } catch (error) {
      console.error('Failed to save category:', error);
      alert(error.response?.data?.message || 'Ошибка сохранения категории');
    }
  };

  const openDeleteModal = (category) => {
    if (category) {
      setDeleteTarget(category);
      setDeleteConfirmName('');
    }
  };

  const closeDeleteModal = () => {
    setDeleteTarget(null);
    setDeleteConfirmName('');
  };

  const handleDeleteConfirm = async () => {
    if (!deleteTarget) return;
    if (deleteConfirmName.trim() !== deleteTarget.name.trim()) {
      alert('Название категории не совпадает. Удаление отменено.');
      return;
    }
    const id = deleteTarget.id;
    closeDeleteModal();
    try {
      await categoriesAPI.delete(id);
      alert('Категория удалена');
      fetchCategories();
    } catch (error) {
      console.error('Failed to delete category:', error);
      alert(error.response?.data?.message || 'Ошибка удаления категории. Возможно, в категории есть оборудование.');
    }
  };

  if (!isAdmin() && !isResponsible()) {
    return (
      <div>
        <div className="error-message">У вас нет прав для доступа к этой странице</div>
      </div>
    );
  }

  if (loading) {
    return <div className="loading"><div className="spinner"></div></div>;
  }

  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
        <h1>Категории оборудования</h1>
        <button onClick={() => handleOpenModal()} className="btn btn-primary">
          <span className="material-icons">add</span>
          Добавить категорию
        </button>
      </div>

      <div className="card">
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ 
              borderBottom: '2px solid var(--divider)',
              background: 'var(--surface)'
            }}>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Название</th>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Описание</th>
              <th style={{ padding: '12px', textAlign: 'right', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Действия</th>
            </tr>
          </thead>
          <tbody>
            {categories.map((category) => (
              <tr key={category.id} style={{ borderBottom: '1px solid var(--divider)' }}>
                <td style={{ padding: '12px', color: 'var(--text-primary)' }}>{category.name}</td>
                <td style={{ padding: '12px', color: 'var(--text-secondary)' }}>
                  {category.description || '-'}
                </td>
                <td style={{ padding: '12px', textAlign: 'right' }}>
                  <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
                    <button
                      onClick={() => handleOpenModal(category)}
                      className="btn btn-secondary"
                      style={{ fontSize: '12px', padding: '6px 12px' }}
                    >
                      <span className="material-icons" style={{ fontSize: '18px' }}>edit</span>
                      Редактировать
                    </button>
                    {(isAdmin() || isResponsible()) && (
                      <button
                        onClick={() => openDeleteModal(category)}
                        className="btn btn-danger"
                        style={{ fontSize: '12px', padding: '6px 12px' }}
                      >
                        <span className="material-icons" style={{ fontSize: '18px' }}>delete</span>
                        Удалить
                      </button>
                    )}
                  </div>
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        {categories.length === 0 && (
          <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
            <span className="material-icons" style={{ fontSize: '64px', marginBottom: '16px', display: 'block' }}>
              category
            </span>
            <p>Нет категорий</p>
          </div>
        )}
      </div>

      {/* Модальное окно подтверждения удаления категории */}
      {deleteTarget && (
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
          onClick={closeDeleteModal}
        >
          <div
            className="card"
            style={{
              maxWidth: '400px',
              width: '90%',
              boxShadow: '0 8px 32px rgba(0,0,0,0.5)',
              border: '1px solid rgba(255,255,255,0.1)',
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <h3 style={{ marginTop: 0 }}>Удаление категории</h3>
            <p style={{ color: 'var(--text-secondary)', marginBottom: '16px' }}>
              Для подтверждения введите точное название: <strong style={{ color: 'var(--text-primary)' }}>{deleteTarget.name}</strong>
            </p>
            <input
              type="text"
              className="input"
              placeholder={deleteTarget.name}
              value={deleteConfirmName}
              onChange={(e) => setDeleteConfirmName(e.target.value)}
              style={{ marginBottom: '16px' }}
              autoFocus
            />
            <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
              <button type="button" onClick={closeDeleteModal} className="btn btn-secondary">
                Отмена
              </button>
              <button type="button" onClick={handleDeleteConfirm} className="btn btn-danger">
                Удалить
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Модальное окно */}
      {showModal && (
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
          onClick={handleCloseModal}
        >
          <div
            className="card"
            style={{
              maxWidth: '500px',
              width: '90%',
              maxHeight: '90vh',
              overflow: 'auto',
              boxShadow: '0 8px 32px rgba(0,0,0,0.5)',
              border: '1px solid rgba(255,255,255,0.1)',
            }}
            onClick={(e) => e.stopPropagation()}
          >
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '24px' }}>
              <h2 style={{ margin: 0 }}>
                {editingCategory ? 'Редактировать категорию' : 'Добавить категорию'}
              </h2>
              <button
                onClick={handleCloseModal}
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

            <form onSubmit={handleSubmit}>
              <div style={{ marginBottom: '16px' }}>
                <label className="label">Название *</label>
                <input
                  type="text"
                  className="input"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  required
                />
              </div>

              <div style={{ marginBottom: '24px' }}>
                <label className="label">Описание</label>
                <textarea
                  className="input"
                  value={formData.description}
                  onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                  rows={4}
                />
              </div>

              <div style={{ display: 'flex', gap: '12px', justifyContent: 'flex-end' }}>
                <button type="button" onClick={handleCloseModal} className="btn btn-secondary">
                  Отмена
                </button>
                <button type="submit" className="btn btn-primary">
                  Сохранить
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </div>
  );
};

export default Categories;
