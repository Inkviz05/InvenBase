import apiClient from './client';

export const instanceAPI = {
  createSetupCode: async (expiresInMinutes) => {
    const response = await apiClient.post('/instance/setup-codes', {
      expires_in_minutes: expiresInMinutes,
    });
    return response.data;
  },

  revokeSetupCode: async (id) => {
    await apiClient.delete(`/instance/setup-codes/${id}`);
  },
};
