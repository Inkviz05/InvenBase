import { expect, test } from '@playwright/test';

const adminUsername = process.env.REAL_SMOKE_ADMIN_USERNAME || 'admin';
const adminPassword = process.env.REAL_SMOKE_ADMIN_PASSWORD || 'smoke-admin-password';

const authHeaders = (token) => ({
  Authorization: `Bearer ${token}`,
});

const uniqueSuffix = () => `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`;

const toDatetimeLocal = (date) => {
  const pad = (value) => String(value).padStart(2, '0');

  return [
    date.getFullYear(),
    pad(date.getMonth() + 1),
    pad(date.getDate()),
  ].join('-') + `T${pad(date.getHours())}:${pad(date.getMinutes())}`;
};

const expectOk = async (response, label) => {
  expect(response.ok(), `${label}: ${response.status()} ${await response.text()}`).toBe(true);
};

test('real backend smoke: login, browse seeded equipment, create booking', async ({ page, request }) => {
  const suffix = uniqueSuffix();
  const equipmentName = `Real Smoke 3D Printer ${suffix}`;
  const purpose = `Real smoke booking ${suffix}`;

  const loginResponse = await request.post('/api/auth/login', {
    data: {
      username: adminUsername,
      password: adminPassword,
    },
  });
  await expectOk(loginResponse, 'api login');
  const { token } = await loginResponse.json();

  const squadResponse = await request.post('/api/squads', {
    headers: authHeaders(token),
    data: {
      name: `Real Smoke Squad ${suffix}`,
      description: 'Created by Playwright real smoke test',
      location: 'Smoke Lab',
      responsible_user_id: null,
    },
  });
  await expectOk(squadResponse, 'create squad');
  const squad = await squadResponse.json();

  const equipmentResponse = await request.post('/api/equipment', {
    headers: authHeaders(token),
    data: {
      name: equipmentName,
      description: 'Created by Playwright real smoke test',
      category_id: null,
      squad_id: squad.id,
      quantity: 2,
      location: 'Smoke Rack',
      responsible_user_id: null,
      is_unique: false,
    },
  });
  await expectOk(equipmentResponse, 'create equipment');
  const equipment = await equipmentResponse.json();

  const start = new Date(Date.now() + 24 * 60 * 60 * 1000);
  start.setMinutes(0, 0, 0);
  const end = new Date(start.getTime() + 2 * 60 * 60 * 1000);

  await page.goto('/#/login');
  await page.getByLabel('Username').fill(adminUsername);
  await page.getByLabel('Password').fill(adminPassword);
  await page.locator('form button[type="submit"]').click();

  await expect(page).toHaveURL(/#\/$/);
  await expect.poll(async () => page.evaluate(() => localStorage.getItem('token'))).not.toBeNull();

  await page.locator('a[href="#/equipment"]').first().click();
  await expect(page).toHaveURL(/#\/equipment$/);
  await page.getByLabel('Search equipment').fill(equipmentName);
  await expect(page.getByRole('heading', { name: equipmentName })).toBeVisible();
  await expect(page.getByText('Smoke Rack')).toBeVisible();

  await page.goto('/#/bookings/create');
  await expect(page).toHaveURL(/#\/bookings\/create$/);
  await page.getByLabel('Equipment').selectOption(equipment.id);
  await page.getByLabel('Quantity').fill('1');
  await page.getByLabel('Start date').fill(toDatetimeLocal(start));
  await page.getByLabel('End date').fill(toDatetimeLocal(end));
  await page.getByLabel('Purpose').fill(purpose);
  await page.locator('form button[type="submit"]').click();

  await expect(page).toHaveURL(/#\/bookings$/);
  await expect(page.getByRole('heading', { name: equipmentName })).toBeVisible();
  await expect(page.getByText(purpose)).toBeVisible();

  const bookingsResponse = await request.get('/api/bookings', {
    headers: authHeaders(token),
  });
  await expectOk(bookingsResponse, 'list bookings');
  const bookings = await bookingsResponse.json();
  expect(bookings.some((booking) => booking.purpose === purpose && booking.equipment_id === equipment.id)).toBe(true);
});
