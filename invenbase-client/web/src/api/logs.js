import apiClient from './client';

export const logsAPI = {
  getAll: async (params = {}) => {
    const response = await apiClient.get('/logs', { params });
    return response.data;
  },
};
