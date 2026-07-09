import React from 'react';
import { MemoryRouter } from 'react-router-dom';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import BookingList from './BookingList';
import { bookingsAPI } from '../api/bookings';
import { equipmentAPI } from '../api/equipment';

const { authState, navigateMock } = vi.hoisted(() => ({
  authState: {
    role: 'admin',
  },
  navigateMock: vi.fn(),
}));

vi.mock('../api/bookings', () => ({
  bookingsAPI: {
    getAll: vi.fn(),
    approve: vi.fn(),
    reject: vi.fn(),
    confirmReturn: vi.fn(),
    cancel: vi.fn(),
  },
}));

vi.mock('../api/equipment', () => ({
  equipmentAPI: {
    getByQR: vi.fn(),
  },
}));

vi.mock('../context/AuthContext', () => ({
  useAuth: () => ({
    user: { username: 'tester', role: authState.role },
    isAdmin: () => authState.role === 'admin',
    isResponsible: () => authState.role === 'responsible' || authState.role === 'admin',
  }),
}));

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => navigateMock,
  };
});

const baseBookings = [
  {
    id: 'booking-pending',
    equipment_name: '3D Printer Alpha',
    username: 'student',
    status: 'pending',
    start_date: '2026-08-10T10:00:00.000Z',
    end_date: '2026-08-10T12:00:00.000Z',
    quantity: 1,
    purpose: 'Workshop',
  },
  {
    id: 'booking-approved',
    equipment_name: 'Laser Cutter',
    username: 'mentor',
    status: 'approved',
    start_date: '2026-08-09T10:00:00.000Z',
    end_date: '2020-08-09T12:00:00.000Z',
    quantity: 2,
    purpose: '',
  },
  {
    id: 'booking-awaiting-return',
    equipment_name: 'VR Headset',
    username: 'student',
    status: 'awaiting_return',
    start_date: '2026-08-08T10:00:00.000Z',
    end_date: '2026-08-08T12:00:00.000Z',
    quantity: 1,
  },
  {
    id: 'booking-returned',
    equipment_name: 'Drone Kit',
    username: 'student',
    status: 'returned',
    start_date: '2026-08-07T10:00:00.000Z',
    end_date: '2026-08-07T12:00:00.000Z',
    quantity: 1,
  },
];

const renderBookingList = () => render(
  <MemoryRouter>
    <BookingList />
  </MemoryRouter>
);

describe('BookingList', () => {
  beforeEach(() => {
    authState.role = 'admin';
    navigateMock.mockReset();
    vi.clearAllMocks();
    vi.spyOn(window, 'confirm').mockReturnValue(true);
    vi.spyOn(window, 'alert').mockImplementation(() => {});
    bookingsAPI.getAll.mockResolvedValue(baseBookings);
    bookingsAPI.approve.mockResolvedValue({});
    bookingsAPI.reject.mockResolvedValue({});
    bookingsAPI.confirmReturn.mockResolvedValue({});
    bookingsAPI.cancel.mockResolvedValue({});
    equipmentAPI.getByQR.mockResolvedValue({ id: 'eq-1' });
  });

  it('loads and renders booking list with statuses', async () => {
    renderBookingList();

    expect(await screen.findByText('3D Printer Alpha')).toBeInTheDocument();
    expect(screen.getByText('Laser Cutter')).toBeInTheDocument();
    expect(screen.getByText('VR Headset')).toBeInTheDocument();
    expect(screen.getByText('Drone Kit')).toBeInTheDocument();
    expect(bookingsAPI.getAll).toHaveBeenCalledTimes(1);
  });

  it('filters bookings by pending status', async () => {
    const user = userEvent.setup();
    renderBookingList();

    await screen.findByText('3D Printer Alpha');
    await user.click(screen.getByLabelText('Filter pending bookings'));

    expect(screen.getByText('3D Printer Alpha')).toBeInTheDocument();
    expect(screen.queryByText('Laser Cutter')).not.toBeInTheDocument();
  });

  it('approves pending booking and reloads list', async () => {
    const user = userEvent.setup();
    renderBookingList();

    await screen.findByText('3D Printer Alpha');
    await user.click(screen.getByLabelText('Approve booking booking-pending'));

    await waitFor(() => expect(bookingsAPI.approve).toHaveBeenCalledWith('booking-pending'));
    expect(bookingsAPI.getAll).toHaveBeenCalledTimes(2);
  });

  it('rejects pending booking after confirmation', async () => {
    const user = userEvent.setup();
    renderBookingList();

    await screen.findByText('3D Printer Alpha');
    await user.click(screen.getByLabelText('Reject booking booking-pending'));

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => expect(bookingsAPI.reject).toHaveBeenCalledWith('booking-pending'));
  });

  it('cancels cancellable booking after confirmation', async () => {
    const user = userEvent.setup();
    renderBookingList();

    await screen.findByText('3D Printer Alpha');
    await user.click(screen.getByLabelText('Cancel booking booking-pending'));

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => expect(bookingsAPI.cancel).toHaveBeenCalledWith('booking-pending'));
  });

  it('confirms return for awaiting return booking', async () => {
    const user = userEvent.setup();
    renderBookingList();

    await screen.findByText('VR Headset');
    await user.click(screen.getByLabelText('Confirm return for booking booking-awaiting-return'));

    expect(window.confirm).toHaveBeenCalled();
    await waitFor(() => expect(bookingsAPI.confirmReturn).toHaveBeenCalledWith('booking-awaiting-return'));
  });

  it('shows alert when approve action fails', async () => {
    const user = userEvent.setup();
    bookingsAPI.approve.mockRejectedValue(new Error('network'));

    renderBookingList();

    await screen.findByText('3D Printer Alpha');
    await user.click(screen.getByLabelText('Approve booking booking-pending'));

    await waitFor(() => expect(window.alert).toHaveBeenCalled());
  });

  it('limits regular user actions to pending cancellation and QR flow', async () => {
    const user = userEvent.setup();
    authState.role = 'user';

    renderBookingList();

    await screen.findByText('3D Printer Alpha');
    expect(screen.queryByLabelText('Approve booking booking-pending')).not.toBeInTheDocument();
    expect(screen.queryByLabelText('Reject booking booking-pending')).not.toBeInTheDocument();
    expect(screen.getByLabelText('Cancel booking booking-pending')).toBeInTheDocument();
    expect(screen.queryByLabelText('Cancel booking booking-approved')).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /создать бронирование/i }));
    await user.click(screen.getByText(/код вручную/i));
    await user.type(screen.getByLabelText('Manual equipment code'), 'QR-1');
    await user.click(screen.getByText(/найти/i));

    await waitFor(() => expect(equipmentAPI.getByQR).toHaveBeenCalledWith('QR-1'));
    expect(navigateMock).toHaveBeenCalledWith('/bookings/create?equipment_id=eq-1');
  });
});
