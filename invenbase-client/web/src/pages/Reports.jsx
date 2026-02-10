import React, { useState, useEffect } from 'react';
import { useAuth } from '../context/AuthContext';
import { reportsAPI } from '../api/reports';
import { bookingsAPI } from '../api/bookings';
import { equipmentAPI } from '../api/equipment';
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
  const [periodPreset, setPeriodPreset] = useState('all'); // all, today, yesterday, week, month, custom

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

  const exportEquipmentReport = () => {
    if (!equipmentReport) return;

    const summarySheetData = [
      ['Отчёт по оборудованию'],
      [],
      ['Показатель', 'Значение'],
      ['Всего оборудования', equipmentReport.total_equipment || 0],
      ['Доступно', equipmentReport.available_equipment || 0],
      ['Забронировано', equipmentReport.booked_equipment || 0],
    ];

    const workbook = XLSX.utils.book_new();
    const summaryWs = XLSX.utils.aoa_to_sheet(summarySheetData);
    XLSX.utils.book_append_sheet(workbook, summaryWs, 'Итоги');

    if (equipmentReport.by_category && equipmentReport.by_category.length > 0) {
      const categoryData = [
        ['Категория', 'Всего', 'Доступно', 'Забронировано'],
        ...equipmentReport.by_category.map(cat => [
          cat.category_name || 'Без категории',
          cat.total || 0,
          cat.available || 0,
          cat.booked || 0,
        ]),
      ];
      const catWs = XLSX.utils.aoa_to_sheet(categoryData);
      XLSX.utils.book_append_sheet(workbook, catWs, 'По категориям');
    }

    // Лист «Оборудование»: каждая позиция с остатками, статусом, доступностью
    if (allEquipment && allEquipment.length > 0) {
      const categoryNameById = {};
      if (equipmentReport.by_category) {
        equipmentReport.by_category.forEach((c) => {
          categoryNameById[c.category_id] = c.category_name || 'Без категории';
        });
      }
      const equipmentStatusRu = { available: 'Доступно', maintenance: 'На обслуживании' };
      const equipmentSheetData = [
        ['№', 'Название', 'Категория', 'Всего (шт.)', 'Доступно (шт.)', 'В бронировании (шт.)', 'Статус', 'Местоположение'],
        ...allEquipment.map((eq, index) => {
          const total = eq.quantity ?? 0;
          const available = eq.available_quantity ?? 0;
          const booked = Math.max(0, total - available);
          const statusLabel = equipmentStatusRu[eq.status] || (eq.status ? 'Иное' : '—');
          return [
            index + 1,
            eq.name || '—',
            categoryNameById[eq.category_id] || 'Без категории',
            total,
            available,
            booked,
            statusLabel,
            eq.location || '—',
          ];
        }),
      ];
      const eqWs = XLSX.utils.aoa_to_sheet(equipmentSheetData);
      XLSX.utils.book_append_sheet(workbook, eqWs, 'Оборудование');
    }

    XLSX.writeFile(workbook, 'equipment_report.xlsx');
  };

  const exportBookingDetailedXlsx = async () => {
    try {
      let data = allBookings;

      // Если по какой-то причине бронирования ещё не загружены
      if (!data || data.length === 0) {
        data = await bookingsAPI.getAll();
        setAllBookings(data || []);
      }

      // Предварительные словари по оборудованию и категориям
      const equipmentById = {};
      allEquipment.forEach((eq) => {
        equipmentById[eq.id] = eq;
      });

      const categoryNameById = {};
      if (equipmentReport && equipmentReport.by_category) {
        equipmentReport.by_category.forEach((cat) => {
          categoryNameById[cat.category_id] = cat.category_name;
        });
      }

      // Фильтрация по периоду на клиенте
      const filtered = (data || []).filter((b) => {
        if (!b.start_date) return false;
        const d = new Date(b.start_date);
        const iso = d.toISOString().slice(0, 10); // YYYY-MM-DD
        if (fromDate && iso < fromDate) return false;
        if (toDate && iso > toDate) return false;
        return true;
      });

      if (!filtered || filtered.length === 0) {
        alert('За выбранный период нет бронирований по оборудованию');
        return;
      }

      // Словари для перевода на русский в экспорте
      const statusLabelsRu = {
        pending: 'Ожидают одобрения',
        approved: 'Одобрены',
        rejected: 'Отклонены',
        cancelled: 'Отменены',
        completed: 'Завершены',
        expired: 'Истекли',
      };
      const permissionTypeLabelsRu = {
        internal: 'Внутренний',
        external: 'Внешний',
      };

      const header = [
        'Оборудование',
        'Категория',
        'Пользователь',
        'Статус',
        'Тип доступа',
        'Количество',
        'Цель',
        'Дата начала',
        'Дата окончания',
      ];

      const rows = filtered
        .filter((b) => b.equipment_id) // только бронирования конкретного оборудования
        .map((b) => {
          const eq = equipmentById[b.equipment_id] || {};
          const categoryId = eq.category_id;
          const equipmentName = b.equipment_name || eq.name || 'Без названия';
          const categoryName =
            (categoryId && categoryNameById[categoryId]) || 'Без категории';
          const statusRu = statusLabelsRu[b.status] || (b.status ? 'Прочее' : '—');
          const permissionRu = permissionTypeLabelsRu[b.permission_type] || (b.permission_type ? 'Прочее' : '—');

          return [
            equipmentName,
            categoryName,
            b.username || '—',
            statusRu,
            permissionRu,
            b.quantity,
            b.purpose || '—',
            b.start_date ? new Date(b.start_date).toLocaleString('ru-RU') : '',
            b.end_date ? new Date(b.end_date).toLocaleString('ru-RU') : '',
          ];
        });

      if (rows.length === 0) {
        alert('За выбранный период нет бронирований по оборудованию');
        return;
      }

      const detailSheet = XLSX.utils.aoa_to_sheet([header, ...rows]);
      const workbook = XLSX.utils.book_new();

      // Итоги по периоду (по отфильтрованным данным)
      const byStatus = {};
      const byPermissionType = {};
      filtered.forEach((b) => {
        byStatus[b.status] = (byStatus[b.status] || 0) + 1;
        const pt = b.permission_type || '—';
        byPermissionType[pt] = (byPermissionType[pt] || 0) + 1;
      });

      const summarySheetData = [
        ['Итоги по выбранному периоду'],
        ['Период', fromDate && toDate ? `${fromDate} — ${toDate}` : fromDate || toDate || 'Всё время'],
        [],
        ['Показатель', 'Значение'],
        ['Всего бронирований в выборке', filtered.length],
      ];
      const summaryWs = XLSX.utils.aoa_to_sheet(summarySheetData);
      XLSX.utils.book_append_sheet(workbook, summaryWs, 'Итоги');

      // Лист «По статусам» (разрез по периоду) — только русские подписи
      const statusSheetData = [
        ['Статус', 'Количество'],
        ...Object.entries(byStatus).map(([status, count]) => [
          statusLabelsRu[status] || status,
          count,
        ]),
      ];
      const statusWs = XLSX.utils.aoa_to_sheet(statusSheetData);
      XLSX.utils.book_append_sheet(workbook, statusWs, 'По статусам');

      // Лист «По типам доступа» (разрез по периоду) — только русские подписи
      const permissionSheetData = [
        ['Тип доступа', 'Количество'],
        ...Object.entries(byPermissionType).map(([type, count]) => [
          permissionTypeLabelsRu[type] || (type && type !== '—' ? 'Прочее' : '—'),
          count,
        ]),
      ];
      const permissionWs = XLSX.utils.aoa_to_sheet(permissionSheetData);
      XLSX.utils.book_append_sheet(workbook, permissionWs, 'По типам доступа');

      XLSX.utils.book_append_sheet(workbook, detailSheet, 'Детализация');

      let filename = 'equipment_usage';
      if (fromDate || toDate) {
        const fromPart = fromDate || '...';
        const toPart = toDate || '...';
        filename += `_${fromPart}_${toPart}`;
      }
      filename += '.xlsx';

      XLSX.writeFile(workbook, filename);
    } catch (error) {
      console.error('Failed to export detailed equipment report:', error);
      alert('Ошибка выгрузки детального отчёта по оборудованию');
    }
  };

  const applyPeriodPreset = (preset) => {
    setPeriodPreset(preset);
    const today = new Date();

    const formatDate = (d) => d.toISOString().slice(0, 10);

    if (preset === 'all') {
      setFromDate('');
      setToDate('');
    } else if (preset === 'today') {
      const d = formatDate(today);
      setFromDate(d);
      setToDate(d);
    } else if (preset === 'yesterday') {
      const d = new Date(today);
      d.setDate(d.getDate() - 1);
      const f = formatDate(d);
      setFromDate(f);
      setToDate(f);
    } else if (preset === 'week') {
      const d = new Date(today);
      d.setDate(d.getDate() - 6); // последние 7 дней
      setFromDate(formatDate(d));
      setToDate(formatDate(today));
    } else if (preset === 'month') {
      const d = new Date(today.getFullYear(), today.getMonth(), 1);
      setFromDate(formatDate(d));
      setToDate(formatDate(today));
    }
  };

  // Общая функция фильтрации бронирований по периоду
  const getFilteredBookingsByPeriod = () => {
    if (!allBookings || allBookings.length === 0) return [];
    return allBookings.filter((b) => {
      if (!b.start_date) return false;
      const d = new Date(b.start_date);
      const iso = d.toISOString().slice(0, 10); // YYYY-MM-DD
      if (fromDate && iso < fromDate) return false;
      if (toDate && iso > toDate) return false;
      return true;
    });
  };

  // Подготовка данных использования оборудования за период
  const computeUsageStats = () => {
    const bookings = getFilteredBookingsByPeriod();
    if (!bookings || bookings.length === 0) {
      return { equipmentUsage: [], categoryUsage: [] };
    }

    const categoryNameById = {};
    if (equipmentReport && equipmentReport.by_category) {
      equipmentReport.by_category.forEach((cat) => {
        categoryNameById[cat.category_id] = cat.category_name;
      });
    }

    const equipmentById = {};
    allEquipment.forEach((eq) => {
      equipmentById[eq.id] = eq;
    });

    const byEquipment = {};
    const byCategory = {};

    bookings.forEach((b) => {
      if (!b.equipment_id) return;
      if (!b.start_date) return;

      const d = new Date(b.start_date);
      const iso = d.toISOString().slice(0, 10);
      if (fromDate && iso < fromDate) return;
      if (toDate && iso > toDate) return;

      const eqId = b.equipment_id;
      const eq = equipmentById[eqId] || {};
      const catId = eq.category_id;

      // По оборудованию
      if (!byEquipment[eqId]) {
        byEquipment[eqId] = {
          equipmentId: eqId,
          equipmentName: b.equipment_name || eq.name || 'Без названия',
          categoryId: catId || null,
          categoryName: (catId && categoryNameById[catId]) || 'Без категории',
          bookingsCount: 0,
          totalQuantity: 0,
        };
      }
      byEquipment[eqId].bookingsCount += 1;
      byEquipment[eqId].totalQuantity += b.quantity || 0;

      // По категориям
      if (catId) {
        if (!byCategory[catId]) {
          byCategory[catId] = {
            categoryId: catId,
            categoryName: categoryNameById[catId] || 'Без категории',
            bookingsCount: 0,
            totalQuantity: 0,
          };
        }
        byCategory[catId].bookingsCount += 1;
        byCategory[catId].totalQuantity += b.quantity || 0;
      }
    });

    const equipmentUsage = Object.values(byEquipment).sort(
      (a, b) => b.bookingsCount - a.bookingsCount || b.totalQuantity - a.totalQuantity
    );
    const categoryUsage = Object.values(byCategory).sort(
      (a, b) => b.bookingsCount - a.bookingsCount || b.totalQuantity - a.totalQuantity
    );

    return { equipmentUsage, categoryUsage };
  };

  const { equipmentUsage, categoryUsage } = computeUsageStats();

  // Итоги для отчётных таблиц по бронированиям (используются в заголовках/подписях)
  const totalEquipmentBookings = equipmentUsage.reduce(
    (sum, item) => sum + (item.bookingsCount || 0),
    0
  );
  const totalEquipmentQuantity = equipmentUsage.reduce(
    (sum, item) => sum + (item.totalQuantity || 0),
    0
  );

  const totalCategoryBookings = categoryUsage.reduce(
    (sum, item) => sum + (item.bookingsCount || 0),
    0
  );
  const totalCategoryQuantity = categoryUsage.reduce(
    (sum, item) => sum + (item.totalQuantity || 0),
    0
  );

  const computeBookingSummary = () => {
    const bookings = getFilteredBookingsByPeriod();
    const summary = {
      total: 0,
      pending: 0,
      approved: 0,
      expired: 0,
      rejected: 0,
      cancelled: 0,
      completed: 0,
    };

    const now = new Date();

    bookings.forEach((b) => {
      summary.total += 1;
      const status = b.status;
      if (status === 'pending') {
        summary.pending += 1;
      } else if (status === 'approved') {
        // Разделяем активные и истёкшие
        const end = b.end_date ? new Date(b.end_date) : null;
        if (end && end < now) {
          summary.expired += 1;
        } else {
          summary.approved += 1;
        }
      } else if (status === 'rejected') {
        summary.rejected += 1;
      } else if (status === 'cancelled') {
        summary.cancelled += 1;
      } else if (status === 'completed') {
        summary.completed += 1;
      }
    });

    return summary;
  };

  const bookingSummary = computeBookingSummary();

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
      <h1 style={{ marginBottom: '24px' }}>Отчёты</h1>

      <div style={{ display: 'flex', flexDirection: 'column', gap: '24px' }}>
        {/* Блок отчёта по оборудованию */}
        <div className="card" style={{ padding: '20px 24px' }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '20px' }}>
            <div>
              <h2 style={{ margin: 0 }}>Отчёт по оборудованию</h2>
              <p style={{ margin: '4px 0 0', color: 'var(--text-secondary)', fontSize: '13px' }}>
                Сводная информация по наличию и использованию оборудования.
              </p>
            </div>
            {equipmentReport && (
              <button className="btn btn-secondary" onClick={exportEquipmentReport}>
                <span className="material-icons" style={{ marginRight: '6px' }}>download</span>
                Выгрузить XLSX
              </button>
            )}
          </div>

          {equipmentReport && (
            <div style={{ display: 'grid', gap: '20px' }}>
              {/* Верхние карточки‑итоги */}
              <div
                style={{
                  display: 'grid',
                  gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
                  gap: '16px',
                }}
              >
                <div className="card">
                  <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                    <div
                      style={{
                        width: '52px',
                        height: '52px',
                        borderRadius: '14px',
                        background: 'linear-gradient(135deg, var(--primary-color) 0%, var(--secondary-color) 100%)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: 'white',
                        boxShadow: '0 6px 16px rgba(168, 85, 247, 0.45)',
                      }}
                    >
                      <span className="material-icons" style={{ fontSize: '28px' }}>inventory</span>
                    </div>
                    <div>
                      <div
                        style={{
                          fontSize: '11px',
                          color: 'var(--text-secondary)',
                          textTransform: 'uppercase',
                          letterSpacing: '0.6px',
                          marginBottom: '4px',
                        }}
                      >
                        Всего позиций
                      </div>
                      <div style={{ fontSize: '30px', fontWeight: 'bold', color: 'var(--text-primary)' }}>
                        {equipmentReport.total_equipment || 0}
                      </div>
                    </div>
                  </div>
                </div>

                <div className="card">
                  <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                    <div
                      style={{
                        width: '52px',
                        height: '52px',
                        borderRadius: '14px',
                        background: 'linear-gradient(135deg, var(--success) 0%, #10B981 100%)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: 'white',
                        boxShadow: '0 6px 16px rgba(16, 185, 129, 0.45)',
                      }}
                    >
                      <span className="material-icons" style={{ fontSize: '28px' }}>check_circle</span>
                    </div>
                    <div>
                      <div
                        style={{
                          fontSize: '11px',
                          color: 'var(--text-secondary)',
                          textTransform: 'uppercase',
                          letterSpacing: '0.6px',
                          marginBottom: '4px',
                        }}
                      >
                        Доступно сейчас
                      </div>
                      <div style={{ fontSize: '30px', fontWeight: 'bold', color: 'var(--text-primary)' }}>
                        {equipmentReport.available_equipment || 0}
                      </div>
                    </div>
                  </div>
                </div>

                <div className="card">
                  <div style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
                    <div
                      style={{
                        width: '52px',
                        height: '52px',
                        borderRadius: '14px',
                        background: 'linear-gradient(135deg, var(--warning) 0%, #F97316 100%)',
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        color: 'white',
                        boxShadow: '0 6px 16px rgba(234, 179, 8, 0.45)',
                      }}
                    >
                      <span className="material-icons" style={{ fontSize: '28px' }}>event</span>
                    </div>
                    <div>
                      <div
                        style={{
                          fontSize: '11px',
                          color: 'var(--text-secondary)',
                          textTransform: 'uppercase',
                          letterSpacing: '0.6px',
                          marginBottom: '4px',
                        }}
                      >
                        В бронировании
                      </div>
                      <div style={{ fontSize: '30px', fontWeight: 'bold', color: 'var(--text-primary)' }}>
                        {equipmentReport.booked_equipment || 0}
                      </div>
                    </div>
                  </div>
                </div>
              </div>

              {/* Статистика по категориям — в едином стиле сайта */}
              {equipmentReport.by_category && equipmentReport.by_category.length > 0 && (
                <div style={{ marginTop: '8px' }}>
                  <h3 className="report-section-title">Статистика по категориям</h3>
                  <div style={{ display: 'grid', gap: '12px' }}>
                    {equipmentReport.by_category.map((cat, index) => (
                      <div
                        key={index}
                        className="card"
                        style={{
                          padding: '16px 18px',
                          background: 'var(--surface)',
                          borderRadius: '12px',
                          borderLeft: '4px solid var(--primary-color)',
                          boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
                          border: '1px solid rgba(255,255,255,0.05)',
                        }}
                      >
                        <div
                          style={{
                            display: 'flex',
                            justifyContent: 'space-between',
                            alignItems: 'center',
                            marginBottom: '10px',
                          }}
                        >
                          <div
                            style={{
                              fontWeight: 600,
                              fontSize: '14px',
                              color: 'var(--text-primary)',
                              textTransform: 'uppercase',
                              letterSpacing: '0.6px',
                            }}
                          >
                            {cat.category_name || 'Без категории'}
                          </div>
                          <div style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                            Всего: <strong style={{ color: 'var(--text-primary)' }}>{cat.total || 0}</strong>
                          </div>
                        </div>
                        <div
                          style={{
                            display: 'grid',
                            gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))',
                            gap: '20px',
                            marginTop: '4px',
                          }}
                        >
                          <div
                            style={{
                              padding: '20px',
                              background: 'var(--surface)',
                              borderRadius: '12px',
                              borderLeft: '4px solid var(--success)',
                              boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
                              border: '1px solid rgba(255,255,255,0.05)',
                            }}
                          >
                            <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>
                              Доступно
                            </div>
                            <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--success)' }}>
                              {cat.available || 0}
                            </div>
                          </div>
                          <div
                            style={{
                              padding: '20px',
                              background: 'var(--surface)',
                              borderRadius: '12px',
                              borderLeft: '4px solid var(--warning)',
                              boxShadow: '0 4px 12px rgba(0,0,0,0.3)',
                              border: '1px solid rgba(255,255,255,0.05)',
                            }}
                          >
                            <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>
                              В бронировании
                            </div>
                            <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--warning)' }}>
                              {cat.booked || 0}
                            </div>
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}

              {/* Таблица по оборудованию: остатки, статус, доступность */}
              {allEquipment && allEquipment.length > 0 && (
                <div style={{ marginTop: '20px' }}>
                  <h3 className="report-section-title">Оборудование: остатки и доступность</h3>
                  <div className="table-report-wrap">
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
                          const categoryName =
                            (equipmentReport && equipmentReport.by_category)
                              ? (equipmentReport.by_category.find((c) => c.category_id === eq.category_id)?.category_name || 'Без категории')
                              : 'Без категории';
                          const total = eq.quantity ?? 0;
                          const available = eq.available_quantity ?? 0;
                          const booked = Math.max(0, total - available);
                          return (
                            <tr key={eq.id}>
                              <td className="col-num">{index + 1}</td>
                              <td>{eq.name || '—'}</td>
                              <td>{categoryName}</td>
                              <td>{total}</td>
                              <td>{available}</td>
                              <td>{booked}</td>
                              <td>{eq.status === 'available' ? 'Доступно' : eq.status === 'maintenance' ? 'На обслуживании' : eq.status || '—'}</td>
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

        {/* Отчёт по бронированиям */}
        <div className="card">
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
            <h2 style={{ margin: 0 }}>Отчёт по бронированиям</h2>
            <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
              <button className="btn btn-primary" onClick={exportBookingDetailedXlsx}>
                <span className="material-icons" style={{ marginRight: '6px' }}>description</span>
                Выгрузить XLSX
              </button>
            </div>
          </div>

          {/* Фильтры периода как в 1С */}
          <div style={{ marginBottom: '16px', display: 'flex', gap: '12px', flexWrap: 'wrap', alignItems: 'center' }}>
            <div style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
              <button
                type="button"
                className={`btn btn-secondary ${periodPreset === 'all' ? 'active' : ''}`}
                onClick={() => applyPeriodPreset('all')}
              >
                За всё время
              </button>
              <button
                type="button"
                className={`btn btn-secondary ${periodPreset === 'today' ? 'active' : ''}`}
                onClick={() => applyPeriodPreset('today')}
              >
                Сегодня
              </button>
              <button
                type="button"
                className={`btn btn-secondary ${periodPreset === 'yesterday' ? 'active' : ''}`}
                onClick={() => applyPeriodPreset('yesterday')}
              >
                Вчера
              </button>
              <button
                type="button"
                className={`btn btn-secondary ${periodPreset === 'week' ? 'active' : ''}`}
                onClick={() => applyPeriodPreset('week')}
              >
                Последние 7 дней
              </button>
              <button
                type="button"
                className={`btn btn-secondary ${periodPreset === 'month' ? 'active' : ''}`}
                onClick={() => applyPeriodPreset('month')}
              >
                Текущий месяц
              </button>
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', fontSize: '12px', color: 'var(--text-secondary)' }}>
              <span>Период с</span>
              <input
                type="date"
                value={fromDate}
                onChange={(e) => {
                  setPeriodPreset('custom');
                  setFromDate(e.target.value);
                }}
                style={{ padding: '6px 8px', borderRadius: '6px', border: '1px solid var(--border-color)', background: 'var(--surface)', color: 'var(--text-primary)' }}
              />
            </div>
            <div style={{ display: 'flex', flexDirection: 'column', fontSize: '12px', color: 'var(--text-secondary)' }}>
              <span>по</span>
              <input
                type="date"
                value={toDate}
                onChange={(e) => {
                  setPeriodPreset('custom');
                  setToDate(e.target.value);
                }}
                style={{ padding: '6px 8px', borderRadius: '6px', border: '1px solid var(--border-color)', background: 'var(--surface)', color: 'var(--text-primary)' }}
              />
            </div>
          </div>
          
          {bookingReport && (
            <div style={{ display: 'grid', gap: '16px' }}>
              <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(220px, 1fr))', gap: '20px' }}>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--primary-color)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Всего бронирований</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--primary-color)' }}>
                    {bookingSummary.total}
                  </div>
                </div>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--warning)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Ожидают одобрения</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--warning)' }}>
                    {bookingSummary.pending}
                  </div>
                </div>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--success)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Одобрены</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--success)' }}>
                    {bookingSummary.approved}
                  </div>
                </div>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--error)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Отклонены</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--error)' }}>
                    {bookingSummary.rejected}
                  </div>
                </div>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--text-secondary)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Истекли</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--text-secondary)' }}>
                    {bookingSummary.expired}
                  </div>
                </div>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--warning)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Отменены</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--warning)' }}>
                    {bookingSummary.cancelled}
                  </div>
                </div>
                <div style={{ padding: '20px', background: 'var(--surface)', borderRadius: '12px', borderLeft: '4px solid var(--success)', boxShadow: '0 4px 12px rgba(0,0,0,0.3)', border: '1px solid rgba(255,255,255,0.05)' }}>
                  <div style={{ fontSize: '12px', color: 'var(--text-secondary)', marginBottom: '8px', fontWeight: 500, textTransform: 'uppercase', letterSpacing: '0.5px' }}>Завершены</div>
                  <div style={{ fontSize: '32px', fontWeight: 700, color: 'var(--success)' }}>
                    {bookingSummary.completed}
                  </div>
                </div>
              </div>

              {/* Топ оборудования по бронированиям */}
              {equipmentUsage && equipmentUsage.length > 0 && (
                <div className="card" style={{ marginTop: '20px', padding: '20px 24px' }}>
                  <h3 className="report-section-title">Топ оборудования по количеству бронирований</h3>
                  <div className="table-report-wrap">
                    <table className="table-report">
                      <thead>
                        <tr>
                          <th className="col-num">#</th>
                          <th>Оборудование</th>
                          <th>Категория</th>
                          <th>Бронирований</th>
                          <th>% от всех бронирований</th>
                          <th>Общий объём (шт.)</th>
                          <th>Средний объём за бронирование</th>
                        </tr>
                      </thead>
                      <tbody>
                        {equipmentUsage.slice(0, 20).map((item, index) => (
                          <tr key={item.equipmentId || index}>
                            <td className="col-num">{index + 1}</td>
                            <td>{item.equipmentName}</td>
                            <td>{item.categoryName}</td>
                            <td>{item.bookingsCount}</td>
                            <td>
                              {totalEquipmentBookings > 0
                                ? ((item.bookingsCount / totalEquipmentBookings) * 100).toFixed(1)
                                : '0.0'}
                            </td>
                            <td>{item.totalQuantity}</td>
                            <td>
                              {item.bookingsCount > 0
                                ? (item.totalQuantity / item.bookingsCount).toFixed(1)
                                : '0.0'}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                      <tfoot>
                        <tr>
                          <td className="col-num" colSpan={3} style={{ textAlign: 'right', fontWeight: 600 }}>
                            Итого (количество бронирований):
                          </td>
                          <td style={{ fontWeight: 600 }}>{totalEquipmentBookings}</td>
                          <td colSpan={3} style={{ color: 'var(--text-secondary)' }}>—</td>
                        </tr>
                      </tfoot>
                    </table>
                  </div>
                </div>
              )}

              {/* Топ категорий по бронированиям */}
              {categoryUsage && categoryUsage.length > 0 && (
                <div className="card" style={{ marginTop: '20px', padding: '20px 24px' }}>
                  <h3 className="report-section-title">Топ категорий по бронированиям</h3>
                  <div className="table-report-wrap">
                    <table className="table-report">
                      <thead>
                        <tr>
                          <th className="col-num">#</th>
                          <th>Категория</th>
                          <th>Бронирований</th>
                          <th>% от всех бронирований</th>
                          <th>Общий объём (шт.)</th>
                          <th>Средний объём за бронирование</th>
                        </tr>
                      </thead>
                      <tbody>
                        {categoryUsage.slice(0, 20).map((item, index) => (
                          <tr key={item.categoryId || index}>
                            <td className="col-num">{index + 1}</td>
                            <td>{item.categoryName}</td>
                            <td>{item.bookingsCount}</td>
                            <td>
                              {totalCategoryBookings > 0
                                ? ((item.bookingsCount / totalCategoryBookings) * 100).toFixed(1)
                                : '0.0'}
                            </td>
                            <td>{item.totalQuantity}</td>
                            <td>
                              {item.bookingsCount > 0
                                ? (item.totalQuantity / item.bookingsCount).toFixed(1)
                                : '0.0'}
                            </td>
                          </tr>
                        ))}
                      </tbody>
                      <tfoot>
                        <tr>
                          <td className="col-num" colSpan={2} style={{ textAlign: 'right', fontWeight: 600 }}>
                            Итого (количество бронирований):
                          </td>
                          <td style={{ fontWeight: 600 }}>{totalCategoryBookings}</td>
                          <td colSpan={3} style={{ color: 'var(--text-secondary)' }}>—</td>
                        </tr>
                      </tfoot>
                    </table>
                  </div>
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
