import apiClient from './client';

export const equipmentAPI = {
  getAll: async (params = {}) => {
    const response = await apiClient.get('/equipment', { params });
    return response.data;
  },

  getById: async (id) => {
    const response = await apiClient.get(`/equipment/${id}`);
    return response.data;
  },

  getByQR: async (qrCode) => {
    const response = await apiClient.get(`/equipment/qr/${qrCode}`);
    return response.data;
  },

  create: async (data) => {
    const response = await apiClient.post('/equipment', data);
    return response.data;
  },

  update: async (id, data) => {
    const response = await apiClient.put(`/equipment/${id}`, data);
    return response.data;
  },

  delete: async (id) => {
    await apiClient.delete(`/equipment/${id}`);
  },

  getMovements: async (id) => {
    const response = await apiClient.get(`/equipment/${id}/movements`);
    return response.data;
  },

  move: async (id, data) => {
    const response = await apiClient.post(`/equipment/${id}/move`, data);
    return response.data;
  },
};

