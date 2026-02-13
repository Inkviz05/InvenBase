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

const Support = () => {
  const { isAdmin } = useAuth();
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
    // Не подставляем старый ответ — каждый раз добавляем новое сообщение в переписку
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
      // Очищаем только текст, форму не закрываем — можно добавить ещё ответ
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

  // Собрать переписку: первое сообщение + сообщения из messages; для старых заявок — admin_comment
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

  return (
    <div style={{ maxWidth: '900px' }}>
      <h1 style={{ marginBottom: '24px', fontSize: '24px', fontWeight: 600 }}>
        Техническая поддержка
      </h1>

      <section
        style={{
          background: 'var(--card-bg)',
          borderRadius: '12px',
          padding: '24px',
          marginBottom: '24px',
          border: '1px solid var(--divider)',
        }}
      >
        <h2 style={{ margin: '0 0 12px', fontSize: '18px', display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span className="material-icons" style={{ fontSize: '22px', color: 'var(--primary-color)' }}>contact_support</span>
          Как получить помощь
        </h2>
        <p style={{ margin: 0, color: 'var(--text-secondary)', lineHeight: 1.6 }}>
          Опишите проблему в форме ниже — мы создадим заявку и ответим вам. В одной заявке можно вести переписку до её закрытия: если ответ не помог — нажмите «Добавить сообщение в заявку» и опишите, что ещё не решено. Создавать новую заявку нужно только после того, как заявка получит статус «Закрыта».
        </p>
      </section>

      <section
        style={{
          background: 'var(--card-bg)',
          borderRadius: '12px',
          padding: '24px',
          marginBottom: '24px',
          border: '1px solid var(--divider)',
        }}
      >
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
          <div style={{ marginBottom: '16px' }}>
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

      <section
        style={{
          background: 'var(--card-bg)',
          borderRadius: '12px',
          padding: '24px',
          border: '1px solid var(--divider)',
        }}
      >
        <h2 style={{ margin: '0 0 16px', fontSize: '18px', display: 'flex', alignItems: 'center', gap: '8px' }}>
          <span className="material-icons" style={{ fontSize: '22px', color: 'var(--primary-color)' }}>list_alt</span>
          {isAdmin() ? 'Все заявки' : 'Мои заявки'}
        </h2>
        {loading ? (
          <div className="loading"><div className="spinner"></div></div>
        ) : requests.length === 0 ? (
          <p style={{ color: 'var(--text-secondary)', margin: 0 }}>Заявок пока нет.</p>
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
            {requests.map((req) => (
              <div
                key={req.id}
                style={{
                  padding: '16px',
                  background: 'var(--background)',
                  borderRadius: '8px',
                  border: '1px solid var(--divider)',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', flexWrap: 'wrap', gap: '8px', marginBottom: '8px' }}>
                  <strong style={{ fontSize: '15px' }}>{req.subject}</strong>
                  <span
                    style={{
                      fontSize: '12px',
                      padding: '4px 8px',
                      borderRadius: '6px',
                      background: req.status === 'closed' ? 'var(--divider)' : 'var(--primary-color)',
                      color: 'white',
                      opacity: req.status === 'closed' ? 0.8 : 1,
                    }}
                  >
                    {STATUS_LABELS[req.status] || req.status}
                  </span>
                </div>
                {isAdmin() && req.user_name && (
                  <div style={{ fontSize: '13px', color: 'var(--text-secondary)', marginBottom: '6px' }}>
                    От: {req.user_name}
                  </div>
                )}
                <div style={{ marginBottom: '8px' }}>
                  {getThread(req).map((item, idx) => (
                    <div
                      key={idx}
                      style={{
                        marginTop: idx > 0 ? '12px' : 0,
                        padding: '10px',
                        background: item.isStaff ? 'rgba(56, 142, 60, 0.15)' : 'var(--background)',
                        borderRadius: '8px',
                        fontSize: '13px',
                        borderLeft: item.isStaff ? '3px solid var(--primary-color)' : '3px solid transparent',
                      }}
                    >
                      {item.isStaff && <strong style={{ display: 'block', marginBottom: '4px' }}>Ответ поддержки</strong>}
                      <span style={{ whiteSpace: 'pre-wrap' }}>{item.message}</span>
                      <div style={{ marginTop: '4px', fontSize: '11px', color: 'var(--text-secondary)' }}>
                        {format(new Date(item.created_at), 'd MMM yyyy, HH:mm', { locale: ru })}
                      </div>
                    </div>
                  ))}
                </div>
                {!isAdmin() && req.status !== 'closed' && (
                  <>
                    {addingMessageTo === req.id ? (
                      <form onSubmit={(e) => handleAddMessage(e, req.id)} style={{ marginTop: '12px' }}>
                        <textarea
                          className="input"
                          value={addMessageText}
                          onChange={(e) => setAddMessageText(e.target.value)}
                          placeholder="Дополните заявку — опишите, что ещё не решено"
                          rows={3}
                          required
                        />
                        <div style={{ display: 'flex', gap: '8px', marginTop: '8px' }}>
                          <button type="submit" className="btn btn-primary" disabled={addMessageSubmitting}>
                            {addMessageSubmitting ? 'Отправка…' : 'Отправить'}
                          </button>
                          <button type="button" className="btn btn-secondary" onClick={() => { setAddingMessageTo(null); setAddMessageText(''); }}>
                            Отмена
                          </button>
                        </div>
                      </form>
                    ) : (
                      <button
                        type="button"
                        className="btn btn-secondary"
                        style={{ fontSize: '13px', padding: '6px 12px', marginTop: '8px' }}
                        onClick={() => setAddingMessageTo(req.id)}
                      >
                        Добавить сообщение в заявку
                      </button>
                    )}
                  </>
                )}
                <div style={{ marginTop: '8px', display: 'flex', justifyContent: 'space-between', alignItems: 'center', flexWrap: 'wrap', gap: '8px' }}>
                  <span style={{ fontSize: '12px', color: 'var(--text-secondary)' }}>
                    {format(new Date(req.created_at), 'd MMMM yyyy, HH:mm', { locale: ru })}
                  </span>
                  {isAdmin() && (
                    <button
                      type="button"
                      className="btn btn-primary"
                      style={{ fontSize: '13px', padding: '6px 12px' }}
                      onClick={() => openReply(req)}
                    >
                      Ответить / добавить ответ
                    </button>
                  )}
                </div>
                {isAdmin() && replyingTo === req.id && (
                  <form onSubmit={handleReplySubmit} style={{ marginTop: '16px', padding: '16px', background: 'var(--card-bg)', borderRadius: '8px', border: '1px solid var(--divider)' }}>
                    <div style={{ marginBottom: '12px' }}>
                      <label className="label">Статус заявки</label>
                      <select
                        className="input"
                        value={replyForm.status}
                        onChange={(e) => setReplyForm((prev) => ({ ...prev, status: e.target.value }))}
                      >
                        {STATUS_OPTIONS.map((o) => (
                          <option key={o.value} value={o.value}>{o.label}</option>
                        ))}
                      </select>
                    </div>
                    <div style={{ marginBottom: '12px' }}>
                      <label className="label">Новый ответ (будет добавлен в переписку)</label>
                      <textarea
                        className="input"
                        value={replyForm.admin_comment}
                        onChange={(e) => setReplyForm((prev) => ({ ...prev, admin_comment: e.target.value }))}
                        placeholder="Введите ответ пользователю. Можно отправить несколько ответов подряд."
                        rows={4}
                      />
                    </div>
                    <div style={{ display: 'flex', gap: '8px' }}>
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
    </div>
  );
};

export default Support;
