import React, { useState, useEffect } from 'react';
import { useAuth } from '../context/AuthContext';
import { reportsAPI } from '../api/reports';
import { bookingsAPI } from '../api/bookings';
import { equipmentAPI } from '../api/equipment';
import { squadsAPI } from '../api/squads';
import * as XLSX from 'xlsx';

const Reports = () => {
  const { isAdmin, isResponsible } = useAuth();
  const [equipmentReport, setEquipmentReport] = useState(null);
  const [bookingReport, setBookingReport] = useState(null);
  const [loading, setLoading] = useState(true);
  const [allBookings, setAllBookings] = useState([]);
  const [allEquipment, setAllEquipment] = useState([]);
  const [fromDate, setFromDate] = useState('');
  const [toDate, setToDate] = useState('');
  const [periodPreset, setPeriodPreset] = useState('all');

  useEffect(() => {
    if (isAdmin() || isResponsible()) {
      fetchReports();
    } else {
      setLoading(false);
    }
  }, [isAdmin, isResponsible]);

  const fetchReports = async () => {
    try {
      const [equipmentData, bookingData, bookingsData, equipmentDataFull] = await Promise.all([
        reportsAPI.getEquipmentReport(),
        reportsAPI.getBookingReport(),
        bookingsAPI.getAll(),
        equipmentAPI.getAll(),
      ]);
      setEquipmentReport(equipmentData);
      setBookingReport(bookingData);
      setAllBookings(bookingsData || []);
      setAllEquipment(equipmentDataFull || []);
    } catch (error) {
      console.error('Failed to fetch reports:', error);
      alert('Ошибка загрузки отчётов');
    } finally {
      setLoading(false);
    }
  };

  const bookingSummary = bookingReport || {
    total: 0, pending: 0, approved: 0, rejected: 0, expired: 0, awaiting_return: 0, returned: 0, cancelled: 0, completed: 0,
  };

  const getFilteredBookingsByPeriod = () => {
    if (!allBookings?.length) return [];
    return allBookings.filter((b) => {
      if (!b.start_date) return false;
      const iso = new Date(b.start_date).toISOString().slice(0, 10);
      if (fromDate && iso < fromDate) return false;
      if (toDate && iso > toDate) return false;
      return true;
    });
  };

  const computeUsageStats = () => {
    const bookings = getFilteredBookingsByPeriod();
    if (!bookings.length) return { equipmentUsage: [], categoryUsage: [], totalEquipmentBookings: 0, totalCategoryBookings: 0 };

    const categoryNameById = {};
    (equipmentReport?.by_category || []).forEach((c) => { categoryNameById[c.category_id] = c.category_name; });

    const equipmentById = {};
    (allEquipment || []).forEach((eq) => { equipmentById[eq.id] = eq; });

    const equipmentCount = {};
    const categoryCount = {};
    bookings.forEach((b) => {
      if (b.equipment_id) {
        const eq = equipmentById[b.equipment_id] || {};
        const name = b.equipment_name || eq.name || '—';
        const catId = eq.category_id;
        const catName = (catId && categoryNameById[catId]) || 'Без категории';
        equipmentCount[b.equipment_id] = equipmentCount[b.equipment_id] || { equipmentId: b.equipment_id, equipmentName: name, categoryId: catId, categoryName: catName, bookingsCount: 0, totalQuantity: 0 };
        equipmentCount[b.equipment_id].bookingsCount += 1;
        equipmentCount[b.equipment_id].totalQuantity += b.quantity || 0;
        categoryCount[catName] = categoryCount[catName] || { categoryName: catName, bookingsCount: 0 };
        categoryCount[catName].bookingsCount += 1;
      }
    });

    const equipmentUsage = Object.values(equipmentCount)
      .map((o) => ({ ...o, totalQuantity: o.totalQuantity }))
      .sort((a, b) => b.bookingsCount - a.bookingsCount);
    const categoryUsage = Object.values(categoryCount).sort((a, b) => b.bookingsCount - a.bookingsCount);
    const totalEquipmentBookings = equipmentUsage.reduce((s, i) => s + i.bookingsCount, 0);
    const totalCategoryBookings = categoryUsage.reduce((s, i) => s + i.bookingsCount, 0);

    return { equipmentUsage, categoryUsage, totalEquipmentBookings, totalCategoryBookings };
  };

  const { equipmentUsage = [], categoryUsage = [], totalEquipmentBookings = 0, totalCategoryBookings = 0 } = computeUsageStats();

  const applyPeriodPreset = (preset) => {
    setPeriodPreset(preset);
    const today = new Date();
    const fmt = (d) => d.toISOString().slice(0, 10);
    if (preset === 'all') { setFromDate(''); setToDate(''); }
    else if (preset === 'today') { setFromDate(fmt(today)); setToDate(fmt(today)); }
    else if (preset === 'yesterday') { const d = new Date(today); d.setDate(d.getDate() - 1); const f = fmt(d); setFromDate(f); setToDate(f); }
    else if (preset === 'week') { const d = new Date(today); d.setDate(d.getDate() - 6); setFromDate(fmt(d)); setToDate(fmt(today)); }
    else if (preset === 'month') { setFromDate(fmt(new Date(today.getFullYear(), today.getMonth(), 1))); setToDate(fmt(today)); }
  };

  /** Один отчёт XLSX: выборка по периоду, понятные листы и подписи */
  const exportReportXlsx = async () => {
    try {
      const periodStr = fromDate && toDate ? `с ${fromDate} по ${toDate}` : (fromDate || toDate) ? `с ${fromDate || '…'} по ${toDate || '…'}` : 'за всё время';
      const wb = XLSX.utils.book_new();
      const filteredBookings = getFilteredBookingsByPeriod();

      let movementsList = [];
      try {
        const data = await reportsAPI.getAuditReport({ from: fromDate || undefined, to: toDate || undefined, action: 'equipment_move', limit: 10000 });
        movementsList = Array.isArray(data) ? data : [];
      } catch { movementsList = []; }

      let squadIdToName = {};
      try {
        const squads = await squadsAPI.getAll();
        (squads || []).forEach((s) => { squadIdToName[s.id] = s.name || '—'; });
      } catch { squadIdToName = {}; }

      const catById = {};
      (equipmentReport?.by_category || []).forEach((c) => { catById[c.category_id] = c.category_name || 'Без категории'; });
      const equipmentById = {};
      (allEquipment || []).forEach((eq) => { equipmentById[eq.id] = eq; });
      const statusRu = { pending: 'Ожидают одобрения', approved: 'Одобрены', rejected: 'Отклонены', awaiting_return: 'Ожидают возврата', returned: 'Возвращены', cancelled: 'Отменены', completed: 'Завершены', expired: 'Истекли' };
      const permRu = { internal: 'Внутренний', external: 'Внешний' };
      const statusEqRu = { available: 'Доступно', maintenance: 'На обслуживании' };

      // Лист 1 — Сводка (одна таблица: Показатель | Значение)
      XLSX.utils.book_append_sheet(wb, XLSX.utils.aoa_to_sheet([
        ['Сводка по системе'],
        ['Период выборки по бронированиям и перемещениям', periodStr],
        [],
        ['Показатель', 'Значение'],
        ['——— Оборудование ———', ''],
        ['Всего позиций оборудования', equipmentReport?.total_equipment ?? 0],
        ['Доступно единиц', equipmentReport?.available_equipment ?? 0],
        ['Сейчас в бронировании', equipmentReport?.booked_equipment ?? 0],
        ['——— Бронирования (вся система) ———', ''],
        ['Всего заявок на бронирование', bookingSummary.total ?? 0],
        ['Ожидают одобрения', bookingSummary.pending ?? 0],
        ['Одобрены (активные)', bookingSummary.approved ?? 0],
        ['Ожидают возврата', bookingSummary.awaiting_return ?? 0],
        ['Возвращены', bookingSummary.returned ?? 0],
        ['Отклонены', bookingSummary.rejected ?? 0],
        ['Истекли', bookingSummary.expired ?? 0],
        ['Отменены', bookingSummary.cancelled ?? 0],
        ['Завершены', bookingSummary.completed ?? 0],
        ['——— За выбранный период ———', ''],
        ['Бронирований за период', filteredBookings.length],
        ['Перемещений оборудования за период', movementsList.length],
      ]), 'Сводка');

      // Лист 2 — Оборудование (справочник)
      XLSX.utils.book_append_sheet(wb, XLSX.utils.aoa_to_sheet([
        ['Справочник оборудования (текущее состояние)'],
        [],
        ['№', 'Название оборудования', 'Категория', 'Всего, шт.', 'Доступно, шт.', 'В бронировании, шт.', 'Статус', 'Местоположение'],
        ...(allEquipment || []).map((eq, i) => {
          const total = eq.quantity ?? 0;
          const available = eq.available_quantity ?? 0;
          const booked = Math.max(0, total - available);
          return [i + 1, eq.name || '—', catById[eq.category_id] || 'Без категории', total, available, booked, statusEqRu[eq.status] || eq.status || '—', eq.location || '—'];
        }),
      ]), 'Оборудование');

      // Лист 3 — Категории
      XLSX.utils.book_append_sheet(wb, XLSX.utils.aoa_to_sheet([
        ['Оборудование по категориям'],
        [],
        ['Категория', 'Всего, шт.', 'Доступно, шт.', 'В бронировании, шт.'],
        ...(equipmentReport?.by_category || []).map((c) => [c.category_name || 'Без категории', c.total ?? 0, c.available ?? 0, c.booked ?? 0]),
      ]), 'Категории');

      // Лист 4 — Бронирования за период
      const bookingRows = filteredBookings.filter((b) => b.equipment_id).map((b) => {
        const eq = equipmentById[b.equipment_id] || {};
        const catName = (eq.category_id && catById[eq.category_id]) || 'Без категории';
        return [
          b.equipment_name || eq.name || '—',
          catName,
          (b.full_name && b.full_name.trim()) ? b.full_name.trim() : (b.username || '—'),
          statusRu[b.status] || b.status || '—',
          permRu[b.permission_type] || b.permission_type || '—',
          b.quantity ?? 0,
          b.purpose || '—',
          b.start_date ? new Date(b.start_date).toLocaleString('ru-RU', { dateStyle: 'short', timeStyle: 'short' }) : '—',
          b.end_date ? new Date(b.end_date).toLocaleString('ru-RU', { dateStyle: 'short', timeStyle: 'short' }) : '—',
        ];
      });
      XLSX.utils.book_append_sheet(wb, XLSX.utils.aoa_to_sheet([
        ['Бронирования за период'],
        ['Период', periodStr],
        [],
        ['Оборудование', 'Категория', 'Пользователь', 'Статус заявки', 'Тип доступа', 'Кол-во, шт.', 'Цель использования', 'Дата и время начала', 'Дата и время окончания'],
        ...bookingRows,
      ]), 'Бронирования');

      // Лист 5 — Перемещения (только названия, без ID/UUID)
      const moveRows = movementsList.map((e) => {
        const d = e.details || {};
        const from = d.from_location || d.from_squad_name || (d.from_squad_id && squadIdToName[d.from_squad_id]) || '—';
        const to = d.to_location || d.to_squad_name || (d.to_squad_id && squadIdToName[d.to_squad_id]) || '—';
        return [
          e.created_at ? new Date(e.created_at).toLocaleString('ru-RU', { dateStyle: 'short', timeStyle: 'short' }) : '—',
          (e.full_name && e.full_name.trim()) ? e.full_name.trim() : (e.username || '—'),
          d.equipment_name || '—',
          from,
          to,
          d.comment || '—',
        ];
      });
      XLSX.utils.book_append_sheet(wb, XLSX.utils.aoa_to_sheet([
        ['Перемещения оборудования за период'],
        ['Период', periodStr],
        [],
        ['Дата и время', 'Кто выполнил', 'Название оборудования', 'Откуда (место/сквад)', 'Куда (место/сквад)', 'Комментарий'],
        ...moveRows,
      ]), 'Перемещения');

      const fname = `otchet_${fromDate || 'vse'}_${toDate || 'vse'}.xlsx`.replace(/\s/g, '_');
      XLSX.writeFile(wb, fname);
    } catch (e) {
      console.error(e);
      alert('Ошибка выгрузки отчёта');
    }
  };

  if (!isAdmin() && !isResponsible()) {
    return (
      <div className="reports-page">
        <h1 className="reports-page-title">Отчёты</h1>
        <div className="error-message">У вас нет прав для доступа к отчётам.</div>
      </div>
    );
  }

  if (loading) return <div className="reports-page"><div className="loading"><div className="spinner"></div></div></div>;

  return (
    <div className="reports-page">
      <h1 className="reports-page-title" style={{ marginBottom: 0 }}>Отчёты</h1>
      <p style={{ color: 'var(--text-secondary)', marginTop: '8px', marginBottom: '16px' }}>
        Один файл XLSX: сводка, оборудование, категории, бронирования и перемещения за выбранный период.
      </p>
      <div className="card report-filters" style={{ padding: '16px 20px', marginBottom: '24px' }}>
        <div style={{ display: 'flex', flexWrap: 'wrap', alignItems: 'flex-end', gap: '12px' }}>
          <label style={{ display: 'flex', flexDirection: 'column', gap: '4px', fontSize: '12px', color: 'var(--text-secondary)' }}>
            Период отчёта: с
            <input type="date" value={fromDate} onChange={(e) => { setPeriodPreset('custom'); setFromDate(e.target.value); }} style={{ padding: '8px', borderRadius: '8px', border: '1px solid var(--divider)', background: 'var(--surface)', color: 'var(--text-primary)' }} />
          </label>
          <label style={{ display: 'flex', flexDirection: 'column', gap: '4px', fontSize: '12px', color: 'var(--text-secondary)' }}>
            по
            <input type="date" value={toDate} onChange={(e) => { setPeriodPreset('custom'); setToDate(e.target.value); }} style={{ padding: '8px', borderRadius: '8px', border: '1px solid var(--divider)', background: 'var(--surface)', color: 'var(--text-primary)' }} />
          </label>
          <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
            <button type="button" className="btn btn-secondary" onClick={() => applyPeriodPreset('today')}>Сегодня</button>
            <button type="button" className="btn btn-secondary" onClick={() => applyPeriodPreset('week')}>7 дней</button>
            <button type="button" className="btn btn-secondary" onClick={() => applyPeriodPreset('month')}>Месяц</button>
            <button type="button" className="btn btn-secondary" onClick={() => applyPeriodPreset('all')}>Всё время</button>
          </div>
          <button type="button" className="btn btn-primary" onClick={exportReportXlsx}>
            <span className="material-icons" style={{ marginRight: '6px' }}>download</span>
            Выгрузить отчёт (XLSX)
          </button>
        </div>
      </div>

      <div className="reports-page-inner" style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
        {/* Блок: отчёт по оборудованию */}
        <div className="card report-card" style={{ padding: '20px 24px' }}>
          <div className="report-card-header" style={{ marginBottom: '20px' }}>
            <h2 style={{ margin: 0 }}>Оборудование</h2>
            <p style={{ margin: '4px 0 0', color: 'var(--text-secondary)', fontSize: '13px' }}>Наличие и использование.</p>
          </div>
          {equipmentReport && (
            <div className="report-card-body" style={{ display: 'grid', gap: '20px', minWidth: 0, maxWidth: '100%' }}>
              <div className="report-stats-grid" style={{ display: 'grid', gap: '16px' }}>
                <div className="card" style={{ padding: '16px' }}>
                  <div style={{ fontSize: '11px', color: 'var(--text-secondary)', textTransform: 'uppercase', marginBottom: '4px' }}>Всего позиций</div>
                  <div style={{ fontSize: '28px', fontWeight: 'bold' }}>{equipmentReport.total_equipment ?? 0}</div>
                </div>
                <div className="card" style={{ padding: '16px' }}>
                  <div style={{ fontSize: '11px', color: 'var(--text-secondary)', textTransform: 'uppercase', marginBottom: '4px' }}>Доступно</div>
                  <div style={{ fontSize: '28px', fontWeight: 'bold', color: 'var(--success)' }}>{equipmentReport.available_equipment ?? 0}</div>
                </div>
                <div className="card" style={{ padding: '16px' }}>
                  <div style={{ fontSize: '11px', color: 'var(--text-secondary)', textTransform: 'uppercase', marginBottom: '4px' }}>В бронировании</div>
                  <div style={{ fontSize: '28px', fontWeight: 'bold', color: 'var(--warning)' }}>{equipmentReport.booked_equipment ?? 0}</div>
                </div>
              </div>
              {equipmentReport.by_category?.length > 0 && (
                <div className="report-section" style={{ marginTop: '8px' }}>
                  <h3 className="report-section-title">По категориям</h3>
                  <div style={{ display: 'grid', gap: '12px' }}>
                    {equipmentReport.by_category.map((cat, idx) => (
                      <div key={idx} className="card" style={{ padding: '16px 18px', borderLeft: '4px solid var(--primary-color)' }}>
                        <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                          <strong>{cat.category_name || 'Без категории'}</strong>
                          <span style={{ color: 'var(--text-secondary)' }}>Всего: {cat.total ?? 0}</span>
                        </div>
                        <div className="report-stats-grid" style={{ display: 'grid', gap: '12px', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))' }}>
                          <div style={{ padding: '12px', background: 'var(--surface)', borderRadius: '8px' }}><span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>Доступно</span><div style={{ fontSize: '20px', fontWeight: 700, color: 'var(--success)' }}>{cat.available ?? 0}</div></div>
                          <div style={{ padding: '12px', background: 'var(--surface)', borderRadius: '8px' }}><span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>В бронировании</span><div style={{ fontSize: '20px', fontWeight: 700, color: 'var(--warning)' }}>{cat.booked ?? 0}</div></div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
              {allEquipment?.length > 0 && (
                <div className="report-section" style={{ marginTop: '20px' }}>
                  <h3 className="report-section-title">Оборудование: остатки и доступность</h3>
                  <div className="table-report-wrap table-responsive">
                    <table className="table-report">
                      <thead>
                        <tr>
                          <th className="col-num">#</th>
                          <th>Название</th>
                          <th>Категория</th>
                          <th>Всего (шт.)</th>
                          <th>Доступно (шт.)</th>
                          <th>В бронировании (шт.)</th>
                          <th>Статус</th>
                          <th>Местоположение</th>
                        </tr>
                      </thead>
                      <tbody>
                        {(allEquipment || []).map((eq, index) => {
                          const catName = (equipmentReport?.by_category?.find((c) => c.category_id === eq.category_id)?.category_name) || 'Без категории';
                          const total = eq.quantity ?? 0;
                          const available = eq.available_quantity ?? 0;
                          const booked = Math.max(0, total - available);
                          const statusLabel = eq.status === 'available' ? 'Доступно' : eq.status === 'maintenance' ? 'На обслуживании' : eq.status || '—';
                          return (
                            <tr key={eq.id}>
                              <td className="col-num">{index + 1}</td>
                              <td>{eq.name || '—'}</td>
                              <td>{catName}</td>
                              <td>{total}</td>
                              <td>{available}</td>
                              <td>{booked}</td>
                              <td>{statusLabel}</td>
                              <td>{eq.location || '—'}</td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}
            </div>
          )}
        </div>

        {/* Блок: отчёт по бронированиям */}
        <div className="card report-card">
          <div className="report-card-header" style={{ marginBottom: '16px' }}>
            <h2 style={{ margin: 0 }}>Бронирования</h2>
            <p style={{ margin: '4px 0 0', color: 'var(--text-secondary)', fontSize: '13px' }}>Статистика и топ за период (период задаётся выше).</p>
          </div>
          <div style={{ marginBottom: '16px', display: 'flex', gap: '12px', flexWrap: 'wrap', alignItems: 'center' }}>
            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap', minWidth: 0, maxWidth: '100%' }}>
              {['all', 'today', 'yesterday', 'week', 'month'].map((preset) => (
                <button key={preset} type="button" className={`btn btn-secondary ${periodPreset === preset ? 'active' : ''}`} onClick={() => applyPeriodPreset(preset)}>
                  {preset === 'all' && 'За всё время'}
                  {preset === 'today' && 'Сегодня'}
                  {preset === 'yesterday' && 'Вчера'}
                  {preset === 'week' && 'Последние 7 дней'}
                  {preset === 'month' && 'Текущий месяц'}
                </button>
              ))}
            </div>
            <label style={{ display: 'flex', flexDirection: 'column', fontSize: '12px', color: 'var(--text-secondary)' }}>
              <span>Период с</span>
              <input type="date" value={fromDate} onChange={(e) => { setPeriodPreset('custom'); setFromDate(e.target.value); }} style={{ padding: '6px 8px', borderRadius: '6px', border: '1px solid var(--border-color)', background: 'var(--surface)', color: 'var(--text-primary)' }} />
            </label>
            <label style={{ display: 'flex', flexDirection: 'column', fontSize: '12px', color: 'var(--text-secondary)' }}>
              <span>по</span>
              <input type="date" value={toDate} onChange={(e) => { setPeriodPreset('custom'); setToDate(e.target.value); }} style={{ padding: '6px 8px', borderRadius: '6px', border: '1px solid var(--border-color)', background: 'var(--surface)', color: 'var(--text-primary)' }} />
            </label>
          </div>
          {bookingReport && (
            <div className="report-card-body" style={{ display: 'grid', gap: '16px', minWidth: 0, maxWidth: '100%' }}>
              <div className="report-stats-grid" style={{ display: 'grid', gap: '16px', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))' }}>
                {[
                  { key: 'total', label: 'Всего бронирований', color: 'var(--primary-color)' },
                  { key: 'pending', label: 'Ожидают одобрения', color: 'var(--warning)' },
                  { key: 'approved', label: 'Одобрены', color: 'var(--success)' },
                  { key: 'awaiting_return', label: 'Ожидают возврата', color: 'var(--warning)' },
                  { key: 'returned', label: 'Возвращены', color: 'var(--success)' },
                  { key: 'rejected', label: 'Отклонены', color: 'var(--error)' },
                  { key: 'expired', label: 'Истекли', color: 'var(--text-secondary)' },
                  { key: 'cancelled', label: 'Отменены', color: 'var(--warning)' },
                  { key: 'completed', label: 'Завершены', color: 'var(--success)' },
                ].map(({ key, label, color }) => (
                  <div key={key} className="card" style={{ padding: '16px', borderLeft: `4px solid ${color}` }}>
                    <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '4px' }}>{label}</div>
                    <div style={{ fontSize: '24px', fontWeight: 700, color }}>{bookingSummary[key] ?? 0}</div>
                  </div>
                ))}
              </div>
              {(equipmentUsage.length > 0 || categoryUsage.length > 0) && (
                <div className="card report-section report-tables-section">
                  <h3 className="report-section-title">Топы за период</h3>
                  {equipmentUsage.length > 0 && (
                    <div className="report-table-block">
                      <h4 className="report-table-block-title">Топ оборудования по количеству бронирований</h4>
                      <div className="table-report-wrap table-responsive">
                        <table className="table-report">
                          <thead>
                            <tr>
                              <th className="col-num">#</th>
                              <th>Оборудование</th>
                              <th>Категория</th>
                              <th>Бронирований</th>
                              <th>% от всех</th>
                              <th>Общий объём (шт.)</th>
                            </tr>
                          </thead>
                          <tbody>
                            {equipmentUsage.slice(0, 20).map((item, index) => (
                              <tr key={item.equipmentId || index}>
                                <td className="col-num">{index + 1}</td>
                                <td>{item.equipmentName}</td>
                                <td>{item.categoryName}</td>
                                <td>{item.bookingsCount}</td>
                                <td>{totalEquipmentBookings > 0 ? ((item.bookingsCount / totalEquipmentBookings) * 100).toFixed(1) : '0.0'}%</td>
                                <td>{item.totalQuantity}</td>
                              </tr>
                            ))}
                          </tbody>
                          <tfoot>
                            <tr>
                              <td className="col-num" colSpan={3} style={{ textAlign: 'right', fontWeight: 600 }}>Итого</td>
                              <td style={{ fontWeight: 600 }}>{totalEquipmentBookings}</td>
                              <td colSpan={2} style={{ color: 'var(--text-secondary)' }}>—</td>
                            </tr>
                          </tfoot>
                        </table>
                      </div>
                    </div>
                  )}
                  {categoryUsage.length > 0 && (
                    <div className="report-table-block">
                      <h4 className="report-table-block-title">Топ категорий по бронированиям</h4>
                      <div className="table-report-wrap table-responsive">
                        <table className="table-report">
                          <thead>
                            <tr>
                              <th className="col-num">#</th>
                              <th>Категория</th>
                              <th>Бронирований</th>
                              <th>% от всех</th>
                            </tr>
                          </thead>
                          <tbody>
                            {categoryUsage.slice(0, 20).map((item, index) => (
                              <tr key={item.categoryName || index}>
                                <td className="col-num">{index + 1}</td>
                                <td>{item.categoryName}</td>
                                <td>{item.bookingsCount}</td>
                                <td>{totalCategoryBookings > 0 ? ((item.bookingsCount / totalCategoryBookings) * 100).toFixed(1) : '0.0'}%</td>
                              </tr>
                            ))}
                          </tbody>
                          <tfoot>
                            <tr>
                              <td className="col-num" colSpan={2} style={{ textAlign: 'right', fontWeight: 600 }}>Итого</td>
                              <td style={{ fontWeight: 600 }}>{totalCategoryBookings}</td>
                              <td style={{ color: 'var(--text-secondary)' }}>—</td>
                            </tr>
                          </tfoot>
                        </table>
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default Reports;
