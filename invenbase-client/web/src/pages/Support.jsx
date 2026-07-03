import React, { useState, useEffect } from 'react';
import { supportAPI } from '../api/support';
import { useAuth } from '../context/AuthContext';
import { format } from 'date-fns';
import { ru } from 'date-fns/locale';

const STATUS_LABELS = {
  open: 'Открыта',
  in_progress: 'В работе',
  answered: 'Ответ дан',
  closed: 'Закрыта',
};

const STATUS_OPTIONS = [
  { value: 'open', label: 'Открыта' },
  { value: 'in_progress', label: 'В работе' },
  { value: 'answered', label: 'Ответ дан' },
  { value: 'closed', label: 'Закрыта' },
];

const TAB_CREATE = 'create';
const TAB_REQUESTS = 'requests';

const Support = () => {
  const { isAdmin } = useAuth();
  const [tab, setTab] = useState(isAdmin() ? TAB_REQUESTS : TAB_CREATE);
  const [requests, setRequests] = useState([]);
  const [loading, setLoading] = useState(true);
  const [formData, setFormData] = useState({ subject: '', message: '' });
  const [submitting, setSubmitting] = useState(false);
  const [replyingTo, setReplyingTo] = useState(null);
  const [replyForm, setReplyForm] = useState({ status: 'open', admin_comment: '' });
  const [replySubmitting, setReplySubmitting] = useState(false);
  const [addingMessageTo, setAddingMessageTo] = useState(null);
  const [addMessageText, setAddMessageText] = useState('');
  const [addMessageSubmitting, setAddMessageSubmitting] = useState(false);
  const [deletingId, setDeletingId] = useState(null);

  useEffect(() => {
    fetchRequests();
  }, []);

  const fetchRequests = async () => {
    try {
      const data = await supportAPI.getRequests();
      setRequests(data);
    } catch (error) {
      console.error('Failed to fetch support requests:', error);
    } finally {
      setLoading(false);
    }
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    if (!formData.subject.trim() || !formData.message.trim()) {
      alert('Заполните тему и сообщение.');
      return;
    }
    setSubmitting(true);
    try {
      await supportAPI.createRequest({
        subject: formData.subject.trim(),
        message: formData.message.trim(),
      });
      setFormData({ subject: '', message: '' });
      await fetchRequests();
      setTab(TAB_REQUESTS);
      alert('Заявка отправлена. Мы ответим в ближайшее время.');
    } catch (error) {
      console.error('Failed to create support request:', error);
      alert(error.response?.data?.message || 'Ошибка при отправке заявки.');
    } finally {
      setSubmitting(false);
    }
  };

  const openReply = (req) => {
    setReplyingTo(req.id);
    setReplyForm({ status: req.status || 'open', admin_comment: '' });
  };

  const handleReplySubmit = async (e) => {
    e.preventDefault();
    if (!replyingTo) return;
    const comment = replyForm.admin_comment.trim();
    setReplySubmitting(true);
    try {
      await supportAPI.updateRequest(replyingTo, {
        status: replyForm.status,
        admin_comment: comment || null,
      });
      setReplyForm((prev) => ({ ...prev, admin_comment: '' }));
      await fetchRequests();
      if (comment) alert('Ответ добавлен в переписку.');
    } catch (error) {
      console.error('Failed to update support request:', error);
      alert(error.response?.data?.message || 'Ошибка при сохранении ответа.');
    } finally {
      setReplySubmitting(false);
    }
  };

  const getThread = (req) => {
    const thread = [
      { isStaff: false, message: req.message, created_at: req.created_at },
    ];
    if (req.messages && req.messages.length > 0) {
      req.messages.forEach((m) => thread.push({ isStaff: m.is_staff, message: m.message, created_at: m.created_at }));
    } else if (req.admin_comment) {
      thread.push({ isStaff: true, message: req.admin_comment, created_at: req.updated_at });
    }
    return thread;
  };

  const handleAddMessage = async (e, requestId) => {
    e.preventDefault();
    const text = addMessageText.trim();
    if (!text) return;
    setAddMessageSubmitting(true);
    try {
      await supportAPI.addMessage(requestId, { message: text });
      setAddingMessageTo(null);
      setAddMessageText('');
      await fetchRequests();
      alert('Сообщение добавлено.');
    } catch (error) {
      console.error('Failed to add message:', error);
      alert(error.response?.data?.message || 'Ошибка при отправке сообщения.');
    } finally {
      setAddMessageSubmitting(false);
    }
  };

  const handleDeleteRequest = async (req) => {
    if (req.status !== 'closed') return;
    if (!window.confirm('Удалить закрытую заявку? Это действие нельзя отменить.')) return;
    const id = req.request?.id ?? req.id;
    if (!id) {
      alert('Ошибка: не найден идентификатор заявки.');
      return;
    }
    setDeletingId(id);
    try {
      await supportAPI.deleteRequest(String(id));
      await fetchRequests();
      setReplyingTo(null);
      alert('Заявка удалена.');
    } catch (error) {
      console.error('Failed to delete request:', error);
      const msg = error.response?.data?.message || (error.response?.status === 404 ? 'Заявка не найдена или уже удалена.' : 'Ошибка при удалении заявки.');
      alert(msg);
    } finally {
      setDeletingId(null);
    }
  };

  const tabStyle = (isActive) => ({
    padding: '12px 20px',
    fontSize: '15px',
    fontWeight: 600,
    border: 'none',
    borderBottom: isActive ? '3px solid var(--primary-color)' : '3px solid transparent',
    background: 'none',
    color: isActive ? 'var(--primary-color)' : 'var(--text-secondary)',
    cursor: 'pointer',
  });

  return (
    <div style={{ maxWidth: '900px' }}>
      <h1 style={{ marginBottom: '20px', fontSize: '24px', fontWeight: 600 }}>
        Техническая поддержка
      </h1>

      {/* Меню раздела поддержки: админ не создаёт заявки — только список */}
      {!isAdmin() && (
        <div style={{ display: 'flex', gap: '0', marginBottom: '24px', borderBottom: '1px solid var(--divider)' }}>
          <button type="button" style={tabStyle(tab === TAB_CREATE)} onClick={() => setTab(TAB_CREATE)}>
            Создать заявку
          </button>
          <button type="button" style={tabStyle(tab === TAB_REQUESTS)} onClick={() => setTab(TAB_REQUESTS)}>
            Заявки
          </button>
        </div>
      )}

      {tab === TAB_CREATE && !isAdmin() && (
        <>
          <section style={{ background: 'var(--card-bg)', borderRadius: '8px', padding: '24px', marginBottom: '24px', border: '1px solid var(--divider)' }}>
            <h2 style={{ margin: '0 0 12px', fontSize: '18px', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span className="material-icons" style={{ fontSize: '22px', color: 'var(--primary-color)' }}>contact_support</span>
              Как получить помощь
            </h2>
            <p style={{ margin: 0, color: 'var(--text-secondary)', lineHeight: 1.6 }}>
              Опишите проблему в форме ниже — мы создадим заявку и ответим вам. В одной заявке можно вести переписку до её закрытия. Создавать новую заявку нужно только после того, как заявка получит статус «Закрыта».
            </p>
          </section>
          <section style={{ background: 'var(--card-bg)', borderRadius: '8px', padding: '24px', border: '1px solid var(--divider)' }}>
            <h2 style={{ margin: '0 0 16px', fontSize: '18px', display: 'flex', alignItems: 'center', gap: '8px' }}>
              <span className="material-icons" style={{ fontSize: '22px', color: 'var(--primary-color)' }}>edit_note</span>
              Оставить заявку
            </h2>
            <form onSubmit={handleSubmit}>
              <div style={{ marginBottom: '16px' }}>
                <label className="label">Тема</label>
                <input
                  type="text"
                  className="input"
                  value={formData.subject}
                  onChange={(e) => setFormData({ ...formData, subject: e.target.value })}
                  placeholder="Кратко опишите тему обращения"
                  maxLength={500}
                  required
                />
              </div>
              <div style={{ marginBottom: '20px' }}>
                <label className="label">Сообщение</label>
                <textarea
                  className="input"
                  value={formData.message}
                  onChange={(e) => setFormData({ ...formData, message: e.target.value })}
                  placeholder="Опишите проблему или вопрос подробнее"
                  rows={5}
                  required
                />
              </div>
              <button type="submit" className="btn btn-primary" disabled={submitting}>
                {submitting ? 'Отправка…' : 'Отправить заявку'}
              </button>
            </form>
          </section>
        </>
      )}

      {tab === TAB_REQUESTS && (
        <section style={{ background: 'var(--card-bg)', borderRadius: '8px', padding: '24px', border: '1px solid var(--divider)' }}>
          {loading ? (
            <div className="loading"><div className="spinner"></div></div>
          ) : requests.length === 0 ? (
            <p style={{ color: 'var(--text-secondary)', margin: 0, fontSize: '15px' }}>Заявок пока нет.</p>
          ) : (
            <div style={{ display: 'flex', flexDirection: 'column', gap: '20px' }}>
              {requests.map((req) => (
                <div
                  key={req.id}
                  style={{
                    padding: '20px',
                    background: 'var(--background)',
                    borderRadius: '8px',
                    border: '1px solid var(--divider)',
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '12px', marginBottom: '10px' }}>
                    <strong style={{ fontSize: '16px', lineHeight: 1.3 }}>{req.subject}</strong>
                    <span
                      style={{
                        fontSize: '13px',
                        padding: '6px 12px',
                        borderRadius: '8px',
                        background: req.status === 'closed' ? 'var(--divider)' : 'var(--primary-color)',
                        color: 'white',
                        opacity: req.status === 'closed' ? 0.85 : 1,
                        fontWeight: 500,
                      }}
                    >
                      {STATUS_LABELS[req.status] || req.status}
                    </span>
                  </div>
                  {isAdmin() && req.user_name && (
                    <div style={{ fontSize: '14px', color: 'var(--text-secondary)', marginBottom: '12px' }}>
                      От: {req.user_name}
                    </div>
                  )}
                  <div style={{ marginBottom: '12px' }}>
                    {getThread(req).map((item, idx) => (
                      <div
                        key={idx}
                        style={{
                          marginTop: idx > 0 ? '14px' : 0,
                          padding: '14px',
                          background: item.isStaff ? 'rgba(56, 142, 60, 0.15)' : 'var(--card-bg)',
                          borderRadius: '10px',
                          fontSize: '14px',
                          lineHeight: 1.5,
                          borderLeft: item.isStaff ? '4px solid var(--primary-color)' : '4px solid transparent',
                        }}
                      >
                        {item.isStaff && <strong style={{ display: 'block', marginBottom: '6px', fontSize: '13px' }}>Ответ поддержки</strong>}
                        <span style={{ whiteSpace: 'pre-wrap' }}>{item.message}</span>
                        <div style={{ marginTop: '8px', fontSize: '12px', color: 'var(--text-secondary)' }}>
                          {format(new Date(item.created_at), 'd MMM yyyy, HH:mm', { locale: ru })}
                        </div>
                      </div>
                    ))}
                  </div>
                  <div style={{ marginTop: '14px', display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '12px' }}>
                    <span style={{ fontSize: '13px', color: 'var(--text-secondary)' }}>
                      Создана: {format(new Date(req.created_at), 'd MMMM yyyy, HH:mm', { locale: ru })}
                    </span>
                    {isAdmin() && (
                      <>
                        {req.status === 'closed' ? (
                          <button
                            type="button"
                            className="btn btn-secondary"
                            disabled={deletingId === req.id}
                            onClick={() => handleDeleteRequest(req)}
                          >
                            {deletingId === req.id ? 'Удаление…' : 'Удалить заявку'}
                          </button>
                        ) : (
                          <button type="button" className="btn btn-primary" onClick={() => openReply(req)}>
                            Ответить / добавить ответ
                          </button>
                        )}
                      </>
                    )}
                  </div>
                  {!isAdmin() && req.status !== 'closed' && (
                    <div style={{ marginTop: '14px' }}>
                      {addingMessageTo === req.id ? (
                        <form onSubmit={(e) => handleAddMessage(e, req.id)}>
                          <textarea
                            className="input"
                            value={addMessageText}
                            onChange={(e) => setAddMessageText(e.target.value)}
                            placeholder="Дополните заявку — опишите, что ещё не решено"
                            rows={3}
                            required
                            style={{ marginBottom: '10px' }}
                          />
                          <div style={{ display: 'flex', gap: '10px' }}>
                            <button type="submit" className="btn btn-primary" disabled={addMessageSubmitting}>
                              {addMessageSubmitting ? 'Отправка…' : 'Отправить'}
                            </button>
                            <button type="button" className="btn btn-secondary" onClick={() => { setAddingMessageTo(null); setAddMessageText(''); }}>
                              Отмена
                            </button>
                          </div>
                        </form>
                      ) : (
                        <button type="button" className="btn btn-secondary" onClick={() => setAddingMessageTo(req.id)}>
                          Добавить сообщение в заявку
                        </button>
                      )}
                    </div>
                  )}
                  {isAdmin() && replyingTo === req.id && req.status !== 'closed' && (
                    <form onSubmit={handleReplySubmit} style={{ marginTop: '20px', padding: '20px', background: 'var(--card-bg)', borderRadius: '8px', border: '1px solid var(--divider)' }}>
                      <div style={{ marginBottom: '14px' }}>
                        <label className="label">Статус заявки</label>
                        <select className="input" value={replyForm.status} onChange={(e) => setReplyForm((prev) => ({ ...prev, status: e.target.value }))} style={{ fontSize: '15px', padding: '10px' }}>
                          {STATUS_OPTIONS.map((o) => (<option key={o.value} value={o.value}>{o.label}</option>))}
                        </select>
                      </div>
                      <div style={{ marginBottom: '14px' }}>
                        <label className="label">Новый ответ (будет добавлен в переписку)</label>
                        <textarea
                          className="input"
                          value={replyForm.admin_comment}
                          onChange={(e) => setReplyForm((prev) => ({ ...prev, admin_comment: e.target.value }))}
                          placeholder="Введите ответ пользователю."
                          rows={4}
                          style={{ fontSize: '15px', padding: '12px' }}
                        />
                      </div>
                      <div style={{ display: 'flex', gap: '10px' }}>
                        <button type="submit" className="btn btn-primary" disabled={replySubmitting}>
                          {replySubmitting ? 'Отправка…' : 'Добавить ответ'}
                        </button>
                        <button type="button" className="btn btn-secondary" onClick={() => { setReplyingTo(null); setReplyForm({ status: 'open', admin_comment: '' }); }}>
                          Закрыть
                        </button>
                      </div>
                    </form>
                  )}
                </div>
              ))}
            </div>
          )}
        </section>
      )}
    </div>
  );
};

export default Support;
