import apiClient from './client';

export const bookingsAPI = {
  getAll: async () => {
    const response = await apiClient.get('/bookings');
    return response.data;
  },

  getById: async (id) => {
    const response = await apiClient.get(`/bookings/${id}`);
    return response.data;
  },

  create: async (data) => {
    const response = await apiClient.post('/bookings', data);
    return response.data;
  },

  update: async (id, data) => {
    const response = await apiClient.put(`/bookings/${id}`, data);
    return response.data;
  },

  approve: async (id) => {
    const response = await apiClient.post(`/bookings/${id}/approve`);
    return response.data;
  },

  reject: async (id) => {
    const response = await apiClient.post(`/bookings/${id}/reject`);
    return response.data;
  },

  confirmReturn: async (id) => {
    const response = await apiClient.post(`/bookings/${id}/return`);
    return response.data;
  },

  cancel: async (id) => {
    const response = await apiClient.post(`/bookings/${id}/cancel`);
    return response.data;
  },

  delete: async (id) => {
    await apiClient.post(`/bookings/${id}/cancel`);
  },
};

