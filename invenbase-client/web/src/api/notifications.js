import apiClient from './client';

export const notificationsAPI = {
  getAll: async () => {
    const response = await apiClient.get('/notifications');
    return response.data;
  },

  getUnreadCount: async () => {
    const response = await apiClient.get('/notifications/count');
    return response.data;
  },

  markAsRead: async (id) => {
    await apiClient.post(`/notifications/${id}/read`);
  },

  markAllAsRead: async () => {
    await apiClient.post('/notifications/read-all');
  },
};

