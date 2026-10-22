import apiClient from './client';

export const authAPI = {
  login: async (username, password) => {
    try {
      const response = await apiClient.post('/auth/login', { username, password });
      return response.data;
    } catch (error) {
      console.error('authAPI.login: Error occurred:', error);
      console.error('authAPI.login: Error details:', {
        message: error.message,
        status: error.response?.status,
        statusText: error.response?.statusText,
        data: error.response?.data,
        headers: error.response?.headers,
        request: error.request
      });
      throw error;
    }
  },

  getCurrentUser: async () => {
    try {
      const response = await apiClient.get('/users/me');
      return response.data;
    } catch (error) {
      console.error('authAPI.getCurrentUser: Error occurred:', error);
      throw error;
    }
  },
};

