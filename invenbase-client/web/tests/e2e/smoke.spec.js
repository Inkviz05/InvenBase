import { expect, test } from '@playwright/test';

const user = {
  id: 'user-1',
  username: 'admin',
  full_name: 'Admin User',
  email: 'admin@example.test',
  role: 'admin',
};

const equipment = [
  {
    id: 'eq-1',
    name: '3D Printer Alpha',
    description: 'Printer for prototypes',
    quantity: 4,
    available_quantity: 2,
    location: 'Lab A',
    squad_id: 'squad-1',
    squad_name: 'RoboLab',
    qr_code: 'QR-1',
    status: 'available',
  },
];

const squads = [
  { id: 'squad-1', name: 'RoboLab' },
];

const createdBooking = {
  id: 'booking-1',
  equipment_id: 'eq-1',
  equipment_name: '3D Printer Alpha',
  username: 'admin',
  status: 'pending',
  quantity: 1,
  purpose: 'Smoke test booking',
  start_date: '2026-08-10T10:00:00.000Z',
  end_date: '2026-08-10T12:00:00.000Z',
};

test.beforeEach(async ({ page }) => {
  let bookings = [];

  await page.route('**/api/auth/login', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ token: 'smoke-token', user }),
    });
  });

  await page.route('**/api/users/me', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(user),
    });
  });

  await page.route('**/api/notifications/count', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({ count: 0 }),
    });
  });

  await page.route('**/api/reports/equipment', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total_equipment: 1,
        available_equipment: 2,
        booked_equipment: 0,
        by_category: [],
      }),
    });
  });

  await page.route('**/api/reports/bookings', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify({
        total: 0,
        pending: 0,
        approved: 0,
        rejected: 0,
        expired: 0,
        awaiting_return: 0,
        returned: 0,
        cancelled: 0,
        completed: 0,
      }),
    });
  });

  await page.route('**/api/squads', async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(squads),
    });
  });

  await page.route(/\/api\/equipment(\?.*)?$/, async (route) => {
    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(equipment),
    });
  });

  await page.route('**/api/bookings', async (route) => {
    const request = route.request();

    if (request.method() === 'POST') {
      const payload = request.postDataJSON();
      bookings = [{
        ...createdBooking,
        quantity: payload.quantity,
        purpose: payload.purpose,
        start_date: payload.start_date,
        end_date: payload.end_date,
      }];

      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify(bookings[0]),
      });
      return;
    }

    await route.fulfill({
      status: 200,
      contentType: 'application/json',
      body: JSON.stringify(bookings),
    });
  });
});

test('login, browse equipment, create booking, and see it in booking list', async ({ page }) => {
  await page.goto('/#/login');

  await page.getByLabel('Username').fill('admin');
  await page.getByLabel('Password').fill('admin-password');
  await page.locator('form button[type="submit"]').click();

  await expect(page).toHaveURL(/#\/$/);
  await expect.poll(async () => page.evaluate(() => localStorage.getItem('token'))).toBe('smoke-token');

  await page.locator('a[href="#/equipment"]').first().click();
  await expect(page).toHaveURL(/#\/equipment$/);
  await expect(page.getByRole('heading', { name: '3D Printer Alpha' })).toBeVisible();
  await expect(page.getByText('Printer for prototypes')).toBeVisible();
  await expect(page.getByText('Lab A')).toBeVisible();

  await page.evaluate(() => {
    window.location.hash = '#/bookings/create';
    window.dispatchEvent(new HashChangeEvent('hashchange'));
  });
  await expect(page).toHaveURL(/#\/bookings\/create$/);
  await page.getByLabel('Equipment').selectOption('eq-1');
  await page.getByLabel('Quantity').fill('1');
  await page.getByLabel('Start date').fill('2026-08-10T10:00');
  await page.getByLabel('End date').fill('2026-08-10T12:00');
  await page.getByLabel('Purpose').fill('Smoke test booking');
  await page.locator('form button[type="submit"]').click();

  await expect(page).toHaveURL(/#\/bookings$/);
  await expect(page.getByRole('heading', { name: '3D Printer Alpha' })).toBeVisible();
  await expect(page.getByText('Smoke test booking')).toBeVisible();
  await expect(page.getByLabel('Approve booking booking-1')).toBeVisible();
});
