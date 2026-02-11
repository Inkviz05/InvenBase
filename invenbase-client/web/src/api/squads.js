import apiClient from './client';

export const squadsAPI = {
  getAll: async () => {
    const response = await apiClient.get('/squads');
    return response.data;
  },

  getById: async (id) => {
    const response = await apiClient.get(`/squads/${id}`);
    return response.data;
  },

  create: async (data) => {
    const response = await apiClient.post('/squads', data);
    return response.data;
  },

  update: async (id, data) => {
    const response = await apiClient.put(`/squads/${id}`, data);
    return response.data;
  },

  delete: async (id) => {
    await apiClient.delete(`/squads/${id}`);
  },

  getEquipment: async (squadId) => {
    const response = await apiClient.get(`/squads/${squadId}/equipment`);
    return response.data;
  },
};
