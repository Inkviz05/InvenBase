import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';

const Login = () => {
  const [username, setUsername] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);
  const { login, isAuthenticated } = useAuth();
  const navigate = useNavigate();

  useEffect(() => {
    if (isAuthenticated()) {
      navigate('/');
    }
  }, [isAuthenticated, navigate]);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');
    setLoading(true);

    try {
      console.log('Login: Attempting to login with username:', username);
      const response = await login(username, password);
      console.log('Login: Login successful, navigating to home');
      navigate('/');
    } catch (err) {
      console.error('Login: Login failed:', err);
      const errorMessage = err.response?.data?.error || err.message || 'Ошибка входа. Проверьте логин и пароль.';
      setError(errorMessage);
      console.error('Login error details:', {
        status: err.response?.status,
        data: err.response?.data,
        message: err.message
      });
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{
      minHeight: '100vh',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
      background: 'var(--login-background)',
      padding: '16px'
    }}>
      <div className="card" style={{ maxWidth: '400px', width: '100%', boxShadow: 'var(--login-card-shadow)', border: '1px solid var(--divider)' }}>
        <div style={{ textAlign: 'center', marginBottom: '32px' }}>
          <img 
            src="/logo.png" 
            alt="InvenBase Logo" 
            style={{
              width: '60px',
              height: '60px',
              borderRadius: 'var(--login-logo-radius)',
              margin: '0 auto 16px',
              objectFit: 'contain',
              background: 'var(--login-logo-bg)',
              padding: '8px',
              boxShadow: 'var(--login-logo-shadow)'
            }}
          />
          <h1 style={{ color: 'var(--text-primary)', marginBottom: '8px', fontSize: '28px', fontWeight: 700 }}>InvenBase</h1>
          <p style={{ color: 'var(--text-secondary)', fontSize: '14px' }}>Inventory Management System</p>
        </div>

        {error && <div className="error-message">{error}</div>}

        <form onSubmit={handleSubmit}>
          <label className="label" htmlFor="username">Имя пользователя</label>
          <input
            id="username"
            name="username"
            type="text"
            className="input"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
            required
            autoFocus
            aria-label="Username"
          />

          <label className="label" htmlFor="password">Пароль</label>
          <input
            id="password"
            name="password"
            type="password"
            className="input"
            value={password}
            onChange={(e) => setPassword(e.target.value)}
            required
            aria-label="Password"
          />

          <button
            type="submit"
            className="btn btn-primary"
            style={{ width: '100%' }}
            disabled={loading}
          >
            {loading ? (
              <>
                <div className="spinner" style={{ width: '20px', height: '20px', borderWidth: '2px' }}></div>
                Вход...
              </>
            ) : (
              'Войти'
            )}
          </button>
        </form>
      </div>
    </div>
  );
};

export default Login;

