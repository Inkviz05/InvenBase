import React, { useState, useEffect, useCallback } from 'react';
import { useAuth } from '../context/AuthContext';
import { reportsAPI } from '../api/reports';
import { usersAPI } from '../api/users';
import * as XLSX from 'xlsx';
import { format } from 'date-fns';

const ACTION_LABELS = {
  login: 'Вход в систему',
  create_equipment: 'Создание оборудования',
  update_equipment: 'Изменение оборудования',
  delete_equipment: 'Удаление оборудования',
  equipment_move: 'Перемещение оборудования',
  create_booking: 'Создание бронирования',
  update_booking: 'Изменение бронирования',
  approve_booking: 'Одобрение бронирования',
  reject_booking: 'Отклонение бронирования',
  create_user: 'Создание пользователя',
  update_user: 'Изменение пользователя',
  delete_user: 'Удаление пользователя',
};

const ENTITY_LABELS = {
  user: 'Пользователь',
  equipment: 'Оборудование',
  booking: 'Бронирование',
};

/** Форматирует детали записи журнала на русском; не выводит UUID, только названия. */
function formatDetails(details) {
  if (!details || typeof details !== 'object') return '—';
  // Перемещение оборудования: "Оборудование: из «место» в «место»" или с названиями сквадов
  if (details.equipment_name) {
    const from = details.from_squad_name || details.from_location || '—';
    const to = details.to_squad_name || details.to_location || '—';
    if (from !== '—' || to !== '—') {
      return `${details.equipment_name}: из «${from}» в «${to}»${details.comment ? `. ${details.comment}` : ''}`;
    }
    return details.comment ? `${details.equipment_name}. ${details.comment}` : details.equipment_name;
  }
  if (details.name) return details.name;
  if (details.status) return `Статус: ${details.status}`;
  if (details.subject) return details.subject;
  if (details.message) return details.message.length > 80 ? details.message.slice(0, 80) + '…' : details.message;
  // Не выводим сырой JSON с UUID — короткая подпись
  const keys = Object.keys(details).filter((k) => !/^[0-9a-f-]{36}$/i.test(details[k]));
  if (keys.length) return keys.map((k) => `${k}: ${String(details[k]).slice(0, 40)}`).join('; ').slice(0, 120) + (keys.length > 2 ? '…' : '');
  return '—';
}

