const API_BASE = '/api';

let authToken = localStorage.getItem('authToken');
let currentUser = null;

// Инициализация
document.addEventListener('DOMContentLoaded', async () => {
    if (!authToken) {
        window.location.href = '/';
        return;
    }

    await loadUserInfo();
    setupTabs();
    setupRoleBasedUI();
    loadDashboard();
    
    // Обработчик выхода
    document.getElementById('logoutBtn').addEventListener('click', () => {
        localStorage.removeItem('authToken');
        window.location.href = '/';
    });
});

// Загрузка информации о пользователе
async function loadUserInfo() {
    try {
        const response = await fetch(`${API_BASE}/users/me`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) {
            throw new Error('Unauthorized');
        }
        
        currentUser = await response.json();
        document.getElementById('username').textContent = `${currentUser.username} (${getRoleText(currentUser.role)})`;
    } catch (error) {
        localStorage.removeItem('authToken');
        window.location.href = '/';
    }
}

// Настройка UI в зависимости от роли
function setupRoleBasedUI() {
    const isAdmin = currentUser.role === 'admin';
    const isResponsible = currentUser.role === 'responsible' || isAdmin;
    const isUser = currentUser.role === 'user';

    // Скрываем/показываем вкладки
    document.querySelectorAll('.tab').forEach(tab => {
        const tabName = tab.dataset.tab;
        
        if (tabName === 'users' && !isAdmin) {
            tab.style.display = 'none';
        }
        if (tabName === 'logs' && !isResponsible) {
            tab.style.display = 'none';
        }
        if (tabName === 'reports' && !isResponsible) {
            tab.style.display = 'none';
        }
    });

    // Скрываем кнопки добавления
    if (!isResponsible) {
        document.querySelectorAll('.btn-primary').forEach(btn => {
            if (btn.textContent.includes('Добавить')) {
                btn.style.display = 'none';
            }
        });
    }
}

// Получение текста роли
function getRoleText(role) {
    const roles = {
        'admin': 'Администратор',
        'responsible': 'Ответственный',
        'user': 'Пользователь'
    };
    return roles[role] || role;
}

// Настройка табов
function setupTabs() {
    document.querySelectorAll('.tab').forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.dataset.tab;
            
            document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
            document.querySelectorAll('.tab-content').forEach(c => c.classList.remove('active'));
            
            tab.classList.add('active');
            document.getElementById(tabName).classList.add('active');
            
            // Загружаем данные при переключении таба
            switch(tabName) {
                case 'equipment':
                    loadEquipment();
                    break;
                case 'categories':
                    loadCategories();
                    break;
                case 'bookings':
                    loadBookings();
                    break;
                case 'users':
                    if (currentUser.role === 'admin') {
                        loadUsers();
                    }
                    break;
                case 'logs':
                    if (currentUser.role === 'admin' || currentUser.role === 'responsible') {
                        loadLogs();
                    }
                    break;
                case 'reports':
                    if (currentUser.role === 'admin' || currentUser.role === 'responsible') {
                        loadReports();
                    }
                    break;
                case 'dashboard':
                    loadDashboard();
                    break;
            }
        });
    });
}

