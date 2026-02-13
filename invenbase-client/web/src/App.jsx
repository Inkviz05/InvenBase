import React from 'react';
import { HashRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './context/AuthContext';
import { CartProvider } from './context/CartContext';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import EquipmentList from './pages/EquipmentList';
import EquipmentDetail from './pages/EquipmentDetail';
import EquipmentCreate from './pages/EquipmentCreate';
import EquipmentEdit from './pages/EquipmentEdit';
import BookingList from './pages/BookingList';
import BookingCreate from './pages/BookingCreate';
import BookingBulkCreate from './pages/BookingBulkCreate';
import QRScanner from './pages/QRScanner';
import Notifications from './pages/Notifications';
import Categories from './pages/Categories';
import Squads from './pages/Squads';
import SquadDetail from './pages/SquadDetail';
import Users from './pages/Users';
import Reports from './pages/Reports';
import Logs from './pages/Logs';
import Support from './pages/Support';
import Layout from './components/Layout';
import './index.css';

const PrivateRoute = ({ children }) => {
  const { isAuthenticated, loading } = useAuth();
  
  if (loading) {
    return <div className="loading"><div className="spinner"></div></div>;
  }
  
  return isAuthenticated() ? children : <Navigate to="/login" />;
};

const AppRoutes = () => {
  return (
    <Routes>
      <Route path="/login" element={<Login />} />
      <Route
        path="/"
        element={
          <PrivateRoute>
            <Layout />
          </PrivateRoute>
        }
      >
        <Route index element={<Dashboard />} />
        <Route path="equipment" element={<EquipmentList />} />
        <Route path="equipment/create" element={<EquipmentCreate />} />
        <Route path="equipment/:id" element={<EquipmentDetail />} />
        <Route path="equipment/:id/edit" element={<EquipmentEdit />} />
        <Route path="bookings" element={<BookingList />} />
        <Route path="bookings/create" element={<BookingCreate />} />
        <Route path="bookings/bulk" element={<BookingBulkCreate />} />
        <Route path="scanner" element={<QRScanner />} />
        <Route path="notifications" element={<Notifications />} />
        <Route path="categories" element={<Categories />} />
        <Route path="squads" element={<Squads />} />
        <Route path="squads/:id" element={<SquadDetail />} />
        <Route path="users" element={<Users />} />
        <Route path="reports" element={<Reports />} />
        <Route path="logs" element={<Logs />} />
        <Route path="support" element={<Support />} />
      </Route>
    </Routes>
  );
};

function App() {
  console.log('App component rendering...');
  
  return (
    <AuthProvider>
      <CartProvider>
        <Router>
          <AppRoutes />
        </Router>
      </CartProvider>
    </AuthProvider>
  );
}

export default App;