const Logs = () => {
  const { isAdmin, isResponsible } = useAuth();
  const [entries, setEntries] = useState([]);
  const [users, setUsers] = useState([]);
  const [loading, setLoading] = useState(true);
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');
  const [actionFilter, setActionFilter] = useState('');
  const [entityTypeFilter, setEntityTypeFilter] = useState('');
  const [userIdFilter, setUserIdFilter] = useState('');
  const [limit] = useState(1000);

  const canAccess = isAdmin() || isResponsible();

  const fetchUsers = useCallback(async () => {
    if (!canAccess) return;
    try {
      const list = await usersAPI.getAll();
      setUsers(Array.isArray(list) ? list : []);
    } catch {
      setUsers([]);
    }
  }, [canAccess]);

  const fetchAudit = useCallback(async () => {
    if (!canAccess) return;
    setLoading(true);
    try {
      const params = { limit, offset: 0 };
      if (fromDate) params.from = fromDate;
      if (toDate) params.to = toDate;
      if (actionFilter) params.action = actionFilter;
      if (entityTypeFilter) params.entity_type = entityTypeFilter;
      if (userIdFilter) params.user_id = userIdFilter;
      const data = await reportsAPI.getAuditReport(params);
      setEntries(Array.isArray(data) ? data : []);
    } catch (err) {
      console.error('Failed to fetch audit report:', err);
      alert('Ошибка загрузки журнала учёта');
      setEntries([]);
    } finally {
      setLoading(false);
    }
  }, [canAccess, fromDate, toDate, actionFilter, entityTypeFilter, userIdFilter, limit]);

  useEffect(() => {
    if (canAccess) {
      fetchUsers();
      fetchAudit();
    } else {
      setLoading(false);
    }
  }, [canAccess]);

  const applyPreset = (preset) => {
    const today = new Date();
    const fmt = (d) => d.toISOString().slice(0, 10);
    if (preset === 'today') {
      setFromDate(fmt(today));
      setToDate(fmt(today));
    } else if (preset === 'week') {
      const d = new Date(today);
      d.setDate(d.getDate() - 6);
      setFromDate(fmt(d));
      setToDate(fmt(today));
    } else if (preset === 'month') {
      setFromDate(fmt(new Date(today.getFullYear(), today.getMonth(), 1)));
      setToDate(fmt(today));
    } else {
      setFromDate('');
      setToDate('');
    }
  };

  const exportXlsx = () => {
    const headers = ['Дата и время', 'Пользователь', 'Действие', 'Тип объекта', 'Наименование объекта', 'Детали'];
    const rows = entries.map((e) => [
      e.created_at ? format(new Date(e.created_at), 'dd.MM.yyyy HH:mm:ss') : '',
      (e.full_name && e.full_name.trim()) ? e.full_name.trim() : (e.username || '—'),
      ACTION_LABELS[e.action] || e.action,
      ENTITY_LABELS[e.entity_type] || e.entity_type,
      (e.entity_name && e.entity_name.trim()) ? e.entity_name.trim() : '—',
      formatDetails(e.details),
    ]);
    const ws = XLSX.utils.aoa_to_sheet([headers, ...rows]);
    const wb = XLSX.utils.book_new();
    XLSX.utils.book_append_sheet(wb, ws, 'Журнал учёта');
    const name = fromDate || toDate ? `audit_${fromDate || '...'}_${toDate || '...'}.xlsx` : 'audit_report.xlsx';
    XLSX.writeFile(wb, name);
  };

  if (!canAccess) {
    return (
      <div className="reports-page">
        <h1 className="reports-page-title">Журнал</h1>
        <div className="error-message">У вас нет прав для доступа к журналу.</div>
      </div>
    );
  }

  return (
    <div className="reports-page">
      <h1 className="reports-page-title" style={{ marginBottom: 0 }}>Журнал действий</h1>
      <p style={{ color: 'var(--text-secondary)', marginTop: '8px', marginBottom: '24px' }}>
        Все действия в системе: оборудование, бронирования, пользователи, вход, перемещения. Выгрузка в XLSX по выбранным фильтрам.
      </p>

      <div className="card report-card" style={{ padding: '20px 24px' }}>
        <div className="report-card-header" style={{ display: 'flex', flexWrap: 'wrap', gap: '12px', marginBottom: '20px', alignItems: 'flex-end' }}>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: '12px', alignItems: 'flex-end', flex: '1 1 200px' }}>
            <label style={{ display: 'flex', flexDirection: 'column', gap: '4px', fontSize: '12px', color: 'var(--text-secondary)' }}>
              Период с
              <input
                type="date"
                value={fromDate}
                onChange={(e) => setFromDate(e.target.value)}
                style={{ padding: '8px 12px', borderRadius: '8px', border: '1px solid var(--divider)', background: 'var(--surface)', color: 'var(--text-primary)' }}
              />
            </label>
            <label style={{ display: 'flex', flexDirection: 'column', gap: '4px', fontSize: '12px', color: 'var(--text-secondary)' }}>
              по
              <input
                type="date"
                value={toDate}
                onChange={(e) => setToDate(e.target.value)}
                style={{ padding: '8px 12px', borderRadius: '8px', border: '1px solid var(--divider)', background: 'var(--surface)', color: 'var(--text-primary)' }}
              />
            </label>
            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
              <button type="button" className="btn btn-secondary" onClick={() => applyPreset('today')}>Сегодня</button>
              <button type="button" className="btn btn-secondary" onClick={() => applyPreset('week')}>7 дней</button>
              <button type="button" className="btn btn-secondary" onClick={() => applyPreset('month')}>Месяц</button>
              <button type="button" className="btn btn-secondary" onClick={() => applyPreset('all')}>Всё время</button>
            </div>
            <label style={{ display: 'flex', flexDirection: 'column', gap: '4px', fontSize: '12px', color: 'var(--text-secondary)' }}>
              Действие
              <select
                value={actionFilter}
                onChange={(e) => setActionFilter(e.target.value)}
                style={{ padding: '8px 12px', borderRadius: '8px', border: '1px solid var(--divider)', background: 'var(--surface)', color: 'var(--text-primary)', minWidth: '180px' }}
              >
                <option value="">Все</option>
                {Object.entries(ACTION_LABELS).map(([k, v]) => (
                  <option key={k} value={k}>{v}</option>
                ))}
              </select>
            </label>
            <label style={{ display: 'flex', flexDirection: 'column', gap: '4px', fontSize: '12px', color: 'var(--text-secondary)' }}>
              Тип объекта
              <select
                value={entityTypeFilter}
                onChange={(e) => setEntityTypeFilter(e.target.value)}
                style={{ padding: '8px 12px', borderRadius: '8px', border: '1px solid var(--divider)', background: 'var(--surface)', color: 'var(--text-primary)' }}
              >
                <option value="">Все</option>
                {Object.entries(ENTITY_LABELS).map(([k, v]) => (
                  <option key={k} value={k}>{v}</option>
                ))}
              </select>
            </label>
            <label style={{ display: 'flex', flexDirection: 'column', gap: '4px', fontSize: '12px', color: 'var(--text-secondary)' }}>
              Пользователь
              <select
                value={userIdFilter}
                onChange={(e) => setUserIdFilter(e.target.value)}
                style={{ padding: '8px 12px', borderRadius: '8px', border: '1px solid var(--divider)', background: 'var(--surface)', color: 'var(--text-primary)', minWidth: '160px' }}
              >
                <option value="">Все</option>
                {users.map((u) => (
                  <option key={u.id} value={u.id}>{u.username || u.full_name || u.id}</option>
                ))}
              </select>
            </label>
          </div>
          <div style={{ display: 'flex', gap: '8px', flexShrink: 0 }}>
            <button type="button" className="btn btn-primary" onClick={fetchAudit} disabled={loading}>
              <span className="material-icons" style={{ marginRight: '6px' }}>search</span>
              Применить
            </button>
            <button type="button" className="btn btn-secondary" onClick={exportXlsx} disabled={!entries.length}>
              <span className="material-icons" style={{ marginRight: '6px' }}>download</span>
              Выгрузить XLSX
            </button>
          </div>
        </div>

        <div className="report-card-body" style={{ minWidth: 0, maxWidth: '100%' }}>
          {loading ? (
            <div className="loading"><div className="spinner"></div></div>
          ) : (
            <>
              <p style={{ marginBottom: '12px', color: 'var(--text-secondary)', fontSize: '14px' }}>
                Записей: <strong style={{ color: 'var(--text-primary)' }}>{entries.length}</strong>
                {entries.length >= limit && ` (показано не более ${limit})`}
              </p>
              <div className="table-report-wrap table-responsive">
                <table className="table-report">
                  <thead>
                    <tr>
                      <th className="col-num">№</th>
                      <th>Дата и время</th>
                      <th>Пользователь</th>
                      <th>Действие</th>
                      <th>Тип объекта</th>
                      <th>Объект</th>
                      <th>Детали</th>
                    </tr>
                  </thead>
                  <tbody>
                    {entries.map((e, index) => (
                      <tr key={e.id}>
                        <td className="col-num">{index + 1}</td>
                        <td style={{ whiteSpace: 'nowrap' }}>
                          {e.created_at ? format(new Date(e.created_at), 'dd.MM.yyyy HH:mm:ss') : '—'}
                        </td>
                        <td>{(e.full_name && e.full_name.trim()) ? e.full_name.trim() : (e.username || '—')}</td>
                        <td>{ACTION_LABELS[e.action] || e.action}</td>
                        <td>{ENTITY_LABELS[e.entity_type] || e.entity_type}</td>
                        <td>{(e.entity_name && e.entity_name.trim()) ? e.entity_name.trim() : '—'}</td>
                        <td style={{ maxWidth: '320px', overflow: 'hidden', textOverflow: 'ellipsis' }} title={formatDetails(e.details)}>
                          {formatDetails(e.details)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
              {!entries.length && (
                <p style={{ textAlign: 'center', color: 'var(--text-secondary)', padding: '24px' }}>
                  Нет записей за выбранный период и фильтры.
                </p>
              )}
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default Logs;
