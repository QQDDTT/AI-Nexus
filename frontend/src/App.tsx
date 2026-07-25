import { BrowserRouter as Router, Routes, Route, Navigate, useLocation } from 'react-router-dom';
import Sidebar from './components/Sidebar';
import Dashboard from './pages/Dashboard';
import Sessions from './pages/Sessions';
import ModelRouter from './pages/ModelRouter';
import TokenLedger from './pages/TokenLedger';
import Gateways from './pages/Gateways';
import Settings from './pages/Settings';
import Skills from './pages/Skills';
import Login from './pages/Login';
import AgentFactory from './pages/AgentFactory';
import Personas from './pages/Personas';
import TaskScheduler from './pages/TaskScheduler';
import { getToken } from './utils/auth';
import './App.css';

import React, { useEffect } from 'react';
const ProtectedRoute = ({ children }: { children: React.ReactNode }) => {
    const token = getToken();
    const location = useLocation();

    if (!token) {
        return <Navigate to="/login" state={{ from: location }} replace />;
    }

    useEffect(() => {
        if (token) {
            fetch('/api/settings', {
                headers: { 'Authorization': `Bearer ${token}` }
            })
            .then(res => res.json())
            .then(data => {
                if (data.theme) {
                    document.documentElement.setAttribute('data-theme', data.theme);
                }
            })
            .catch(console.error);
        }
    }, [token]);

    return (
        <>
            <Sidebar />
            {children}
        </>
    );
};

function App() {
  return (
    <Router>
      <div className="app-container">
        <Routes>
          <Route path="/login" element={<Login />} />
          <Route path="/" element={<ProtectedRoute><Dashboard /></ProtectedRoute>} />
          <Route path="/sessions" element={<ProtectedRoute><Sessions /></ProtectedRoute>} />
          <Route path="/model-router" element={<ProtectedRoute><ModelRouter /></ProtectedRoute>} />
          <Route path="/token-ledger" element={<ProtectedRoute><TokenLedger /></ProtectedRoute>} />
          <Route path="/gateways" element={<ProtectedRoute><Gateways /></ProtectedRoute>} />
          <Route path="/skills" element={<ProtectedRoute><Skills /></ProtectedRoute>} />
          <Route path="/agent-factory" element={<ProtectedRoute><AgentFactory /></ProtectedRoute>} />
          <Route path="/personas" element={<ProtectedRoute><Personas /></ProtectedRoute>} />
          <Route path="/task-scheduler" element={<ProtectedRoute><TaskScheduler /></ProtectedRoute>} />
          <Route path="/settings" element={<ProtectedRoute><Settings /></ProtectedRoute>} />
        </Routes>
      </div>
    </Router>
  );
}

export default App;
