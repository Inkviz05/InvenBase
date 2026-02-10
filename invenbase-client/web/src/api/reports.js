import apiClient from './client';

export const reportsAPI = {
  getEquipmentReport: async () => {
    const response = await apiClient.get('/reports/equipment');
    return response.data;
  },

  getBookingReport: async () => {
    const response = await apiClient.get('/reports/bookings');
    return response.data;
  },

  getBookingDetailedReport: async (from, to) => {
    const params = {};
    if (from) params.from = from;
    if (to) params.to = to;
    const response = await apiClient.get('/reports/bookings/detailed', { params });
    return response.data;
  },
};
