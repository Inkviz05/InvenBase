import apiClient from './client';

export const authAPI = {
  login: async (username, password) => {
    try {
      console.log('authAPI.login: Sending request to /auth/login with:', { username, password: '***' });
      const response = await apiClient.post('/auth/login', { username, password });
      console.log('authAPI.login: Response received:', response.data);
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
      console.log('authAPI.getCurrentUser: Sending request to /users/me');
      const response = await apiClient.get('/users/me');
      console.log('authAPI.getCurrentUser: Response received:', response.data);
      return response.data;
    } catch (error) {
      console.error('authAPI.getCurrentUser: Error occurred:', error);
      throw error;
    }
  },
};

