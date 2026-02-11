import React, { useState, useEffect } from 'react';
import { useParams, Link, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { squadsAPI } from '../api/squads';
import { categoriesAPI } from '../api/categories';

const SquadDetail = () => {
  const { id } = useParams();
  const navigate = useNavigate();
  const { isAdmin, isResponsible } = useAuth();
  const [squad, setSquad] = useState(null);
  const [equipment, setEquipment] = useState([]);
  const [categories, setCategories] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchData();
  }, [id]);

  const fetchData = async () => {
    setLoading(true);
    try {
      const [squadData, equipmentData, categoriesData] = await Promise.all([
        squadsAPI.getById(id),
        squadsAPI.getEquipment(id),
        categoriesAPI.getAll({ squad_id: id }),
      ]);
      setSquad(squadData);
      setEquipment(equipmentData);
      setCategories(categoriesData.filter((c) => c.squad_id === id));
    } catch (error) {
      console.error('Failed to fetch squad:', error);
      setSquad(null);
    } finally {
      setLoading(false);
    }
  };

  if (loading) {
    return <div className="loading"><div className="spinner"></div></div>;
  }

  if (!squad) {
    return (
      <div>
        <p className="error-message">Сквад не найден</p>
        <button onClick={() => navigate('/squads')} className="btn btn-secondary">К списку сквадов</button>
      </div>
    );
  }

  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: '16px', marginBottom: '24px' }}>
        <button onClick={() => navigate('/squads')} className="btn btn-secondary">
          <span className="material-icons">arrow_back</span>
          Назад
        </button>
        {(isAdmin() || isResponsible()) && (
          <Link to="/squads" className="btn btn-primary" style={{ textDecoration: 'none' }}>
            <span className="material-icons">list</span>
            К списку сквадов
          </Link>
        )}
      </div>

      <div className="card" style={{ marginBottom: '24px' }}>
        <h1 style={{ marginTop: 0 }}>{squad.name}</h1>
        {squad.location && (
          <p style={{ color: 'var(--text-secondary)', marginBottom: '8px' }}>
            <span className="material-icons" style={{ verticalAlign: 'middle', marginRight: '8px' }}>location_on</span>
            {squad.location}
          </p>
        )}
        {squad.responsible_name && (
          <p style={{ color: 'var(--text-secondary)', marginBottom: '8px' }}>
            <span className="material-icons" style={{ verticalAlign: 'middle', marginRight: '8px' }}>person</span>
            Ответственный: {squad.responsible_name}
          </p>
        )}
        {squad.description && (
          <p style={{ color: 'var(--text-secondary)' }}>{squad.description}</p>
        )}
      </div>

      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(400px, 1fr))', gap: '24px' }}>
        <div className="card">
          <h2 style={{ marginTop: 0, marginBottom: '16px', fontSize: '18px' }}>
            <span className="material-icons" style={{ verticalAlign: 'middle', marginRight: '8px' }}>inventory</span>
            Оборудование ({equipment.length})
          </h2>
          {(isAdmin() || isResponsible()) && (
            <Link to={`/equipment/create?squad_id=${id}`} className="btn btn-primary" style={{ marginBottom: '16px', display: 'inline-flex', textDecoration: 'none' }}>
              <span className="material-icons">add</span>
              Добавить оборудование
            </Link>
          )}
          <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
            {equipment.length === 0 ? (
              <li style={{ color: 'var(--text-secondary)' }}>Нет оборудования в этом скваде</li>
            ) : (
              equipment.map((eq) => (
                <li key={eq.id} style={{ padding: '8px 0', borderBottom: '1px solid var(--divider)' }}>
                  <Link to={`/equipment/${eq.id}`} style={{ color: 'var(--primary-color)', textDecoration: 'none' }}>
                    {eq.name}
                  </Link>
                  {eq.quantity > 0 && (
                    <span style={{ color: 'var(--text-secondary)', fontSize: '14px', marginLeft: '8px' }}>
                      — {eq.available_quantity}/{eq.quantity}
                    </span>
                  )}
                </li>
              ))
            )}
          </ul>
        </div>

        <div className="card">
          <h2 style={{ marginTop: 0, marginBottom: '16px', fontSize: '18px' }}>
            <span className="material-icons" style={{ verticalAlign: 'middle', marginRight: '8px' }}>category</span>
            Категории сквада ({categories.length})
          </h2>
          <ul style={{ listStyle: 'none', padding: 0, margin: 0 }}>
            {categories.length === 0 ? (
              <li style={{ color: 'var(--text-secondary)' }}>Нет категорий, привязанных к этому скваду</li>
            ) : (
              categories.map((cat) => (
                <li key={cat.id} style={{ padding: '8px 0', borderBottom: '1px solid var(--divider)' }}>
                  {cat.name}
                  {cat.description && (
                    <span style={{ color: 'var(--text-secondary)', fontSize: '14px', marginLeft: '8px' }}>
                      — {cat.description}
                    </span>
                  )}
                </li>
              ))
            )}
          </ul>
        </div>
      </div>
    </div>
  );
};

export default SquadDetail;
