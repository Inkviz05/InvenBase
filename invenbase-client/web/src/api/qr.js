import apiClient from './client';

export const qrAPI = {
  generate: async (id) => {
    const response = await apiClient.get(`/qr/${id}`, {
      responseType: 'blob',
    });
    return response.data;
  },

  getData: async (id) => {
    const response = await apiClient.get(`/qr/${id}/data`);
    return response.data;
  },
};
