import apiClient from './client';

export const supportAPI = {
  getRequests: async () => {
    const response = await apiClient.get('/support/requests');
    return response.data;
  },

  createRequest: async (data) => {
    const response = await apiClient.post('/support/requests', data);
    return response.data;
  },

  updateRequest: async (id, data) => {
    const response = await apiClient.put(`/support/requests/${id}`, data);
    return response.data;
  },

  addMessage: async (requestId, data) => {
    const response = await apiClient.post(`/support/requests/${requestId}/messages`, data);
    return response.data;
  },

  deleteRequest: async (id) => {
    await apiClient.delete(`/support/requests/${id}`);
  },
};
