const API_PORT = parseInt(import.meta.env.VITE_API_PORT, 10) || 8080;

const getDefaultApiUrl = () => {
  if (typeof window !== 'undefined' && window.location?.hostname) {
    return `http://${window.location.hostname}:${API_PORT}/api`;
  }

  return `http://localhost:${API_PORT}/api`;
};

export const getApiBaseUrl = () => {
  if (import.meta.env.DEV) {
    return '/api';
  }

  return import.meta.env.VITE_API_URL || getDefaultApiUrl();
};

export const API_BASE_URL = getApiBaseUrl();

export const ROLES = {
  ADMIN: 'admin',
  RESPONSIBLE: 'responsible',
  USER: 'user'
};

export const BOOKING_STATUS = {
  PENDING: 'pending',
  APPROVED: 'approved',
  REJECTED: 'rejected',
  CANCELLED: 'cancelled'
};

export const PERMISSION_TYPE = {
  INTERNAL: 'internal',
  EXTERNAL: 'external'
};
