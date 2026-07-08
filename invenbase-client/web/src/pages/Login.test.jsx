import React from 'react';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import Login from './Login';

const { loginMock, navigateMock, isAuthenticatedMock } = vi.hoisted(() => ({
  loginMock: vi.fn(),
  navigateMock: vi.fn(),
  isAuthenticatedMock: vi.fn(),
}));

vi.mock('../context/AuthContext', () => ({
  useAuth: () => ({
    login: loginMock,
    isAuthenticated: isAuthenticatedMock,
  }),
}));

vi.mock('react-router-dom', () => ({
  useNavigate: () => navigateMock,
}));

describe('Login', () => {
  beforeEach(() => {
    loginMock.mockReset();
    navigateMock.mockReset();
    isAuthenticatedMock.mockReset();
    isAuthenticatedMock.mockReturnValue(false);
  });

  it('renders login form fields', () => {
    render(<Login />);

    expect(screen.getByRole('img', { name: /invenbase logo/i })).toBeInTheDocument();
    expect(screen.getByLabelText('Username')).toBeInTheDocument();
    expect(screen.getByLabelText('Password')).toBeInTheDocument();
    expect(screen.getByRole('button')).toBeEnabled();
  });

  it('submits credentials and navigates home after successful login', async () => {
    const user = userEvent.setup();
    loginMock.mockResolvedValue({
      token: 'token',
      user: { username: 'admin' },
    });

    render(<Login />);

    await user.type(screen.getByLabelText('Username'), 'admin');
    await user.type(screen.getByLabelText('Password'), 'secret');
    await user.click(screen.getByRole('button'));

    await waitFor(() => expect(loginMock).toHaveBeenCalledWith('admin', 'secret'));
    expect(navigateMock).toHaveBeenCalledWith('/');
  });

  it('shows API error message when login fails', async () => {
    const user = userEvent.setup();
    loginMock.mockRejectedValue({
      response: { data: { error: 'Invalid credentials' } },
    });

    render(<Login />);

    await user.type(screen.getByLabelText('Username'), 'admin');
    await user.type(screen.getByLabelText('Password'), 'wrong');
    await user.click(screen.getByRole('button'));

    expect(await screen.findByText('Invalid credentials')).toBeInTheDocument();
    expect(navigateMock).not.toHaveBeenCalledWith('/');
  });

  it('redirects authenticated users away from login page', () => {
    isAuthenticatedMock.mockReturnValue(true);

    render(<Login />);

    expect(navigateMock).toHaveBeenCalledWith('/');
  });
});
