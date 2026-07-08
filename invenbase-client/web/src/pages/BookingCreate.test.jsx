import React from 'react';
import { MemoryRouter } from 'react-router-dom';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import BookingCreate from './BookingCreate';
import { bookingsAPI } from '../api/bookings';
import { equipmentAPI } from '../api/equipment';

const { authState, navigateMock, searchParamsState } = vi.hoisted(() => ({
  authState: {
    role: 'admin',
  },
  navigateMock: vi.fn(),
  searchParamsState: {
    params: new URLSearchParams(),
  },
}));

vi.mock('../api/bookings', () => ({
  bookingsAPI: {
    create: vi.fn(),
  },
}));

vi.mock('../api/equipment', () => ({
  equipmentAPI: {
    getAll: vi.fn(),
    getById: vi.fn(),
    getByQR: vi.fn(),
  },
}));

vi.mock('../context/AuthContext', () => ({
  useAuth: () => ({
    isAdmin: () => authState.role === 'admin',
    isResponsible: () => authState.role === 'responsible' || authState.role === 'admin',
  }),
}));

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => navigateMock,
    useSearchParams: () => [searchParamsState.params],
  };
});

const equipmentItems = [
  {
    id: 'eq-1',
    name: '3D Printer Alpha',
    quantity: 4,
    available_quantity: 2,
  },
  {
    id: 'eq-2',
    name: 'Laser Cutter',
    quantity: 1,
    available_quantity: 1,
  },
];

const renderBookingCreate = () => render(
  <MemoryRouter>
    <BookingCreate />
  </MemoryRouter>
);

const fillBookingForm = async (user) => {
  await user.clear(screen.getByLabelText('Quantity'));
  await user.type(screen.getByLabelText('Quantity'), '2');
  await user.type(screen.getByLabelText('Start date'), '2026-08-10T10:00');
  await user.type(screen.getByLabelText('End date'), '2026-08-10T12:00');
  await user.selectOptions(screen.getByLabelText('Permission type'), 'external');
  await user.type(screen.getByLabelText('Purpose'), 'Robotics workshop');
};

describe('BookingCreate', () => {
  beforeEach(() => {
    authState.role = 'admin';
    navigateMock.mockReset();
    searchParamsState.params = new URLSearchParams();
    equipmentAPI.getAll.mockReset();
    equipmentAPI.getById.mockReset();
    equipmentAPI.getByQR.mockReset();
    bookingsAPI.create.mockReset();
    equipmentAPI.getAll.mockResolvedValue(equipmentItems);
    equipmentAPI.getById.mockResolvedValue(equipmentItems[0]);
    equipmentAPI.getByQR.mockResolvedValue(equipmentItems[0]);
    bookingsAPI.create.mockResolvedValue({ id: 'booking-1' });

    Object.defineProperty(navigator, 'mediaDevices', {
      configurable: true,
      value: {
        getUserMedia: vi.fn().mockRejectedValue(new Error('No camera')),
        enumerateDevices: vi.fn().mockResolvedValue([]),
      },
    });
  });

  it('loads equipment options for admin users', async () => {
    renderBookingCreate();

    expect(await screen.findByText('3D Printer Alpha (Доступно: 2)')).toBeInTheDocument();
    expect(screen.getByText('Laser Cutter (Доступно: 1)')).toBeInTheDocument();
    expect(equipmentAPI.getAll).toHaveBeenCalledTimes(1);
  });

  it('creates booking for selected equipment', async () => {
    const user = userEvent.setup();
    renderBookingCreate();

    await screen.findByText('3D Printer Alpha (Доступно: 2)');
    await user.selectOptions(screen.getByLabelText('Equipment'), 'eq-1');
    await fillBookingForm(user);
    await user.click(screen.getByRole('button', { name: /создать бронирование/i }));

    await waitFor(() => expect(bookingsAPI.create).toHaveBeenCalledTimes(1));
    expect(bookingsAPI.create).toHaveBeenCalledWith({
      equipment_id: 'eq-1',
      group_id: null,
      quantity: 2,
      start_date: new Date('2026-08-10T10:00').toISOString(),
      end_date: new Date('2026-08-10T12:00').toISOString(),
      purpose: 'Robotics workshop',
      permission_type: 'external',
    });
    expect(navigateMock).toHaveBeenCalledWith('/bookings');
  });

  it('shows API error when booking creation fails', async () => {
    const user = userEvent.setup();
    bookingsAPI.create.mockRejectedValue({
      response: { data: { error: 'Equipment is not available' } },
    });

    renderBookingCreate();

    await screen.findByText('3D Printer Alpha (Доступно: 2)');
    await user.selectOptions(screen.getByLabelText('Equipment'), 'eq-1');
    await fillBookingForm(user);
    await user.click(screen.getByRole('button', { name: /создать бронирование/i }));

    expect(await screen.findByText('Equipment is not available')).toBeInTheDocument();
    expect(navigateMock).not.toHaveBeenCalledWith('/bookings');
  });

  it('loads selected equipment from URL for regular users', async () => {
    authState.role = 'user';
    searchParamsState.params = new URLSearchParams('equipment_id=eq-1');

    renderBookingCreate();

    expect(await screen.findByText(/3D Printer Alpha/)).toBeInTheDocument();
    expect(equipmentAPI.getById).toHaveBeenCalledWith('eq-1');
    expect(equipmentAPI.getAll).not.toHaveBeenCalled();
  });

  it('lets regular users select equipment by manual QR code', async () => {
    const user = userEvent.setup();
    authState.role = 'user';

    renderBookingCreate();

    await user.click(screen.getByText(/QR-код вручную/i));
    await user.type(screen.getByLabelText('Manual QR code'), 'QR-1');
    await user.click(screen.getByText(/найти оборудование/i));

    await waitFor(() => expect(equipmentAPI.getByQR).toHaveBeenCalledWith('QR-1'));
    expect(await screen.findByText(/3D Printer Alpha/)).toBeInTheDocument();
  });
});