// Загрузка дашборда
async function loadDashboard() {
    try {
        if (currentUser.role !== 'admin' && currentUser.role !== 'responsible') {
            // Для обычных пользователей показываем упрощённую версию
            const bookingsResponse = await fetch(`${API_BASE}/bookings`, {
                headers: { 'Authorization': `Bearer ${authToken}` }
            });
            
            if (bookingsResponse.ok) {
                const bookings = await bookingsResponse.json();
                const myBookings = bookings.filter(b => b.user_id === currentUser.id);
                const pending = myBookings.filter(b => b.status === 'pending').length;
                const approved = myBookings.filter(b => b.status === 'approved').length;
                
                document.getElementById('stat-total').textContent = myBookings.length;
                document.getElementById('stat-available').textContent = approved;
                document.getElementById('stat-booked').textContent = pending;
                document.getElementById('stat-pending').textContent = '-';
            }
            return;
        }

        const response = await fetch(`${API_BASE}/reports/equipment`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (response.ok) {
            const data = await response.json();
            document.getElementById('stat-total').textContent = data.total_equipment || 0;
            document.getElementById('stat-available').textContent = data.available_equipment || 0;
            document.getElementById('stat-booked').textContent = data.booked_equipment || 0;
            
            // Добавляем статистику по категориям
            if (data.by_category && data.by_category.length > 0) {
                const categoryStats = data.by_category.map(cat => `
                    <div style="margin: 12px 0; padding: 18px; background: linear-gradient(135deg, #f8f9fa 0%, #ffffff 100%); border-radius: 10px; border-left: 4px solid #1976d2; box-shadow: 0 2px 8px rgba(0,0,0,0.05);">
                        <div style="font-weight: 600; font-size: 16px; color: #333; margin-bottom: 8px;">
                            ${cat.category_name || 'Без категории'}
                        </div>
                        <div style="display: grid; grid-template-columns: repeat(3, 1fr); gap: 15px; margin-top: 10px;">
                            <div><strong>Всего:</strong> <span style="color: #1976d2; font-size: 18px;">${cat.total || 0}</span></div>
                            <div><strong>Доступно:</strong> <span style="color: #388e3c; font-size: 18px;">${cat.available || 0}</span></div>
                            <div><strong>Забронировано:</strong> <span style="color: #f57c00; font-size: 18px;">${cat.booked || 0}</span></div>
                        </div>
                    </div>
                `).join('');
                
                const dashboardContent = document.getElementById('dashboard');
                let categorySection = dashboardContent.querySelector('#category-stats');
                if (!categorySection) {
                    categorySection = document.createElement('div');
                    categorySection.id = 'category-stats';
                    categorySection.style.cssText = 'margin-top: 40px;';
                    dashboardContent.appendChild(categorySection);
                }
                categorySection.innerHTML = '<h3 style="margin-bottom: 20px; color: #333; font-size: 22px;">📊 Статистика по категориям</h3>' + categoryStats;
            }
        }

        const bookingsResponse = await fetch(`${API_BASE}/reports/bookings`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (bookingsResponse.ok) {
            const bookingsData = await bookingsResponse.json();
            document.getElementById('stat-pending').textContent = bookingsData.pending || 0;
            
            // Добавляем дополнительную информацию о бронированиях
            const dashboardContent = document.getElementById('dashboard');
            let bookingInfo = dashboardContent.querySelector('#booking-info');
            if (!bookingInfo) {
                bookingInfo = document.createElement('div');
                bookingInfo.id = 'booking-info';
                bookingInfo.style.cssText = 'margin-top: 30px; padding: 20px; background: #f9f9f9; border-radius: 8px;';
                dashboardContent.appendChild(bookingInfo);
            }
            bookingInfo.innerHTML = `
                <h3 style="margin-bottom: 20px; color: #333; font-size: 22px;">📅 Статистика бронирований</h3>
                <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 20px; margin-top: 15px;">
                    <div style="padding: 20px; background: linear-gradient(135deg, #fff3cd 0%, #ffffff 100%); border-radius: 10px; border-left: 4px solid #f57c00; box-shadow: 0 2px 8px rgba(0,0,0,0.05);">
                        <div style="font-size: 14px; color: #666; margin-bottom: 8px;">Ожидают одобрения</div>
                        <div style="font-size: 32px; font-weight: 700; color: #f57c00;">${bookingsData.pending || 0}</div>
                    </div>
                    <div style="padding: 20px; background: linear-gradient(135deg, #d4edda 0%, #ffffff 100%); border-radius: 10px; border-left: 4px solid #388e3c; box-shadow: 0 2px 8px rgba(0,0,0,0.05);">
                        <div style="font-size: 14px; color: #666; margin-bottom: 8px;">Одобрены</div>
                        <div style="font-size: 32px; font-weight: 700; color: #388e3c;">${bookingsData.approved || 0}</div>
                    </div>
                    <div style="padding: 20px; background: linear-gradient(135deg, #f8d7da 0%, #ffffff 100%); border-radius: 10px; border-left: 4px solid #d32f2f; box-shadow: 0 2px 8px rgba(0,0,0,0.05);">
                        <div style="font-size: 14px; color: #666; margin-bottom: 8px;">Истекли</div>
                        <div style="font-size: 32px; font-weight: 700; color: #d32f2f;">${bookingsData.expired || 0}</div>
                    </div>
                </div>
            `;
        }
    } catch (error) {
        console.error('Ошибка загрузки дашборда:', error);
        showError('Не удалось загрузить данные дашборда');
    }
}

// Загрузка оборудования
async function loadEquipment() {
    try {
        const response = await fetch(`${API_BASE}/equipment`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) {
            throw new Error('Failed to load equipment');
        }
        
        const equipment = await response.json();
        const tbody = document.getElementById('equipmentTableBody');
        const isResponsible = currentUser.role === 'responsible' || currentUser.role === 'admin';
        
        tbody.innerHTML = equipment.map(eq => `
            <tr>
                <td>${eq.name}</td>
                <td>${eq.category_name || '-'}</td>
                <td>${eq.quantity}</td>
                <td>${eq.available_quantity}</td>
                <td>${eq.location || '-'}</td>
                <td><span class="status-badge status-${eq.status}">${getEquipmentStatusText(eq.status)}</span></td>
                <td>
                    ${isResponsible ? `
                        <div class="action-buttons">
                            <button class="btn btn-sm btn-primary" onclick="editEquipment('${eq.id}')">Редактировать</button>
                            ${currentUser.role === 'admin' ? `<button class="btn btn-sm btn-danger" onclick="deleteEquipment('${eq.id}')">Удалить</button>` : ''}
                            <button class="btn btn-sm btn-secondary" onclick="generateQR('${eq.id}')">QR-код</button>
                        </div>
                    ` : '<span style="color: #999;">-</span>'}
                </td>
            </tr>
        `).join('');
    } catch (error) {
        console.error('Ошибка загрузки оборудования:', error);
        showError('Не удалось загрузить оборудование');
    }
}

// Загрузка категорий
async function loadCategories() {
    try {
        const response = await fetch(`${API_BASE}/categories`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        const categories = await response.json();
        const tbody = document.getElementById('categoriesTableBody');
        const isResponsible = currentUser.role === 'responsible' || currentUser.role === 'admin';
        
        tbody.innerHTML = categories.map(cat => `
            <tr>
                <td>${cat.name}</td>
                <td>${cat.description || '-'}</td>
                <td>
                    ${isResponsible ? `
                        <div class="action-buttons">
                            <button class="btn btn-sm btn-primary" onclick="editCategory('${cat.id}')">Редактировать</button>
                            ${currentUser.role === 'admin' ? `<button class="btn btn-sm btn-danger" onclick="deleteCategory('${cat.id}')">Удалить</button>` : ''}
                        </div>
                    ` : '<span style="color: #999;">-</span>'}
                </td>
            </tr>
        `).join('');
    } catch (error) {
        console.error('Ошибка загрузки категорий:', error);
        showError('Не удалось загрузить категории');
    }
}

// Загрузка бронирований
async function loadBookings() {
    try {
        const filter = document.getElementById('bookingFilter')?.value || '';
        const url = filter ? `${API_BASE}/bookings?status=${filter}` : `${API_BASE}/bookings`;
        
        const response = await fetch(url, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) {
            throw new Error('Failed to load bookings');
        }
        
        const bookings = await response.json();
        const tbody = document.getElementById('bookingsTableBody');
        const isResponsible = currentUser.role === 'responsible' || currentUser.role === 'admin';
        const currentUserId = currentUser.id;
        
        tbody.innerHTML = bookings.map(booking => {
            const canApprove = isResponsible && booking.status === 'pending';
            const canReject = isResponsible && booking.status === 'pending';
            const canCancel = booking.user_id === currentUserId && booking.status === 'pending';
            const canDelete = booking.user_id === currentUserId || isResponsible;
            
            return `
                <tr>
                    <td>${booking.username || booking.user_name || '-'}</td>
                    <td>${booking.equipment_name || '-'}</td>
                    <td>${booking.quantity}</td>
                    <td>${new Date(booking.start_date).toLocaleDateString('ru-RU')}</td>
                    <td>${new Date(booking.end_date).toLocaleDateString('ru-RU')}</td>
                    <td><span class="status-badge status-${booking.status}">${getBookingStatusText(booking.status)}</span></td>
                    <td>
                        <div class="action-buttons">
                            ${canApprove ? `<button class="btn btn-sm btn-success" onclick="approveBooking('${booking.id}')">Одобрить</button>` : ''}
                            ${canReject ? `<button class="btn btn-sm btn-warning" onclick="rejectBooking('${booking.id}')">Отклонить</button>` : ''}
                            ${canCancel ? `<button class="btn btn-sm btn-secondary" onclick="cancelBooking('${booking.id}')">Отменить</button>` : ''}
                            ${canDelete && booking.status !== 'approved' ? `<button class="btn btn-sm btn-danger" onclick="deleteBooking('${booking.id}')">Удалить</button>` : ''}
                        </div>
                    </td>
                </tr>
            `;
        }).join('');
    } catch (error) {
        console.error('Ошибка загрузки бронирований:', error);
        showError('Не удалось загрузить бронирования');
    }
}

function getBookingStatusText(status) {
    const statuses = {
        'pending': 'Ожидает',
        'approved': 'Одобрено',
        'rejected': 'Отклонено',
        'cancelled': 'Отменено'
    };
    return statuses[status] || status;
}

function getEquipmentStatusText(status) {
    const statuses = {
        'available': 'Доступно',
        'maintenance': 'На обслуживании',
        'unavailable': 'Недоступно'
    };
    return statuses[status] || status;
}

// Одобрение бронирования
async function approveBooking(id) {
    if (!confirm('Одобрить это бронирование?')) return;
    
    try {
        const response = await fetch(`${API_BASE}/bookings/${id}/approve`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (response.ok) {
            showSuccess('Бронирование одобрено');
            loadBookings();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка одобрения бронирования');
        }
    } catch (error) {
        showError('Ошибка при одобрении бронирования');
    }
}

// Отклонение бронирования
async function rejectBooking(id) {
    if (!confirm('Отклонить это бронирование?')) return;
    
    try {
        const response = await fetch(`${API_BASE}/bookings/${id}/reject`, {
            method: 'POST',
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (response.ok) {
            showSuccess('Бронирование отклонено');
            loadBookings();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка отклонения бронирования');
        }
    } catch (error) {
        showError('Ошибка при отклонении бронирования');
    }
}

// Отмена бронирования
async function cancelBooking(id) {
    if (!confirm('Отменить это бронирование?')) return;
    
    try {
        const response = await fetch(`${API_BASE}/bookings/${id}`, {
            method: 'PUT',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify({ status: 'cancelled' })
        });
        
        if (response.ok) {
            showSuccess('Бронирование отменено');
            loadBookings();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка отмены бронирования');
        }
    } catch (error) {
        showError('Ошибка при отмене бронирования');
    }
}

// Загрузка пользователей (только для админов)
async function loadUsers() {
    if (currentUser.role !== 'admin') {
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/users`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) {
            throw new Error('Failed to load users');
        }
        
        const users = await response.json();
        const tbody = document.getElementById('usersTableBody');
        
        tbody.innerHTML = users.map(user => `
            <tr>
                <td>${user.username}</td>
                <td>${user.full_name || '-'}</td>
                <td>${user.email || '-'}</td>
                <td><span class="role-badge role-${user.role}">${getRoleText(user.role)}</span></td>
                <td>
                    <div class="action-buttons">
                        <button class="btn btn-sm btn-primary" onclick="editUser('${user.id}')">Редактировать</button>
                        ${user.id !== currentUser.id ? `<button class="btn btn-sm btn-danger" onclick="deleteUser('${user.id}')">Удалить</button>` : '<span style="color: #999; font-size: 12px; padding: 8px 16px;">Вы</span>'}
                    </div>
                </td>
            </tr>
        `).join('');
    } catch (error) {
        console.error('Ошибка загрузки пользователей:', error);
        showError('Не удалось загрузить пользователей');
    }
}

// Загрузка логов (только для админов и ответственных)
async function loadLogs() {
    if (currentUser.role !== 'admin' && currentUser.role !== 'responsible') {
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/logs?limit=100`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) {
            throw new Error('Failed to load logs');
        }
        
        const logs = await response.json();
        const tbody = document.getElementById('logsTableBody');
        
        tbody.innerHTML = logs.map(log => `
            <tr>
                <td>${new Date(log.created_at).toLocaleString('ru-RU')}</td>
                <td>${log.action}</td>
                <td>${log.entity_type}</td>
                <td>${log.details ? JSON.stringify(log.details) : '-'}</td>
            </tr>
        `).join('');
    } catch (error) {
        console.error('Ошибка загрузки логов:', error);
        showError('Не удалось загрузить логи');
    }
}

// Загрузка отчётов
async function loadReports() {
    try {
        const equipmentResponse = await fetch(`${API_BASE}/reports/equipment`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        const bookingsResponse = await fetch(`${API_BASE}/reports/bookings`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        let reportHtml = '<h3>Отчёт по оборудованию</h3>';
        
        if (equipmentResponse.ok) {
            const equipmentData = await equipmentResponse.json();
            reportHtml += `
                <p>Всего оборудования: ${equipmentData.total_equipment || 0}</p>
                <p>Доступно: ${equipmentData.available_equipment || 0}</p>
                <p>Забронировано: ${equipmentData.booked_equipment || 0}</p>
            `;
        }
        
        reportHtml += '<h3>Отчёт по бронированиям</h3>';
        
        if (bookingsResponse.ok) {
            const bookingsData = await bookingsResponse.json();
            reportHtml += `
                <p>Ожидающие: ${bookingsData.pending || 0}</p>
                <p>Одобренные: ${bookingsData.approved || 0}</p>
                <p>Отклонённые: ${bookingsData.rejected || 0}</p>
            `;
        }
        
        document.getElementById('reportResult').innerHTML = reportHtml;
    } catch (error) {
        console.error('Ошибка загрузки отчётов:', error);
        showError('Не удалось загрузить отчёты');
    }
}

// Модальные окна
function showAddEquipmentModal() {
    if (currentUser.role !== 'admin' && currentUser.role !== 'responsible') {
        showError('Недостаточно прав');
        return;
    }
    
    document.getElementById('modalTitle').textContent = 'Добавить оборудование';
    document.getElementById('modalBody').innerHTML = `
        <form id="equipmentForm" onsubmit="saveEquipment(event)">
            <div class="form-group">
                <label>Название *</label>
                <input type="text" name="name" required>
            </div>
            <div class="form-group">
                <label>Описание</label>
                <textarea name="description"></textarea>
            </div>
            <div class="form-group">
                <label>Категория</label>
                <select name="category_id" id="categorySelect"></select>
            </div>
            <div class="form-group">
                <label>Количество *</label>
                <input type="number" name="quantity" required min="1">
            </div>
            <div class="form-group">
                <label>Местоположение</label>
                <input type="text" name="location">
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModals()">Отмена</button>
                <button type="submit" class="btn btn-primary">Сохранить</button>
            </div>
        </form>
    `;
    loadCategoriesForSelect();
    document.getElementById('modalOverlay').classList.add('show');
}

function showAddCategoryModal() {
    if (currentUser.role !== 'admin' && currentUser.role !== 'responsible') {
        showError('Недостаточно прав');
        return;
    }
    
    document.getElementById('modalTitle').textContent = 'Добавить категорию';
    document.getElementById('modalBody').innerHTML = `
        <form id="categoryForm" onsubmit="saveCategory(event)">
            <div class="form-group">
                <label>Название *</label>
                <input type="text" name="name" required>
            </div>
            <div class="form-group">
                <label>Описание</label>
                <textarea name="description"></textarea>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModals()">Отмена</button>
                <button type="submit" class="btn btn-primary">Сохранить</button>
            </div>
        </form>
    `;
    document.getElementById('modalOverlay').classList.add('show');
}

function showAddUserModal() {
    if (currentUser.role !== 'admin') {
        showError('Только администраторы могут создавать пользователей');
        return;
    }
    
    document.getElementById('modalTitle').textContent = 'Добавить пользователя';
    document.getElementById('modalBody').innerHTML = `
        <form id="userForm" onsubmit="saveUser(event)">
            <div class="form-group">
                <label>Имя пользователя *</label>
                <input type="text" name="username" required>
            </div>
            <div class="form-group">
                <label>Пароль *</label>
                <input type="password" name="password" required>
            </div>
            <div class="form-group">
                <label>ФИО</label>
                <input type="text" name="full_name">
            </div>
            <div class="form-group">
                <label>Email</label>
                <input type="email" name="email">
            </div>
            <div class="form-group">
                <label>Роль *</label>
                <select name="role" required>
                    <option value="user">Пользователь</option>
                    <option value="responsible">Ответственный</option>
                    <option value="admin">Администратор</option>
                </select>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModals()">Отмена</button>
                <button type="submit" class="btn btn-primary">Сохранить</button>
            </div>
        </form>
    `;
    document.getElementById('modalOverlay').classList.add('show');
}

function closeModals() {
    document.getElementById('modalOverlay').classList.remove('show');
}

// Сохранение данных
async function saveEquipment(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const data = Object.fromEntries(formData);
    
    if (data.category_id === '') delete data.category_id;
    data.quantity = parseInt(data.quantity);
    
    try {
        const response = await fetch(`${API_BASE}/equipment`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            showSuccess('Оборудование добавлено');
            closeModals();
            loadEquipment();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка добавления оборудования');
        }
    } catch (error) {
        showError('Ошибка при добавлении оборудования');
    }
}

async function saveCategory(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const data = Object.fromEntries(formData);
    
    try {
        const response = await fetch(`${API_BASE}/categories`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            showSuccess('Категория добавлена');
            closeModals();
            loadCategories();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка добавления категории');
        }
    } catch (error) {
        showError('Ошибка при добавлении категории');
    }
}

async function saveUser(event) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const data = Object.fromEntries(formData);
    
    try {
        const response = await fetch(`${API_BASE}/users`, {
            method: 'POST',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            showSuccess('Пользователь добавлен');
            closeModals();
            loadUsers();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка добавления пользователя');
        }
    } catch (error) {
        showError('Ошибка при добавлении пользователя');
    }
}

// Удаление
async function deleteEquipment(id) {
    if (currentUser.role !== 'admin') {
        showError('Только администраторы могут удалять оборудование');
        return;
    }
    
    // Загружаем информацию об оборудовании для подтверждения
    try {
        const equipmentResponse = await fetch(`${API_BASE}/equipment/${id}`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!equipmentResponse.ok) {
            throw new Error('Failed to load equipment');
        }
        
        const equipment = await equipmentResponse.json();
        
        const confirmed = confirm(
            `Вы уверены, что хотите удалить оборудование "${equipment.name}"?\n\n` +
            `Это действие нельзя отменить. Все связанные бронирования будут затронуты.`
        );
        
        if (!confirmed) return;
        
        const response = await fetch(`${API_BASE}/equipment/${id}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (response.ok) {
            showSuccess(`Оборудование "${equipment.name}" успешно удалено`);
            loadEquipment();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка удаления оборудования');
        }
    } catch (error) {
        console.error('Ошибка удаления оборудования:', error);
        showError('Ошибка при удалении оборудования');
    }
}

async function deleteCategory(id) {
    if (currentUser.role !== 'admin') {
        showError('Только администраторы могут удалять категории');
        return;
    }
    
    // Загружаем информацию о категории для подтверждения
    try {
        const categoryResponse = await fetch(`${API_BASE}/categories/${id}`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!categoryResponse.ok) {
            throw new Error('Failed to load category');
        }
        
        const category = await categoryResponse.json();
        
        const confirmed = confirm(
            `Вы уверены, что хотите удалить категорию "${category.name}"?\n\n` +
            `Внимание: Если в этой категории есть оборудование, удаление может быть невозможно.\n` +
            `Это действие нельзя отменить.`
        );
        
        if (!confirmed) return;
        
        const response = await fetch(`${API_BASE}/categories/${id}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (response.ok) {
            showSuccess(`Категория "${category.name}" успешно удалена`);
            loadCategories();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка удаления категории. Возможно, в категории есть оборудование.');
        }
    } catch (error) {
        console.error('Ошибка удаления категории:', error);
        showError('Ошибка при удалении категории');
    }
}

async function deleteUser(id) {
    if (currentUser.role !== 'admin') {
        showError('Только администраторы могут удалять пользователей');
        return;
    }
    
    if (id === currentUser.id) {
        showError('Вы не можете удалить свой собственный аккаунт');
        return;
    }
    
    // Загружаем информацию о пользователе для подтверждения
    try {
        const userResponse = await fetch(`${API_BASE}/users/${id}`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!userResponse.ok) {
            throw new Error('Failed to load user');
        }
        
        const user = await userResponse.json();
        
        const confirmed = confirm(
            `Вы уверены, что хотите удалить пользователя "${user.username}" (${user.full_name || user.email || 'без имени'})?\n\n` +
            `Роль: ${getRoleText(user.role)}\n` +
            `Это действие нельзя отменить. Все связанные данные пользователя будут затронуты.`
        );
        
        if (!confirmed) return;
        
        const response = await fetch(`${API_BASE}/users/${id}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (response.ok) {
            showSuccess(`Пользователь "${user.username}" успешно удалён`);
            loadUsers();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка удаления пользователя');
        }
    } catch (error) {
        console.error('Ошибка удаления пользователя:', error);
        showError('Ошибка при удалении пользователя');
    }
}

async function deleteBooking(id) {
    // Загружаем информацию о бронировании для подтверждения
    try {
        const bookingResponse = await fetch(`${API_BASE}/bookings/${id}`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!bookingResponse.ok) {
            throw new Error('Failed to load booking');
        }
        
        const booking = await bookingResponse.json();
        
        const statusText = getBookingStatusText(booking.status);
        const confirmed = confirm(
            `Вы уверены, что хотите удалить бронирование?\n\n` +
            `Оборудование: ${booking.equipment_name || 'Не указано'}\n` +
            `Количество: ${booking.quantity}\n` +
            `Статус: ${statusText}\n` +
            `Период: ${new Date(booking.start_date).toLocaleDateString('ru-RU')} - ${new Date(booking.end_date).toLocaleDateString('ru-RU')}\n\n` +
            `Это действие нельзя отменить.`
        );
        
        if (!confirmed) return;
        
        const response = await fetch(`${API_BASE}/bookings/${id}`, {
            method: 'DELETE',
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (response.ok) {
            showSuccess('Бронирование успешно удалено');
            loadBookings();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка удаления бронирования');
        }
    } catch (error) {
        console.error('Ошибка удаления бронирования:', error);
        showError('Ошибка при удалении бронирования');
    }
}

// Редактирование
async function editEquipment(id) {
    if (currentUser.role !== 'admin' && currentUser.role !== 'responsible') {
        showError('Недостаточно прав');
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/equipment/${id}`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) {
            throw new Error('Failed to load equipment');
        }
        
        const equipment = await response.json();
        
        document.getElementById('modalTitle').textContent = 'Редактировать оборудование';
        document.getElementById('modalBody').innerHTML = `
            <form id="equipmentForm" onsubmit="updateEquipment(event, '${id}')">
                <div class="form-group">
                    <label>Название *</label>
                    <input type="text" name="name" value="${equipment.name || ''}" required>
                </div>
                <div class="form-group">
                    <label>Описание</label>
                    <textarea name="description">${equipment.description || ''}</textarea>
                </div>
                <div class="form-group">
                    <label>Категория</label>
                    <select name="category_id" id="categorySelect">
                        <option value="">Выберите категорию</option>
                    </select>
                </div>
                <div class="form-group">
                    <label>Количество *</label>
                    <input type="number" name="quantity" value="${equipment.quantity || 1}" required min="1">
                </div>
                <div class="form-group">
                    <label>Доступно</label>
                    <input type="number" name="available_quantity" value="${equipment.available_quantity || 0}" min="0">
                </div>
                <div class="form-group">
                    <label>Местоположение</label>
                    <input type="text" name="location" value="${equipment.location || ''}">
                </div>
                <div class="form-group">
                    <label>Статус</label>
                    <select name="status">
                        <option value="available" ${equipment.status === 'available' ? 'selected' : ''}>Доступно</option>
                        <option value="maintenance" ${equipment.status === 'maintenance' ? 'selected' : ''}>На обслуживании</option>
                        <option value="unavailable" ${equipment.status === 'unavailable' ? 'selected' : ''}>Недоступно</option>
                    </select>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary" onclick="closeModals()">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        await loadCategoriesForSelect();
        if (equipment.category_id) {
            document.getElementById('categorySelect').value = equipment.category_id;
        }
        
        document.getElementById('modalOverlay').classList.add('show');
    } catch (error) {
        showError('Ошибка загрузки данных оборудования');
    }
}

async function editCategory(id) {
    if (currentUser.role !== 'admin' && currentUser.role !== 'responsible') {
        showError('Недостаточно прав');
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/categories/${id}`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) {
            throw new Error('Failed to load category');
        }
        
        const category = await response.json();
        
        document.getElementById('modalTitle').textContent = 'Редактировать категорию';
        document.getElementById('modalBody').innerHTML = `
            <form id="categoryForm" onsubmit="updateCategory(event, '${id}')">
                <div class="form-group">
                    <label>Название *</label>
                    <input type="text" name="name" value="${category.name || ''}" required>
                </div>
                <div class="form-group">
                    <label>Описание</label>
                    <textarea name="description">${category.description || ''}</textarea>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary" onclick="closeModals()">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        document.getElementById('modalOverlay').classList.add('show');
    } catch (error) {
        showError('Ошибка загрузки данных категории');
    }
}

async function editUser(id) {
    if (currentUser.role !== 'admin') {
        showError('Только администраторы могут редактировать пользователей');
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/users/${id}`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) {
            throw new Error('Failed to load user');
        }
        
        const user = await response.json();
        
        document.getElementById('modalTitle').textContent = 'Редактировать пользователя';
        document.getElementById('modalBody').innerHTML = `
            <form id="userForm" onsubmit="updateUser(event, '${id}')">
                <div class="form-group">
                    <label>Имя пользователя *</label>
                    <input type="text" name="username" value="${user.username || ''}" required>
                </div>
                <div class="form-group">
                    <label>Новый пароль (оставьте пустым, чтобы не менять)</label>
                    <input type="password" name="password">
                </div>
                <div class="form-group">
                    <label>ФИО</label>
                    <input type="text" name="full_name" value="${user.full_name || ''}">
                </div>
                <div class="form-group">
                    <label>Email</label>
                    <input type="email" name="email" value="${user.email || ''}">
                </div>
                <div class="form-group">
                    <label>Роль *</label>
                    <select name="role" required>
                        <option value="user" ${user.role === 'user' ? 'selected' : ''}>Пользователь</option>
                        <option value="responsible" ${user.role === 'responsible' ? 'selected' : ''}>Ответственный</option>
                        <option value="admin" ${user.role === 'admin' ? 'selected' : ''}>Администратор</option>
                    </select>
                </div>
                <div class="form-actions">
                    <button type="button" class="btn btn-secondary" onclick="closeModals()">Отмена</button>
                    <button type="submit" class="btn btn-primary">Сохранить</button>
                </div>
            </form>
        `;
        
        document.getElementById('modalOverlay').classList.add('show');
    } catch (error) {
        showError('Ошибка загрузки данных пользователя');
    }
}

// Обновление данных
async function updateEquipment(event, id) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const data = Object.fromEntries(formData);
    
    if (data.category_id === '') delete data.category_id;
    data.quantity = parseInt(data.quantity);
    if (data.available_quantity !== undefined) {
        data.available_quantity = parseInt(data.available_quantity);
    }
    
    try {
        const response = await fetch(`${API_BASE}/equipment/${id}`, {
            method: 'PUT',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            showSuccess('Оборудование обновлено');
            closeModals();
            loadEquipment();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка обновления оборудования');
        }
    } catch (error) {
        showError('Ошибка при обновлении оборудования');
    }
}

async function updateCategory(event, id) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const data = Object.fromEntries(formData);
    
    try {
        const response = await fetch(`${API_BASE}/categories/${id}`, {
            method: 'PUT',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            showSuccess('Категория обновлена');
            closeModals();
            loadCategories();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка обновления категории');
        }
    } catch (error) {
        showError('Ошибка при обновлении категории');
    }
}

async function updateUser(event, id) {
    event.preventDefault();
    const formData = new FormData(event.target);
    const data = Object.fromEntries(formData);
    
    // Если пароль не указан, не отправляем его
    if (!data.password || data.password === '') {
        delete data.password;
    }
    
    try {
        const response = await fetch(`${API_BASE}/users/${id}`, {
            method: 'PUT',
            headers: {
                'Authorization': `Bearer ${authToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(data)
        });
        
        if (response.ok) {
            showSuccess('Пользователь обновлён');
            closeModals();
            loadUsers();
        } else {
            const error = await response.json();
            showError(error.message || 'Ошибка обновления пользователя');
        }
    } catch (error) {
        showError('Ошибка при обновлении пользователя');
    }
}

// Вспомогательные функции
async function loadCategoriesForSelect() {
    try {
        const response = await fetch(`${API_BASE}/categories`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (response.ok) {
            const categories = await response.json();
            const select = document.getElementById('categorySelect');
            select.innerHTML = '<option value="">Выберите категорию</option>' +
                categories.map(cat => `<option value="${cat.id}">${cat.name}</option>`).join('');
        }
    } catch (error) {
        console.error('Ошибка загрузки категорий:', error);
    }
}

async function generateQR(id) {
    try {
        // Получаем данные QR-кода
        const dataResponse = await fetch(`${API_BASE}/qr/${id}/data`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        let qrData = '';
        if (dataResponse.ok) {
            const data = await dataResponse.json();
            qrData = data.qr_code || '';
        }
        
        // Создаём URL для QR-кода с токеном авторизации
        const qrUrl = `${API_BASE}/qr/${id}`;
        
        document.getElementById('modalTitle').textContent = 'QR-код оборудования';
        document.getElementById('modalBody').innerHTML = `
            <div class="qr-code-container">
                <div class="qr-code-image">
                    <img src="${qrUrl}?t=${Date.now()}" alt="QR Code" style="display: block; width: 400px; height: 400px; max-width: 100%;" 
                         onerror="this.onerror=null; this.src='data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iNDAwIiBoZWlnaHQ9IjQwMCIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj48cmVjdCB3aWR0aD0iNDAwIiBoZWlnaHQ9IjQwMCIgZmlsbD0iI2Y1ZjVmNSIvPjx0ZXh0IHg9IjUwJSIgeT0iNTAlIiBmb250LWZhbWlseT0iQXJpYWwiIGZvbnQtc2l6ZT0iMTgiIGZpbGw9IiM5OTkiIHRleHQtYW5jaG9yPSJtaWRkbGUiIGR5PSIuM2VtIj5RUtL0vQ8L3RleHQ+PC9zdmc+';">
                </div>
                <div class="qr-code-text">
                    <strong>Код:</strong> ${qrData}
                </div>
                <div style="margin-top: 20px;">
                    <button class="btn btn-primary" onclick="downloadQR('${id}', '${qrUrl}')">Скачать QR-код</button>
                </div>
            </div>
        `;
        
        // Устанавливаем заголовок авторизации для изображения через fetch и создаём blob URL
        try {
            const qrResponse = await fetch(qrUrl, {
                headers: { 'Authorization': `Bearer ${authToken}` }
            });
            
            if (qrResponse.ok) {
                const svgBlob = await qrResponse.blob();
                const svgUrl = URL.createObjectURL(svgBlob);
                const img = document.querySelector('.qr-code-image img');
                if (img) {
                    img.src = svgUrl;
                }
            }
        } catch (e) {
            console.error('Ошибка загрузки QR-кода:', e);
        }
        
        document.getElementById('modalOverlay').classList.add('show');
    } catch (error) {
        console.error('Ошибка генерации QR-кода:', error);
        showError('Ошибка генерации QR-кода');
    }
}

function downloadQR(id, qrUrl) {
    const link = document.createElement('a');
    link.href = qrUrl;
    link.download = `qr-code-${id}.svg`;
    link.target = '_blank';
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);
}

function filterEquipment() {
    const search = document.getElementById('equipmentSearch').value.toLowerCase();
    const rows = document.querySelectorAll('#equipmentTableBody tr');
    
    rows.forEach(row => {
        const text = row.textContent.toLowerCase();
        row.style.display = text.includes(search) ? '' : 'none';
    });
}

// Создание бронирования
async function showAddBookingModal() {
    document.getElementById('modalTitle').textContent = 'Создать бронирование';
    document.getElementById('modalBody').innerHTML = `
        <form id="bookingForm" onsubmit="createBooking(event)">
            <div class="form-group">
                <label>QR-код оборудования (введите код для быстрого поиска)</label>
                <div style="display: flex; gap: 10px;">
                    <input type="text" id="qrCodeInput" placeholder="Введите QR-код..." 
                           style="flex: 1;" onblur="searchEquipmentByQR()">
                    <button type="button" class="btn btn-secondary" onclick="searchEquipmentByQR()">Найти</button>
                </div>
                <small style="color: #666; margin-top: 5px; display: block;">
                    Введите код из QR-кода оборудования для автоматического заполнения формы
                </small>
            </div>
            <div class="form-group">
                <label>Оборудование *</label>
                <select name="equipment_id" id="bookingEquipmentSelect" required>
                    <option value="">Выберите оборудование</option>
                </select>
            </div>
            <div class="form-group">
                <label>Количество *</label>
                <input type="number" name="quantity" id="bookingQuantity" required min="1" value="1">
            </div>
            <div class="form-group">
                <label>Дата начала *</label>
                <input type="datetime-local" name="start_date" id="bookingStartDate" required>
            </div>
            <div class="form-group">
                <label>Дата окончания *</label>
                <input type="datetime-local" name="end_date" id="bookingEndDate" required>
            </div>
            <div class="form-group">
                <label>Цель использования</label>
                <textarea name="purpose" id="bookingPurpose" placeholder="Опишите цель использования оборудования"></textarea>
            </div>
            <div class="form-actions">
                <button type="button" class="btn btn-secondary" onclick="closeModals()">Отмена</button>
                <button type="submit" class="btn btn-primary">Создать бронирование</button>
            </div>
        </form>
    `;
    
    // Загружаем список оборудования
    await loadEquipmentForBooking();
    
    // Устанавливаем минимальную дату на сегодня
    const now = new Date();
    now.setMinutes(now.getMinutes() - now.getTimezoneOffset());
    const minDate = now.toISOString().slice(0, 16);
    document.getElementById('bookingStartDate').min = minDate;
    document.getElementById('bookingEndDate').min = minDate;
    
    document.getElementById('modalOverlay').classList.add('show');
}

async function searchEquipmentByQR() {
    const qrCodeInput = document.getElementById('qrCodeInput');
    const qrCode = qrCodeInput.value.trim();
    
    if (!qrCode) {
        showError('Введите QR-код');
        return;
    }
    
    try {
        const response = await fetch(`${API_BASE}/equipment/qr/${encodeURIComponent(qrCode)}`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) {
            if (response.status === 404) {
                showError('Оборудование с таким QR-кодом не найдено');
            } else {
                throw new Error('Failed to search equipment');
            }
            return;
        }
        
        const equipment = await response.json();
        
        // Заполняем форму найденным оборудованием
        const equipmentSelect = document.getElementById('bookingEquipmentSelect');
        equipmentSelect.value = equipment.id;
        
        // Устанавливаем максимальное количество
        const quantityInput = document.getElementById('bookingQuantity');
        quantityInput.max = equipment.available_quantity || equipment.quantity;
        quantityInput.value = Math.min(1, equipment.available_quantity || 1);
        
        showSuccess(`Найдено оборудование: ${equipment.name}`);
        
        // Если оборудование недоступно, показываем предупреждение
        if (equipment.status !== 'available' || equipment.available_quantity === 0) {
            showError(`Внимание: Оборудование "${equipment.name}" недоступно для бронирования`);
        }
    } catch (error) {
        console.error('Ошибка поиска оборудования:', error);
        showError('Ошибка при поиске оборудования по QR-коду');
    }
}

async function loadEquipmentForBooking() {
    try {
        const response = await fetch(`${API_BASE}/equipment`, {
            headers: { 'Authorization': `Bearer ${authToken}` }
        });
        
        if (!response.ok) throw new Error('Failed to load equipment');
        
        const equipment = await response.json();
        const select = document.getElementById('bookingEquipmentSelect');
        
        // Очищаем опции кроме первой
        select.innerHTML = '<option value="">Выберите оборудование</option>';
        
        equipment.forEach(eq => {
            if (eq.status === 'available' && eq.available_quantity > 0) {
                const option = document.createElement('option');
                option.value = eq.id;
                option.textContent = `${eq.name} (Доступно: ${eq.available_quantity}/${eq.quantity})`;
                option.dataset.availableQuantity = eq.available_quantity;
                select.appendChild(option);
            }
        });
        
        // Обновляем максимальное количество при изменении выбора оборудования
        select.addEventListener('change', function() {
            const selectedOption = this.options[this.selectedIndex];
            if (selectedOption.value) {
                const availableQty = parseInt(selectedOption.dataset.availableQuantity || '0');
                const quantityInput = document.getElementById('bookingQuantity');
                quantityInput.max = availableQty;
                if (parseInt(quantityInput.value) > availableQty) {
                    quantityInput.value = availableQty;
                }
            }
        });
    } catch (error) {
        console.error('Ошибка загрузки оборудования:', error);
        showError('Не удалось загрузить список оборудования');
    }
}

async function createBooking(event) {
    event.preventDefault();
    
    const form = event.target;
    const formData = new FormData(form);
    
    const equipmentId = formData.get('equipment_id');
    if (!equipmentId) {
        showError('Выберите оборудование');
        return;
    }
    
    const startDate = formData.get('start_date');
    const endDate = formData.get('end_date');
    
    if (new Date(startDate) >= new Date(endDate)) {
        showError('Дата окончания должна быть позже даты начала');
        return;
    }
    
    const bookingData = {
        equipment_id: equipmentId,
        quantity: parseInt(formData.get('quantity')),
        start_date: new Date(startDate).toISOString(),
        end_date: new Date(endDate).toISOString(),
        purpose: formData.get('purpose') || null
    };
    
    try {
        const response = await fetch(`${API_BASE}/bookings`, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${authToken}`
            },
            body: JSON.stringify(bookingData)
        });
        
        if (!response.ok) {
            const error = await response.json();
            showError(error.message || 'Ошибка создания бронирования');
            return;
        }
        
        showSuccess('Бронирование успешно создано');
        closeModals();
        loadBookings();
    } catch (error) {
        console.error('Ошибка создания бронирования:', error);
        showError('Ошибка при создании бронирования');
    }
}

// Уведомления
function showSuccess(message) {
    showNotification(message, 'success');
}

function showError(message) {
    showNotification(message, 'error');
}

function showNotification(message, type) {
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    notification.textContent = message;
    notification.style.cssText = `
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 15px 20px;
        background: ${type === 'success' ? '#4caf50' : '#f44336'};
        color: white;
        border-radius: 4px;
        z-index: 10000;
        box-shadow: 0 2px 5px rgba(0,0,0,0.2);
    `;
    
    document.body.appendChild(notification);
    
    setTimeout(() => {
        notification.remove();
    }, 3000);
}


