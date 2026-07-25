import { useState } from 'react';
import { Planet, LockKey } from '@phosphor-icons/react';
import { setToken } from '../utils/auth';
import { API_ROUTES } from '../constants';

export default function Login() {
    const [username, setUsername] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState('');
    const [loading, setLoading] = useState(false);

    const handleLogin = async (e: React.FormEvent) => {
        e.preventDefault();
        setError('');
        setLoading(true);

        try {
            const res = await fetch(API_ROUTES.LOGIN, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ username, password })
            });

            if (res.ok) {
                const data = await res.json();
                setToken(data.token);
                window.location.href = '/';
            } else {
                setError('Invalid username or password');
            }
        } catch (err) {
            setError('Failed to connect to the server');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div style={{ display: 'flex', width: '100vw', height: '100vh', alignItems: 'center', justifyContent: 'center', background: 'var(--bg-color)' }}>
            <div className="panel" style={{ width: '400px', display: 'flex', flexDirection: 'column', alignItems: 'center', padding: '3rem 2rem' }}>
                <div className="brand" style={{ marginBottom: '1rem', fontSize: '2rem' }}>
                    <Planet weight="fill" /> AI-Nexus
                </div>
                <p style={{ color: 'var(--text-secondary)', marginBottom: '2rem' }}>Sign in to access the command center</p>

                {error && <div style={{ color: 'var(--error-color)', background: 'rgba(239, 68, 68, 0.1)', padding: '0.75rem', borderRadius: '8px', width: '100%', textAlign: 'center', marginBottom: '1.5rem', border: '1px solid rgba(239, 68, 68, 0.2)' }}>{error}</div>}

                <form onSubmit={handleLogin} style={{ width: '100%', display: 'flex', flexDirection: 'column', gap: '1.25rem' }}>
                    <div>
                        <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-secondary)', fontSize: '0.9rem' }}>Username</label>
                        <input 
                            type="text" 
                            value={username} 
                            onChange={e => setUsername(e.target.value)}
                            required
                            style={{ width: '100%', padding: '0.85rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }}
                        />
                    </div>
                    <div>
                        <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-secondary)', fontSize: '0.9rem' }}>Password</label>
                        <div style={{ position: 'relative' }}>
                            <input 
                                type="password" 
                                value={password} 
                                onChange={e => setPassword(e.target.value)}
                                required
                                style={{ width: '100%', padding: '0.85rem', paddingRight: '2.5rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }}
                            />
                            <LockKey size={20} color="var(--text-secondary)" style={{ position: 'absolute', right: '12px', top: '50%', transform: 'translateY(-50%)' }} />
                        </div>
                    </div>
                    <button type="submit" className="btn-primary" style={{ marginTop: '1rem', padding: '0.85rem' }} disabled={loading}>
                        {loading ? 'Authenticating...' : 'Secure Login'}
                    </button>
                </form>
            </div>
        </div>
    );
}
