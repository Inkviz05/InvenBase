import React from 'react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import Layout from './Layout';

const logoutMock = vi.fn();

vi.mock('../context/AuthContext', () => ({
  useAuth: () => ({
    user: {
      username: 'tester',
      full_name: 'Test User',
      role: 'admin',
      email: 'tester@example.test',
    },
    logout: logoutMock,
    isAdmin: () => true,
    isResponsible: () => true,
  }),
}));

vi.mock('../context/CartContext', () => ({
  useCart: () => ({
    getCartItemsCount: () => 0,
  }),
}));

vi.mock('../api/notifications', () => ({
  notificationsAPI: {
    getUnreadCount: vi.fn().mockResolvedValue({ count: 0 }),
  },
}));

const renderLayout = () => render(
  <MemoryRouter initialEntries={['/']}>
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<div>Dashboard content</div>} />
      </Route>
    </Routes>
  </MemoryRouter>
);

describe('Layout theme selector', () => {
  it('uses technopark theme by default', () => {
    renderLayout();

    expect(document.documentElement.dataset.theme).toBe('technopark');
    expect(localStorage.getItem('invenbase-theme')).toBe('technopark');
  });

  it('uses saved neon purple theme on first render', () => {
    localStorage.setItem('invenbase-theme', 'neon-purple');

    renderLayout();

    expect(document.documentElement.dataset.theme).toBe('neon-purple');
  });

  it('switches theme from the user dropdown', async () => {
    const user = userEvent.setup();
    renderLayout();

    await user.click(screen.getByRole('button', { name: /tester/i }));
    await user.click(screen.getByText('auto_awesome').closest('button'));

    expect(document.documentElement.dataset.theme).toBe('neon-purple');
    expect(localStorage.getItem('invenbase-theme')).toBe('neon-purple');
  });
});
