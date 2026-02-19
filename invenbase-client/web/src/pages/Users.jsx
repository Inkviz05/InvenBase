import React, { useState, useEffect } from 'react';
import { useAuth } from '../context/AuthContext';
import { usersAPI } from '../api/users';

const Users = () => {
  const { isAdmin, user: currentUser } = useAuth();
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [showModal, setShowModal] = useState(false);
  const [editingUser, setEditingUser] = useState(null);
  const [formData, setFormData] = useState({
    username: '',
    password: '',
    full_name: '',
    email: '',
    role: 'user',
  });

  useEffect(() => {
    if (isAdmin()) {
      fetchUsers();
    } else {
      setLoading(false);
    }
  }, [isAdmin]);

  const fetchUsers = async () => {
    try {
      const data = await usersAPI.getAll();
      setUsers(data);
    } catch (error) {
      console.error('Failed to fetch users:', error);
      alert('Ошибка загрузки пользователей');
    } finally {
      setLoading(false);
    }
  };

  const getRoleText = (role) => {
    const roles = {
      admin: 'Администратор',
      responsible: 'Ответственный',
      user: 'Пользователь',
    };
    return roles[role] || role;
  };

  const handleOpenModal = (user = null) => {
    if (user) {
      setEditingUser(user);
      setFormData({
        username: user.username || '',
        password: '',
        full_name: user.full_name || '',
        email: user.email || '',
        role: user.role || 'user',
      });
    } else {
      setEditingUser(null);
      setFormData({
        username: '',
        password: '',
        full_name: '',
        email: '',
        role: 'user',
      });
    }
    setShowModal(true);
  };

  const handleCloseModal = () => {
    setShowModal(false);
    setEditingUser(null);
    setFormData({
      username: '',
      password: '',
      full_name: '',
      email: '',
      role: 'user',
    });
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    try {
      const dataToSend = { ...formData };
      // Если пароль не указан при редактировании, не отправляем его
      if (editingUser && !dataToSend.password) {
        delete dataToSend.password;
      }

      if (editingUser) {
        await usersAPI.update(editingUser.id, dataToSend);
        alert('Пользователь обновлён');
      } else {
        await usersAPI.create(dataToSend);
        alert('Пользователь создан');
      }
      handleCloseModal();
      fetchUsers();
    } catch (error) {
      console.error('Failed to save user:', error);
      alert(error.response?.data?.message || 'Ошибка сохранения пользователя');
    }
  };

  const handleDelete = async (id) => {
    if (id === currentUser?.id) {
      alert('Вы не можете удалить свой собственный аккаунт');
      return;
    }

    const user = users.find(u => u.id === id);
    if (!window.confirm(
      `Вы уверены, что хотите удалить пользователя "${user?.username}" (${user?.full_name || user?.email || 'без имени'})?\n\n` +
      `Роль: ${getRoleText(user?.role)}\n` +
      `Это действие нельзя отменить. Все связанные данные пользователя будут затронуты.`
    )) {
      return;
    }

    try {
      await usersAPI.delete(id);
      alert('Пользователь удалён');
      fetchUsers();
    } catch (error) {
      console.error('Failed to delete user:', error);
      alert(error.response?.data?.message || 'Ошибка удаления пользователя');
    }
  };

  if (!isAdmin()) {
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
        <h1>Пользователи</h1>
        <button onClick={() => handleOpenModal()} className="btn btn-primary">
          <span className="material-icons">add</span>
          Добавить пользователя
        </button>
      </div>

      <div className="card">
        <div className="table-responsive">
          <table style={{ width: '100%', borderCollapse: 'collapse' }}>
            <thead>
            <tr style={{ 
              borderBottom: '2px solid var(--divider)',
              background: 'var(--surface)'
            }}>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Имя пользователя</th>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>ФИО</th>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Email</th>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Роль</th>
              <th style={{ padding: '12px', textAlign: 'right', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Действия</th>
            </tr>
            </thead>
            <tbody>
            {users.map((user) => (
              <tr key={user.id} style={{ borderBottom: '1px solid var(--divider)' }}>
                <td style={{ padding: '12px', color: 'var(--text-primary)' }}>{user.username}</td>
                <td style={{ padding: '12px', color: 'var(--text-secondary)' }}>
                  {user.full_name || '-'}
                </td>
                <td style={{ padding: '12px', color: 'var(--text-secondary)' }}>
                  {user.email || '-'}
                </td>
                <td style={{ padding: '12px' }}>
                  <span
                    style={{
                      padding: '6px 12px',
                      borderRadius: '8px',
                      background:
                        user.role === 'admin'
                          ? 'rgba(168, 85, 247, 0.2)'
                          : user.role === 'responsible'
                          ? 'rgba(236, 72, 153, 0.2)'
                          : 'rgba(255, 255, 255, 0.05)',
                      color:
                        user.role === 'admin'
                          ? 'var(--primary-color)'
                          : user.role === 'responsible'
                          ? 'var(--secondary-color)'
                          : 'var(--text-secondary)',
                      fontSize: '13px',
                      fontWeight: 600,
                      border: `1px solid ${user.role === 'admin' ? 'rgba(168, 85, 247, 0.3)' : user.role === 'responsible' ? 'rgba(236, 72, 153, 0.3)' : 'transparent'}`,
                    }}
                  >
                    {getRoleText(user.role)}
                  </span>
                </td>
                <td style={{ padding: '12px', textAlign: 'right' }}>
                  <div style={{ display: 'flex', gap: '8px', justifyContent: 'flex-end' }}>
                    <button
                      onClick={() => handleOpenModal(user)}
                      className="btn btn-secondary"
                      style={{ fontSize: '12px', padding: '6px 12px' }}
                    >
                      <span className="material-icons" style={{ fontSize: '18px' }}>edit</span>
                      Редактировать
                    </button>
                    {user.id !== currentUser?.id && (
                      <button
                        onClick={() => handleDelete(user.id)}
                        className="btn btn-danger"
                        style={{ fontSize: '12px', padding: '6px 12px' }}
                      >
                        <span className="material-icons" style={{ fontSize: '18px' }}>delete</span>
                        Удалить
                      </button>
                    )}
                    {user.id === currentUser?.id && (
                      <span style={{ color: 'var(--text-secondary)', fontSize: '12px', padding: '8px 16px' }}>
                        Вы
                      </span>
                    )}
                  </div>
                </td>
              </tr>
            ))}
            </tbody>
          </table>
        </div>

        {users.length === 0 && (
          <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
            <span className="material-icons" style={{ fontSize: '64px', marginBottom: '16px', display: 'block' }}>
              people
            </span>
            <p>Нет пользователей</p>
          </div>
        )}
      </div>

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
                {editingUser ? 'Редактировать пользователя' : 'Добавить пользователя'}
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
                <label className="label">Имя пользователя *</label>
                <input
                  type="text"
                  className="input"
                  value={formData.username}
                  onChange={(e) => setFormData({ ...formData, username: e.target.value })}
                  required
                />
              </div>

              <div style={{ marginBottom: '16px' }}>
                <label className="label">
                  {editingUser ? 'Новый пароль (оставьте пустым, чтобы не менять)' : 'Пароль *'}
                </label>
                <input
                  type="password"
                  className="input"
                  value={formData.password}
                  onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                  required={!editingUser}
                />
              </div>

              <div style={{ marginBottom: '16px' }}>
                <label className="label">ФИО</label>
                <input
                  type="text"
                  className="input"
                  value={formData.full_name}
                  onChange={(e) => setFormData({ ...formData, full_name: e.target.value })}
                />
              </div>

              <div style={{ marginBottom: '16px' }}>
                <label className="label">Email</label>
                <input
                  type="email"
                  className="input"
                  value={formData.email}
                  onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                />
              </div>

              <div style={{ marginBottom: '24px' }}>
                <label className="label">Роль *</label>
                <select
                  className="input"
                  value={formData.role}
                  onChange={(e) => setFormData({ ...formData, role: e.target.value })}
                  required
                >
                  <option value="user">Пользователь</option>
                  <option value="responsible">Ответственный</option>
                  <option value="admin">Администратор</option>
                </select>
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

export default Users;
