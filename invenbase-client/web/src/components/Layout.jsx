import React, { useState } from 'react';
import { Outlet, Link, useLocation, useNavigate } from 'react-router-dom';
import { useAuth } from '../context/AuthContext';
import { useCart } from '../context/CartContext';
import { notificationsAPI } from '../api/notifications';

const Layout = () => {
  const { user, logout, isAdmin, isResponsible } = useAuth();
  const { getCartItemsCount } = useCart();
  const location = useLocation();
  const navigate = useNavigate();
  const [unreadCount, setUnreadCount] = useState(0);
  const [showMenu, setShowMenu] = useState(false);
  const [showUserDetails, setShowUserDetails] = useState(false);
  const [showMobileMenu, setShowMobileMenu] = useState(false);
  const [isMobile, setIsMobile] = useState(window.innerWidth <= 768);
  const cartCount = getCartItemsCount();
  
  // Проверяем, находимся ли мы в Android WebView
  const isAndroidWebView = typeof window !== 'undefined' && 
    (window.ANDROID_WEBVIEW === true || 
     (window.navigator.userAgent.includes('Android') && 
      (window.navigator.userAgent.includes('wv') || window.location.protocol === 'file:')));

  React.useEffect(() => {
    const handleResize = () => {
      setIsMobile(window.innerWidth <= 768);
      if (window.innerWidth > 768) {
        setShowMobileMenu(false);
      }
    };

    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  React.useEffect(() => {
    const fetchUnreadCount = async () => {
      try {
        const response = await notificationsAPI.getUnreadCount();
        setUnreadCount(response.count || 0);
      } catch (error) {
        console.error('Failed to fetch unread count:', error);
      }
    };

    if (user) {
      fetchUnreadCount();
      const interval = setInterval(fetchUnreadCount, 30000); // Обновляем каждые 30 секунд
      return () => clearInterval(interval);
    }
  }, [user]);

  React.useEffect(() => {
    const handleClickOutside = (event) => {
      if (showMenu && !event.target.closest('.user-menu')) {
        setShowMenu(false);
      }
    };

    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [showMenu]);

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  const isActive = (path) => location.pathname === path;

  return (
    <div style={{ minHeight: '100vh', display: 'flex', flexDirection: 'column' }}>
      {/* Header - одинаковый для веб и Android */}
      <header className="app-header" style={{
        background: 'var(--surface)',
        color: 'var(--text-primary)',
        padding: '16px 24px',
        boxShadow: '0 2px 8px rgba(0,0,0,0.3)',
        position: 'sticky',
        top: 0,
        zIndex: 1000,
        borderBottom: '1px solid var(--divider)'
      }}>
        <div className="app-header-inner">
          <div style={{ display: 'flex', alignItems: 'center', gap: '12px', minWidth: 0 }}>
            <button
              onClick={() => setShowMobileMenu(!showMobileMenu)}
              style={{
                background: 'none',
                border: 'none',
                color: 'white',
                cursor: 'pointer',
                display: isMobile ? 'flex' : 'none',
                alignItems: 'center',
                padding: '8px'
              }}
              className="mobile-menu-button"
            >
              <span className="material-icons">menu</span>
            </button>
            <h1 style={{ margin: 0, fontSize: '20px', fontWeight: 600, color: 'var(--text-primary)' }}>InvenBase</h1>
          </div>
          <div className="app-header-right">
              <Link
                to="/notifications"
                style={{
                  color: 'var(--text-primary)',
                  textDecoration: 'none',
                  position: 'relative',
                  display: 'flex',
                  alignItems: 'center'
                }}
              >
                <span className="material-icons">notifications</span>
                {unreadCount > 0 && (
                  <span style={{
                    position: 'absolute',
                    top: '-8px',
                    right: '-8px',
                    background: 'var(--error)',
                    borderRadius: '50%',
                    width: '20px',
                    height: '20px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    fontSize: '12px',
                    fontWeight: 'bold'
                  }}>
                    {unreadCount > 9 ? '9+' : unreadCount}
                  </span>
                )}
              </Link>
              {cartCount > 0 && (
                <Link
                  to="/bookings/bulk"
                  style={{
                    color: 'var(--text-primary)',
                    textDecoration: 'none',
                    position: 'relative',
                    display: 'flex',
                    alignItems: 'center',
                    cursor: 'pointer'
                  }}
                  title="Корзина бронирований"
                >
                  <span className="material-icons">shopping_cart</span>
                  <span style={{
                    position: 'absolute',
                    top: '-8px',
                    right: '-8px',
                    background: '#388E3C',
                    color: 'white',
                    borderRadius: '50%',
                    width: '20px',
                    height: '20px',
                    display: 'flex',
                    alignItems: 'center',
                    justifyContent: 'center',
                    fontSize: '12px',
                    fontWeight: 'bold'
                  }}>
                    {cartCount > 9 ? '9+' : cartCount}
                  </span>
                </Link>
              )}
            <div style={{ position: 'relative' }} className="user-menu">
              <button
                onClick={() => setShowMenu(!showMenu)}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--text-primary)',
                  cursor: 'pointer',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px',
                  padding: '8px'
                }}
              >
                <span className="material-icons">account_circle</span>
                <span className="app-header-username">{user?.username || 'Пользователь'}</span>
                <span className="material-icons">{showMenu ? 'expand_less' : 'expand_more'}</span>
              </button>
              {showMenu && (
                <div 
                  style={{
                    position: 'absolute',
                    top: '100%',
                    right: 0,
                    background: 'var(--card-bg)',
                    boxShadow: '0 4px 12px rgba(0,0,0,0.5)',
                    borderRadius: '12px',
                    marginTop: '8px',
                    minWidth: '200px',
                    zIndex: 1001,
                    overflow: 'hidden',
                    border: '1px solid rgba(255,255,255,0.1)'
                  }}
                  onClick={(e) => e.stopPropagation()}
                >
                  <button
                    onClick={handleLogout}
                    style={{
                      width: '100%',
                      padding: '12px',
                      background: 'none',
                      border: 'none',
                      textAlign: 'left',
                      cursor: 'pointer',
                      color: 'var(--text-primary)',
                      display: 'flex',
                      alignItems: 'center',
                      gap: '8px',
                      transition: 'background 0.2s ease'
                    }}
                    onMouseEnter={(e) => e.currentTarget.style.background = 'var(--sidebar-hover)'}
                    onMouseLeave={(e) => e.currentTarget.style.background = 'none'}
                  >
                    <span className="material-icons">logout</span>
                    Выйти
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>
      </header>

      <div className="layout-body" style={{ display: 'flex', flex: 1, position: 'relative', minWidth: 0, overflowX: 'hidden' }}>
        {/* Mobile Menu Overlay */}
        {showMobileMenu && isMobile && (
          <div
            className="mobile-menu-overlay"
            onClick={() => setShowMobileMenu(false)}
            style={{
              position: 'fixed',
              top: 0,
              left: 0,
              right: 0,
              bottom: 0,
              background: 'rgba(0, 0, 0, 0.5)',
              zIndex: 999
            }}
          />
        )}

        {/* Sidebar */}
        <nav 
          className={`sidebar ${showMobileMenu ? 'sidebar-open' : ''}`}
          style={{
            width: '250px',
            background: 'var(--sidebar-bg)',
            borderRight: 'none',
            padding: '0',
            minHeight: 'calc(100vh - 64px)',
            transition: 'transform 0.3s ease'
          }}
        >
          {/* Logo/Gradient Section */}
          <div style={{ 
            padding: '20px 16px', 
            background: `linear-gradient(135deg, var(--sidebar-gradient-start) 0%, var(--sidebar-gradient-end) 100%)`,
            borderBottom: '1px solid rgba(255,255,255,0.1)'
          }}>
            <div style={{ 
              display: 'flex', 
              alignItems: 'center', 
              gap: '14px', 
              color: 'white'
            }}>
              <img 
                src="/logo.png" 
                alt="InvenBase Logo" 
                style={{
                  width: '64px',
                  height: '64px',
                  borderRadius: '10px',
                  objectFit: 'contain',
                  background: 'rgba(255,255,255,0.12)',
                  padding: '6px',
                  flexShrink: 0
                }}
              />
              <div style={{ flex: 1, minWidth: 0 }}>
                <div style={{ 
                  fontWeight: 700, 
                  fontSize: '19px', 
                  letterSpacing: '0.3px',
                  lineHeight: '1.3',
                  marginBottom: '2px'
                }}>
                  InvenBase
                </div>
                <div style={{ 
                  fontSize: '10px', 
                  opacity: 0.9, 
                  letterSpacing: '0.2px',
                  fontWeight: 400,
                  textTransform: 'none',
                  lineHeight: '1.2'
                }}>
                  inventory base system
                </div>
              </div>
            </div>
          </div>

          {/* User Profile Section */}
          <div style={{ borderBottom: '1px solid rgba(255,255,255,0.1)' }}>
            <div 
              style={{ 
                padding: '20px', 
                cursor: 'pointer',
                transition: 'background 0.2s ease'
              }}
              onClick={() => setShowUserDetails(!showUserDetails)}
              onMouseEnter={(e) => e.currentTarget.style.background = 'var(--sidebar-hover)'}
              onMouseLeave={(e) => e.currentTarget.style.background = 'transparent'}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: '12px', color: 'white' }}>
                <div style={{
                  width: '40px',
                  height: '40px',
                  borderRadius: '50%',
                  background: `linear-gradient(135deg, var(--primary-color) 0%, var(--secondary-color) 100%)`,
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  color: 'white',
                  fontWeight: 'bold',
                  fontSize: '16px'
                }}>
                  {user?.username?.charAt(0).toUpperCase() || 'U'}
                </div>
                <div style={{ flex: 1 }}>
                  <div style={{ fontWeight: 500, fontSize: '14px' }}>{user?.full_name || user?.username || 'Пользователь'}</div>
                  <div style={{ fontSize: '12px', opacity: 0.7 }}>{user?.role === 'admin' ? 'Администратор' : user?.role === 'responsible' ? 'Ответственный' : 'Пользователь'}</div>
                </div>
                <span 
                  className="material-icons" 
                  style={{ 
                    fontSize: '18px', 
                    opacity: 0.7,
                    transform: showUserDetails ? 'rotate(180deg)' : 'rotate(0deg)',
                    transition: 'transform 0.2s ease'
                  }}
                >
                  arrow_drop_down
                </span>
              </div>
            </div>
            {showUserDetails && (
              <div style={{ 
                padding: '12px 20px 20px 20px', 
                background: 'rgba(0,0,0,0.2)',
                borderTop: '1px solid rgba(255,255,255,0.05)'
              }}>
                {user?.email && (
                  <div style={{ 
                    fontSize: '13px', 
                    color: 'rgba(255,255,255,0.8)', 
                    marginBottom: '8px',
                    display: 'flex',
                    alignItems: 'center',
                    gap: '8px'
                  }}>
                    <span className="material-icons" style={{ fontSize: '16px', opacity: 0.7 }}>email</span>
                    {user.email}
                  </div>
                )}
                <div style={{ 
                  fontSize: '12px', 
                  color: 'rgba(255,255,255,0.6)',
                  display: 'flex',
                  alignItems: 'center',
                  gap: '8px'
                }}>
                  <span className="material-icons" style={{ fontSize: '16px', opacity: 0.7 }}>badge</span>
                  {user?.role === 'admin' ? 'Администратор' : user?.role === 'responsible' ? 'Ответственный' : 'Пользователь'}
                </div>
              </div>
            )}
          </div>

          <Link
            to="/"
            onClick={() => setShowMobileMenu(false)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              padding: '12px 24px',
              color: isActive('/') ? 'white' : 'rgba(255,255,255,0.8)',
              textDecoration: 'none',
              fontWeight: isActive('/') ? 600 : 400,
              background: isActive('/') ? 'var(--sidebar-hover)' : 'transparent',
              borderLeft: isActive('/') ? '3px solid var(--primary-color)' : '3px solid transparent',
              transition: 'all 0.2s ease',
              textTransform: 'uppercase',
              fontSize: '13px',
              letterSpacing: '0.5px'
            }}
          >
            <span className="material-icons" style={{ fontSize: '20px' }}>dashboard</span>
            Главная
          </Link>
          <Link
            to="/equipment"
            onClick={() => setShowMobileMenu(false)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              padding: '12px 24px',
              color: isActive('/equipment') ? 'white' : 'rgba(255,255,255,0.8)',
              textDecoration: 'none',
              fontWeight: isActive('/equipment') ? 600 : 400,
              background: isActive('/equipment') ? 'var(--sidebar-hover)' : 'transparent',
              borderLeft: isActive('/equipment') ? '3px solid var(--primary-color)' : '3px solid transparent',
              transition: 'all 0.2s ease',
              textTransform: 'uppercase',
              fontSize: '13px',
              letterSpacing: '0.5px'
            }}
          >
            <span className="material-icons" style={{ fontSize: '20px' }}>inventory</span>
            Оборудование
          </Link>
          <Link
            to="/bookings"
            onClick={() => setShowMobileMenu(false)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              padding: '12px 24px',
              color: isActive('/bookings') ? 'white' : 'rgba(255,255,255,0.8)',
              textDecoration: 'none',
              fontWeight: isActive('/bookings') ? 600 : 400,
              background: isActive('/bookings') ? 'var(--sidebar-hover)' : 'transparent',
              borderLeft: isActive('/bookings') ? '3px solid var(--primary-color)' : '3px solid transparent',
              transition: 'all 0.2s ease',
              textTransform: 'uppercase',
              fontSize: '13px',
              letterSpacing: '0.5px'
            }}
          >
            <span className="material-icons" style={{ fontSize: '20px' }}>event</span>
            Бронирования
          </Link>
          <Link
            to="/scanner"
            onClick={() => setShowMobileMenu(false)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              padding: '12px 24px',
              color: isActive('/scanner') ? 'white' : 'rgba(255,255,255,0.8)',
              textDecoration: 'none',
              fontWeight: isActive('/scanner') ? 600 : 400,
              background: isActive('/scanner') ? 'var(--sidebar-hover)' : 'transparent',
              borderLeft: isActive('/scanner') ? '3px solid var(--primary-color)' : '3px solid transparent',
              transition: 'all 0.2s ease',
              textTransform: 'uppercase',
              fontSize: '13px',
              letterSpacing: '0.5px'
            }}
          >
            <span className="material-icons" style={{ fontSize: '20px' }}>qr_code_scanner</span>
            Сканер QR
          </Link>
          <Link
            to="/support"
            onClick={() => setShowMobileMenu(false)}
            style={{
              display: 'flex',
              alignItems: 'center',
              gap: '12px',
              padding: '12px 24px',
              color: isActive('/support') ? 'white' : 'rgba(255,255,255,0.8)',
              textDecoration: 'none',
              fontWeight: isActive('/support') ? 600 : 400,
              background: isActive('/support') ? 'var(--sidebar-hover)' : 'transparent',
              borderLeft: isActive('/support') ? '3px solid var(--primary-color)' : '3px solid transparent',
              transition: 'all 0.2s ease',
              textTransform: 'uppercase',
              fontSize: '13px',
              letterSpacing: '0.5px'
            }}
          >
            <span className="material-icons" style={{ fontSize: '20px' }}>support_agent</span>
            Поддержка
          </Link>
          {(isAdmin() || isResponsible()) && (
            <>
              <div style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid rgba(255,255,255,0.1)' }}>
                <div style={{ padding: '12px 24px', fontSize: '11px', color: 'rgba(255,255,255,0.5)', textTransform: 'uppercase', letterSpacing: '1px' }}>
                  Администрирование
                </div>
              </div>
              <Link
                to="/categories"
                onClick={() => setShowMobileMenu(false)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px',
                  padding: '12px 24px',
                  color: isActive('/categories') ? 'white' : 'rgba(255,255,255,0.8)',
                  textDecoration: 'none',
                  fontWeight: isActive('/categories') ? 600 : 400,
                  background: isActive('/categories') ? 'var(--sidebar-hover)' : 'transparent',
                  borderLeft: isActive('/categories') ? '3px solid var(--primary-color)' : '3px solid transparent',
                  transition: 'all 0.2s ease',
                  textTransform: 'uppercase',
                  fontSize: '13px',
                  letterSpacing: '0.5px'
                }}
              >
                <span className="material-icons" style={{ fontSize: '20px' }}>category</span>
                Категории
              </Link>
              <Link
                to="/squads"
                onClick={() => setShowMobileMenu(false)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px',
                  padding: '12px 24px',
                  color: isActive('/squads') ? 'white' : 'rgba(255,255,255,0.8)',
                  textDecoration: 'none',
                  fontWeight: isActive('/squads') ? 600 : 400,
                  background: isActive('/squads') ? 'var(--sidebar-hover)' : 'transparent',
                  borderLeft: isActive('/squads') ? '3px solid var(--primary-color)' : '3px solid transparent',
                  transition: 'all 0.2s ease',
                  textTransform: 'uppercase',
                  fontSize: '13px',
                  letterSpacing: '0.5px'
                }}
              >
                <span className="material-icons" style={{ fontSize: '20px' }}>groups</span>
                Сквады
              </Link>
              <Link
                to="/reports"
                onClick={() => setShowMobileMenu(false)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px',
                  padding: '12px 24px',
                  color: isActive('/reports') ? 'white' : 'rgba(255,255,255,0.8)',
                  textDecoration: 'none',
                  fontWeight: isActive('/reports') ? 600 : 400,
                  background: isActive('/reports') ? 'var(--sidebar-hover)' : 'transparent',
                  borderLeft: isActive('/reports') ? '3px solid var(--primary-color)' : '3px solid transparent',
                  transition: 'all 0.2s ease',
                  textTransform: 'uppercase',
                  fontSize: '13px',
                  letterSpacing: '0.5px'
                }}
              >
                <span className="material-icons" style={{ fontSize: '20px' }}>assessment</span>
                Отчёты
              </Link>
              <Link
                to="/logs"
                onClick={() => setShowMobileMenu(false)}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: '12px',
                  padding: '12px 24px',
                  color: isActive('/logs') ? 'white' : 'rgba(255,255,255,0.8)',
                  textDecoration: 'none',
                  fontWeight: isActive('/logs') ? 600 : 400,
                  background: isActive('/logs') ? 'var(--sidebar-hover)' : 'transparent',
                  borderLeft: isActive('/logs') ? '3px solid var(--primary-color)' : '3px solid transparent',
                  transition: 'all 0.2s ease',
                  textTransform: 'uppercase',
                  fontSize: '13px',
                  letterSpacing: '0.5px'
                }}
              >
                <span className="material-icons" style={{ fontSize: '20px' }}>description</span>
                Журнал
              </Link>
            </>
          )}
          {isAdmin() && (
            <Link
              to="/users"
              onClick={() => setShowMobileMenu(false)}
              style={{
                display: 'flex',
                alignItems: 'center',
                gap: '12px',
                padding: '12px 24px',
                color: isActive('/users') ? 'white' : 'rgba(255,255,255,0.8)',
                textDecoration: 'none',
                fontWeight: isActive('/users') ? 600 : 400,
                background: isActive('/users') ? 'var(--sidebar-hover)' : 'transparent',
                borderLeft: isActive('/users') ? '3px solid var(--primary-color)' : '3px solid transparent',
                transition: 'all 0.2s ease',
                textTransform: 'uppercase',
                fontSize: '13px',
                letterSpacing: '0.5px'
              }}
            >
              <span className="material-icons" style={{ fontSize: '20px' }}>people</span>
              Пользователи
            </Link>
          )}
        </nav>

        {/* Main Content */}
        <main className="main-content" style={{ flex: 1, minWidth: 0, padding: '24px', overflowX: 'hidden', overflowY: 'auto', background: 'var(--background)', boxSizing: 'border-box' }}>
          <Outlet />
        </main>
      </div>
    </div>
  );
};

export default Layout;

