import React from 'react';
import { act, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { AuthProvider, useAuth } from './AuthContext';
import { authAPI } from '../api/auth';

vi.mock('../api/auth', () => ({
  authAPI: {
    getCurrentUser: vi.fn(),
    login: vi.fn(),
  },
}));

const authSnapshot = {};

const AuthProbe = () => {
  const auth = useAuth();
  Object.assign(authSnapshot, auth);

  return (
    <div>
      <span data-testid="loading">{String(auth.loading)}</span>
      <span data-testid="username">{auth.user?.username || ''}</span>
      <span data-testid="token">{auth.token || ''}</span>
      <span data-testid="is-authenticated">{String(auth.isAuthenticated())}</span>
      <span data-testid="is-admin">{String(auth.isAdmin())}</span>
      <span data-testid="is-responsible">{String(auth.isResponsible())}</span>
    </div>
  );
};

const renderAuth = () => render(
  <AuthProvider>
    <AuthProbe />
  </AuthProvider>
);

describe('AuthProvider', () => {
  beforeEach(() => {
    Object.keys(authSnapshot).forEach((key) => delete authSnapshot[key]);
    vi.clearAllMocks();
  });

  it('starts unauthenticated when localStorage is empty', async () => {
    renderAuth();

    await waitFor(() => expect(screen.getByTestId('loading')).toHaveTextContent('false'));

    expect(authAPI.getCurrentUser).not.toHaveBeenCalled();
    expect(screen.getByTestId('is-authenticated')).toHaveTextContent('false');
    expect(screen.getByTestId('username')).toHaveTextContent('');
  });

  it('restores stored credentials and validates current user', async () => {
    localStorage.setItem('token', 'stored-token');
    localStorage.setItem('user', JSON.stringify({ username: 'cached', role: 'user' }));
    authAPI.getCurrentUser.mockResolvedValue({ username: 'fresh', role: 'admin' });

    renderAuth();

    await waitFor(() => expect(screen.getByTestId('loading')).toHaveTextContent('false'));

    expect(authAPI.getCurrentUser).toHaveBeenCalledTimes(1);
    expect(screen.getByTestId('username')).toHaveTextContent('fresh');
    expect(screen.getByTestId('token')).toHaveTextContent('stored-token');
    expect(screen.getByTestId('is-authenticated')).toHaveTextContent('true');
    expect(screen.getByTestId('is-admin')).toHaveTextContent('true');
    expect(screen.getByTestId('is-responsible')).toHaveTextContent('true');
    expect(JSON.parse(localStorage.getItem('user'))).toEqual({ username: 'fresh', role: 'admin' });
  });

  it('clears auth state when stored token validation fails', async () => {
    localStorage.setItem('token', 'bad-token');
    localStorage.setItem('user', JSON.stringify({ username: 'cached', role: 'user' }));
    authAPI.getCurrentUser.mockRejectedValue(new Error('invalid token'));

    renderAuth();

    await waitFor(() => expect(screen.getByTestId('loading')).toHaveTextContent('false'));

    expect(localStorage.getItem('token')).toBeNull();
    expect(localStorage.getItem('user')).toBeNull();
    expect(screen.getByTestId('is-authenticated')).toHaveTextContent('false');
  });

  it('handles malformed stored user and recovers from token validation', async () => {
    localStorage.setItem('token', 'stored-token');
    localStorage.setItem('user', '{broken-json');
    authAPI.getCurrentUser.mockResolvedValue({ username: 'fresh', role: 'responsible' });

    renderAuth();

    await waitFor(() => expect(screen.getByTestId('loading')).toHaveTextContent('false'));

    expect(screen.getByTestId('username')).toHaveTextContent('fresh');
    expect(screen.getByTestId('is-responsible')).toHaveTextContent('true');
    expect(screen.getByTestId('is-admin')).toHaveTextContent('false');
  });

  it('login stores token and user, logout clears them', async () => {
    authAPI.login.mockResolvedValue({
      token: 'new-token',
      user: { username: 'admin', role: 'admin' },
    });

    renderAuth();
    await waitFor(() => expect(screen.getByTestId('loading')).toHaveTextContent('false'));

    await act(async () => {
      await authSnapshot.login('admin', 'secret');
    });

    expect(authAPI.login).toHaveBeenCalledWith('admin', 'secret');
    expect(localStorage.getItem('token')).toBe('new-token');
    expect(JSON.parse(localStorage.getItem('user'))).toEqual({ username: 'admin', role: 'admin' });
    expect(screen.getByTestId('is-authenticated')).toHaveTextContent('true');

    act(() => {
      authSnapshot.logout();
    });

    expect(localStorage.getItem('token')).toBeNull();
    expect(localStorage.getItem('user')).toBeNull();
    expect(screen.getByTestId('is-authenticated')).toHaveTextContent('false');
  });
});
