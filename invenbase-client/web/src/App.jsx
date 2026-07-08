import React, { Suspense } from 'react';
import { HashRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider, useAuth } from './context/AuthContext';
import { CartProvider } from './context/CartContext';
import Layout from './components/Layout';
import './index.css';

const Login = React.lazy(() => import('./pages/Login'));
const Dashboard = React.lazy(() => import('./pages/Dashboard'));
const EquipmentList = React.lazy(() => import('./pages/EquipmentList'));
const EquipmentDetail = React.lazy(() => import('./pages/EquipmentDetail'));
const EquipmentCreate = React.lazy(() => import('./pages/EquipmentCreate'));
const EquipmentEdit = React.lazy(() => import('./pages/EquipmentEdit'));
const BookingList = React.lazy(() => import('./pages/BookingList'));
const BookingCreate = React.lazy(() => import('./pages/BookingCreate'));
const BookingBulkCreate = React.lazy(() => import('./pages/BookingBulkCreate'));
const QRScanner = React.lazy(() => import('./pages/QRScanner'));
const Notifications = React.lazy(() => import('./pages/Notifications'));
const Categories = React.lazy(() => import('./pages/Categories'));
const Squads = React.lazy(() => import('./pages/Squads'));
const SquadDetail = React.lazy(() => import('./pages/SquadDetail'));
const Users = React.lazy(() => import('./pages/Users'));
const Reports = React.lazy(() => import('./pages/Reports'));
const Logs = React.lazy(() => import('./pages/Logs'));
const Support = React.lazy(() => import('./pages/Support'));

const PageLoader = () => <div className="loading"><div className="spinner"></div></div>;

const PrivateRoute = ({ children }) => {
  const { isAuthenticated, loading } = useAuth();
  
  if (loading) {
    return <PageLoader />;
  }
  
  return isAuthenticated() ? children : <Navigate to="/login" />;
};

const AppRoutes = () => {
  return (
    <Suspense fallback={<PageLoader />}>
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
    </Suspense>
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

