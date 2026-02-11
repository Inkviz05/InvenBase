import React, { useState, useEffect } from 'react';
import { Link } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { squadsAPI } from '../api/squads';
import { usersAPI } from '../api/users';

const Squads = () => {
  const { isAdmin, isResponsible } = useAuth();
  const [squads, setSquads] = useState([]);
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [editingSquad, setEditingSquad] = useState(null);
  const [formData, setFormData] = useState({ 
    name: '', 
    description: '', 
    location: '',
    responsible_user_id: '' 
  });
  const [deleteTarget, setDeleteTarget] = useState(null);
  const [deleteConfirmName, setDeleteConfirmName] = useState('');

  useEffect(() => {
    if (isAdmin() || isResponsible()) {
      fetchSquads();
      fetchUsers();
    } else {
      setLoading(false);
    }
  }, [isAdmin, isResponsible]);

  const fetchSquads = async () => {
    try {
      const data = await squadsAPI.getAll();
      setSquads(data);
    } catch (error) {
      console.error('Failed to fetch squads:', error);
      alert('Ошибка загрузки сквадов');
    } finally {
      setLoading(false);
    }
  };

  const fetchUsers = async () => {
    try {
      const data = await usersAPI.getAll();
      setUsers(data);
    } catch (error) {
      console.error('Failed to fetch users:', error);
    }
  };

  const handleOpenModal = (squad = null) => {
    if (squad) {
      setEditingSquad(squad);
      setFormData({ 
        name: squad.name || '', 
        description: squad.description || '', 
        location: squad.location || '',
        responsible_user_id: squad.responsible_user_id || '' 
      });
    } else {
      setEditingSquad(null);
      setFormData({ name: '', description: '', location: '', responsible_user_id: '' });
    }
    setShowModal(true);
  };

  const handleCloseModal = () => {
    setShowModal(false);
    setEditingSquad(null);
    setFormData({ name: '', description: '', location: '', responsible_user_id: '' });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      const submitData = {
        name: formData.name,
        description: formData.description || null,
        location: formData.location || null,
        responsible_user_id: formData.responsible_user_id || null,
      };
      if (editingSquad) {
        await squadsAPI.update(editingSquad.id, submitData);
        alert('Сквад обновлён');
      } else {
        await squadsAPI.create(submitData);
        alert('Сквад создан');
      }
      handleCloseModal();
      fetchSquads();
    } catch (error) {
      console.error('Failed to save squad:', error);
      alert(error.response?.data?.message || 'Ошибка сохранения сквада');
    }
  };

  const openDeleteModal = (squad) => {
    if (squad) {
      setDeleteTarget(squad);
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
      alert('Название сквада не совпадает. Удаление отменено.');
      return;
    }
    const id = deleteTarget.id;
    closeDeleteModal();
    try {
      await squadsAPI.delete(id);
      alert('Сквад удалён');
      fetchSquads();
    } catch (error) {
      console.error('Failed to delete squad:', error);
      alert(error.response?.data?.message || 'Ошибка удаления сквада. Возможно, в скваде есть оборудование.');
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
        <h1>Сквады</h1>
        <button onClick={() => handleOpenModal()} className="btn btn-primary">
          <span className="material-icons">add</span>
          Добавить сквад
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
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Кабинет</th>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Ответственный</th>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Описание</th>
              <th style={{ padding: '12px', textAlign: 'right', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Действия</th>
            </tr>
          </thead>
          <tbody>
            {squads.map((squad) => (
              <tr key={squad.id} style={{ borderBottom: '1px solid var(--divider)' }}>
                <td style={{ padding: '12px', color: 'var(--text-primary)' }}>
                  <Link to={`/squads/${squad.id}`} style={{ color: 'var(--primary-color)', textDecoration: 'none' }}>{squad.name}</Link>
                </td>
                <td style={{ padding: '12px', color: 'var(--text-secondary)' }}>
                  {squad.location || '-'}
                </td>
                <td style={{ padding: '12px', color: 'var(--text-secondary)' }}>
                  {squad.responsible_name || '-'}
                </td>
                <td style={{ padding: '12px', color: 'var(--text-secondary)' }}>
                  {squad.description || '-'}
                </td>
                <td style={{ padding: '12px', textAlign: 'right' }}>
                  <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
                    <button
                      onClick={() => handleOpenModal(squad)}
                      className="btn btn-secondary"
                      style={{ fontSize: '12px', padding: '6px 12px' }}
                    >
                      <span className="material-icons" style={{ fontSize: '18px' }}>edit</span>
                      Редактировать
                    </button>
                    {(isAdmin() || isResponsible()) && (
                      <button
                        onClick={() => openDeleteModal(squad)}
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

        {squads.length === 0 && (
          <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
            <span className="material-icons" style={{ fontSize: '64px', marginBottom: '16px', display: 'block' }}>
              groups
            </span>
            <p>Нет сквадов</p>
          </div>
        )}
      </div>

      {/* Модальное окно подтверждения удаления */}
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
            <h3 style={{ marginTop: 0 }}>Удаление сквада</h3>
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
                {editingSquad ? 'Редактировать сквад' : 'Добавить сквад'}
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

              <div style={{ marginBottom: '16px' }}>
                <label className="label">Кабинет/Локация</label>
                <input
                  type="text"
                  className="input"
                  value={formData.location}
                  onChange={(e) => setFormData({ ...formData, location: e.target.value })}
                  placeholder="Например: Кабинет 227"
                />
              </div>

              <div style={{ marginBottom: '16px' }}>
                <label className="label">Ответственный</label>
                <select
                  className="input"
                  value={formData.responsible_user_id}
                  onChange={(e) => setFormData({ ...formData, responsible_user_id: e.target.value })}
                >
                  <option value="">Не назначен</option>
                  {users.map((user) => (
                    <option key={user.id} value={user.id}>
                      {user.full_name || user.username} ({user.role === 'admin' ? 'Администратор' : user.role === 'responsible' ? 'Ответственный' : 'Пользователь'})
                    </option>
                  ))}
                </select>
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

export default Squads;
