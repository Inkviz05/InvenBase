import React from 'react';
import { MemoryRouter } from 'react-router-dom';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import EquipmentList from './EquipmentList';
import { equipmentAPI } from '../api/equipment';
import { squadsAPI } from '../api/squads';

const { authState, cartState, navigateMock } = vi.hoisted(() => ({
  authState: {
    role: 'admin',
  },
  cartState: {
    addToCart: vi.fn(),
    inCartIds: new Set(),
  },
  navigateMock: vi.fn(),
}));

vi.mock('../api/equipment', () => ({
  equipmentAPI: {
    getAll: vi.fn(),
    getByQR: vi.fn(),
  },
}));

vi.mock('../api/squads', () => ({
  squadsAPI: {
    getAll: vi.fn(),
  },
}));

vi.mock('../context/AuthContext', () => ({
  useAuth: () => ({
    user: { username: 'tester', role: authState.role },
    isAdmin: () => authState.role === 'admin',
    isResponsible: () => authState.role === 'responsible' || authState.role === 'admin',
  }),
}));

vi.mock('../context/CartContext', () => ({
  useCart: () => ({
    addToCart: cartState.addToCart,
    isInCart: (id) => cartState.inCartIds.has(id),
  }),
}));

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom');
  return {
    ...actual,
    useNavigate: () => navigateMock,
  };
});

const equipmentItems = [
  {
    id: 'eq-1',
    name: '3D Printer Alpha',
    description: 'Printer for prototyping',
    quantity: 4,
    available_quantity: 2,
    location: 'Lab A',
    squad_id: 'squad-1',
    squad_name: 'RoboLab',
    qr_code: 'QR-1',
  },
  {
    id: 'eq-2',
    name: 'Laser Cutter',
    description: 'Cutting station',
    quantity: 1,
    available_quantity: 0,
    location: 'Lab B',
    squad_id: 'squad-2',
    squad_name: 'DesignLab',
    qr_code: 'QR-2',
  },
];

const squads = [
  { id: 'squad-1', name: 'RoboLab' },
  { id: 'squad-2', name: 'DesignLab' },
];

const renderEquipmentList = () => render(
  <MemoryRouter>
    <EquipmentList />
  </MemoryRouter>
);

describe('EquipmentList', () => {
  beforeEach(() => {
    authState.role = 'admin';
    cartState.addToCart.mockReset();
    cartState.inCartIds = new Set();
    navigateMock.mockReset();
    equipmentAPI.getAll.mockReset();
    equipmentAPI.getByQR.mockReset();
    squadsAPI.getAll.mockReset();
    squadsAPI.getAll.mockResolvedValue(squads);
    equipmentAPI.getAll.mockResolvedValue(equipmentItems);
    equipmentAPI.getByQR.mockResolvedValue(equipmentItems[0]);
  });

  it('loads and renders equipment for admin users', async () => {
    renderEquipmentList();

    expect(await screen.findByText('3D Printer Alpha')).toBeInTheDocument();
    expect(screen.getByText('Laser Cutter')).toBeInTheDocument();
    expect(screen.getByText(/Lab A/)).toBeInTheDocument();
    expect(screen.getAllByText('RoboLab').length).toBeGreaterThanOrEqual(1);
    expect(equipmentAPI.getAll).toHaveBeenCalledWith({});
    expect(squadsAPI.getAll).toHaveBeenCalledTimes(1);
  });

  it('filters rendered equipment by search term', async () => {
    const user = userEvent.setup();
    renderEquipmentList();

    await screen.findByText('3D Printer Alpha');
    await user.type(screen.getByLabelText('Search equipment'), 'laser');

    expect(screen.queryByText('3D Printer Alpha')).not.toBeInTheDocument();
    expect(screen.getByText('Laser Cutter')).toBeInTheDocument();
  });

  it('reloads equipment with selected squad filter', async () => {
    const user = userEvent.setup();
    renderEquipmentList();

    await screen.findByText('3D Printer Alpha');
    await user.selectOptions(screen.getByLabelText('Squad filter'), 'squad-1');

    await waitFor(() => expect(equipmentAPI.getAll).toHaveBeenLastCalledWith({ squad_id: 'squad-1' }));
  });

  it('adds available equipment to booking cart', async () => {
    const user = userEvent.setup();
    renderEquipmentList();

    await screen.findByText('3D Printer Alpha');
    await user.click(screen.getByLabelText('Add 3D Printer Alpha to booking cart'));

    expect(cartState.addToCart).toHaveBeenCalledWith(equipmentItems[0], 1);
    expect(screen.queryByLabelText('Add Laser Cutter to booking cart')).not.toBeInTheDocument();
  });

  it('shows already-in-cart state without adding a duplicate marker manually', async () => {
    cartState.inCartIds = new Set(['eq-1']);

    renderEquipmentList();

    await screen.findByText('3D Printer Alpha');
    expect(screen.getByLabelText('Add 3D Printer Alpha to booking cart')).toHaveClass('btn-secondary');
  });

  it('shows QR/manual lookup mode for regular users without loading all equipment', async () => {
    const user = userEvent.setup();
    authState.role = 'user';

    renderEquipmentList();

    expect(equipmentAPI.getAll).not.toHaveBeenCalled();
    expect(squadsAPI.getAll).not.toHaveBeenCalled();

    await user.click(screen.getByText(/код вручную/i));
    await user.type(screen.getByLabelText('Manual equipment code'), 'QR-1');
    await user.click(screen.getByText(/найти/i));

    await waitFor(() => expect(equipmentAPI.getByQR).toHaveBeenCalledWith('QR-1'));
    expect(navigateMock).toHaveBeenCalledWith('/equipment/eq-1');
  });
});
