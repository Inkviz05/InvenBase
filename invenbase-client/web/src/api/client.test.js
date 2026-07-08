import { beforeEach, describe, expect, it } from 'vitest';
import apiClient from './client';

const resolveWithConfig = (config) => Promise.resolve({
  data: { ok: true },
  status: 200,
  statusText: 'OK',
  headers: {},
  config,
});

describe('apiClient', () => {
  beforeEach(() => {
    localStorage.clear();
  });

  it('uses the configured API base URL', () => {
    expect(apiClient.defaults.baseURL).toBe('/api');
  });

  it('adds bearer token to non-auth requests', async () => {
    localStorage.setItem('token', 'token-123');

    const response = await apiClient.get('/equipment', {
      adapter: resolveWithConfig,
    });

    expect(response.config.headers.Authorization).toBe('Bearer token-123');
  });

  it('does not add bearer token to login request', async () => {
    localStorage.setItem('token', 'token-123');

    const response = await apiClient.post('/auth/login', {
      username: 'admin',
      password: 'secret',
    }, {
      adapter: resolveWithConfig,
    });

    expect(response.config.headers.Authorization).toBeUndefined();
  });

  it('clears auth state and redirects to login on 401', async () => {
    localStorage.setItem('token', 'token-123');
    localStorage.setItem('user', JSON.stringify({ username: 'admin' }));

    await expect(apiClient.get('/users/me', {
      adapter: (config) => Promise.reject({
        config,
        response: { status: 401 },
      }),
    })).rejects.toMatchObject({
      response: { status: 401 },
    });

    expect(localStorage.getItem('token')).toBeNull();
    expect(localStorage.getItem('user')).toBeNull();
    expect(window.location.hash).toBe('#/login');
  });
});
