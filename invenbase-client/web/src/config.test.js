import { describe, expect, it } from 'vitest';
import { BOOKING_STATUS, getApiBaseUrl, PERMISSION_TYPE, ROLES } from './config';

describe('config', () => {
  it('uses the Vite proxy API URL in dev and test mode', () => {
    expect(getApiBaseUrl()).toBe('/api');
  });

  it('exports API domain constants used by the app', () => {
    expect(ROLES).toEqual({
      ADMIN: 'admin',
      RESPONSIBLE: 'responsible',
      USER: 'user',
    });

    expect(BOOKING_STATUS).toMatchObject({
      PENDING: 'pending',
      APPROVED: 'approved',
      REJECTED: 'rejected',
      CANCELLED: 'cancelled',
    });

    expect(PERMISSION_TYPE).toEqual({
      INTERNAL: 'internal',
      EXTERNAL: 'external',
    });
  });
});
