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

  /** Единый журнал учёта: все действия с фильтрами. Параметры: from, to (YYYY-MM-DD), action, entity_type, user_id, limit, offset */
  getAuditReport: async (params = {}) => {
    const response = await apiClient.get('/reports/audit', { params });
    return response.data;
  },
};
