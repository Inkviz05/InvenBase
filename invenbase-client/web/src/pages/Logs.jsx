import React, { useState, useEffect } from 'react';
import { useAuth } from '../context/AuthContext';
import { logsAPI } from '../api/logs';
import { format } from 'date-fns';

const Logs = () => {
  const { isAdmin, isResponsible } = useAuth();
  const [logs, setLogs] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    if (isAdmin() || isResponsible()) {
      fetchLogs();
    } else {
      setLoading(false);
    }
  }, [isAdmin, isResponsible]);

  const fetchLogs = async () => {
    try {
      const data = await logsAPI.getAll({ limit: 100 });
      setLogs(data);
    } catch (error) {
      console.error('Failed to fetch logs:', error);
      alert('Ошибка загрузки журнала');
    } finally {
      setLoading(false);
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
        <h1>Журнал действий</h1>
        <button onClick={fetchLogs} className="btn btn-secondary">
          <span className="material-icons">refresh</span>
          Обновить
        </button>
      </div>

      <div className="card">
        <table style={{ width: '100%', borderCollapse: 'collapse' }}>
          <thead>
            <tr style={{ 
              borderBottom: '2px solid var(--divider)',
              background: 'var(--surface)'
            }}>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Время</th>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Действие</th>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Тип объекта</th>
              <th style={{ padding: '12px', textAlign: 'left', color: 'var(--text-primary)', fontWeight: 600, textTransform: 'uppercase', fontSize: '12px', letterSpacing: '0.5px' }}>Детали</th>
            </tr>
          </thead>
          <tbody>
            {logs.map((log, index) => (
              <tr key={index} style={{ borderBottom: '1px solid var(--divider)' }}>
                <td style={{ padding: '12px', color: 'var(--text-secondary)', fontSize: '14px' }}>
                  {format(new Date(log.created_at), 'dd.MM.yyyy HH:mm:ss')}
                </td>
                <td style={{ padding: '12px', color: 'var(--text-primary)' }}>{log.action}</td>
                <td style={{ padding: '12px', color: 'var(--text-secondary)' }}>{log.entity_type}</td>
                <td style={{ padding: '12px', color: 'var(--text-secondary)', fontSize: '14px' }}>
                  {log.details ? (
                    <pre style={{ margin: 0, whiteSpace: 'pre-wrap', fontFamily: 'inherit', color: 'var(--text-secondary)' }}>
                      {JSON.stringify(log.details, null, 2)}
                    </pre>
                  ) : (
                    '-'
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>

        {logs.length === 0 && (
          <div style={{ textAlign: 'center', padding: '40px', color: 'var(--text-secondary)' }}>
            <span className="material-icons" style={{ fontSize: '64px', marginBottom: '16px', display: 'block' }}>
              description
            </span>
            <p>Нет записей в журнале</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default Logs;
