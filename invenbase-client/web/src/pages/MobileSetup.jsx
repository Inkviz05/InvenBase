import React, { useMemo, useState } from 'react';
import { QRCodeSVG } from 'qrcode.react';
import { Navigate } from 'react-router-dom';
import { instanceAPI } from '../api/instance';
import { useAuth } from '../context/AuthContext';

const normalizeServerUrl = (value) => {
  const url = new URL(value.trim());
  if (!['http:', 'https:'].includes(url.protocol)) throw new Error('Нужен HTTP(S) URL');
  if (url.username || url.password || url.search || url.hash) {
    throw new Error('URL не должен содержать credentials, query или fragment');
  }
  if (url.pathname !== '/' && url.pathname !== '') {
    throw new Error('Введите адрес сервера без /api и других путей');
  }
  return url.origin;
};

const MobileSetup = () => {
  const { isAdmin } = useAuth();
  const [serverUrl, setServerUrl] = useState(window.location.origin);
  const [ttl, setTtl] = useState(10);
  const [setup, setSetup] = useState(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');

  const setupLink = useMemo(() => {
    if (!setup) return '';
    const params = new URLSearchParams({ server: setup.serverUrl, code: setup.setup_code });
    return `invenbase://pair?${params.toString()}`;
  }, [setup]);

  if (!isAdmin()) return <Navigate to="/" replace />;

  const createCode = async () => {
    setError('');
    setLoading(true);
    try {
      const normalized = normalizeServerUrl(serverUrl);
      const response = await instanceAPI.createSetupCode(Number(ttl));
      setSetup({ ...response, serverUrl: normalized });
    } catch (requestError) {
      setError(requestError.apiMessage || requestError.message || 'Не удалось создать код');
    } finally {
      setLoading(false);
    }
  };

  const revoke = async () => {
    if (!setup) return;
    setLoading(true);
    try {
      await instanceAPI.revokeSetupCode(setup.id);
      setSetup(null);
    } catch (requestError) {
      setError(requestError.apiMessage || 'Не удалось отозвать код');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{ maxWidth: 720, margin: '0 auto' }}>
      <h1>Подключение мобильного приложения</h1>
      <p style={{ color: 'var(--text-secondary)' }}>
        Создайте одноразовую ссылку для привязки приложения к этому серверу. QR не содержит
        логин, пароль или JWT.
      </p>

      <div className="card" style={{ padding: 24 }}>
        <label htmlFor="mobile-server-url">Публичный HTTPS-адрес сервера</label>
        <input
          id="mobile-server-url"
          className="form-control"
          value={serverUrl}
          onChange={(event) => setServerUrl(event.target.value)}
          disabled={loading}
          style={{ marginTop: 8, marginBottom: 16 }}
        />

        <label htmlFor="mobile-code-ttl">Срок действия</label>
        <select
          id="mobile-code-ttl"
          className="form-control"
          value={ttl}
          onChange={(event) => setTtl(event.target.value)}
          disabled={loading}
          style={{ marginTop: 8, marginBottom: 20 }}
        >
          <option value={5}>5 минут</option>
          <option value={10}>10 минут</option>
          <option value={30}>30 минут</option>
          <option value={60}>60 минут</option>
        </select>

        <button type="button" className="btn btn-primary" onClick={createCode} disabled={loading}>
          {loading ? 'Подождите…' : 'Создать QR и код'}
        </button>
        {error && <div style={{ color: 'var(--error)', marginTop: 16 }}>{error}</div>}
      </div>

      {setup && (
        <div className="card" style={{ padding: 24, marginTop: 24, textAlign: 'center' }}>
          <QRCodeSVG value={setupLink} size={280} level="M" includeMargin />
          <div style={{ fontFamily: 'monospace', fontSize: 20, marginTop: 16 }}>
            {setup.setup_code}
          </div>
          <div style={{ color: 'var(--text-secondary)', marginTop: 8 }}>
            Действует до {new Date(setup.expires_at).toLocaleString()}
          </div>
          <div style={{ display: 'flex', gap: 12, justifyContent: 'center', marginTop: 20, flexWrap: 'wrap' }}>
            <button type="button" className="btn btn-secondary" onClick={() => navigator.clipboard.writeText(setupLink)}>
              Копировать ссылку
            </button>
            <button type="button" className="btn btn-danger" onClick={revoke} disabled={loading}>
              Отозвать код
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default MobileSetup;
