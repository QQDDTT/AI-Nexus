import Header from '../components/Header';
import { useNavigate } from 'react-router-dom';
import { ShieldCheck, HardDrives, Palette, User } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES } from '../constants';

interface SettingsDTO {
    db_path: string;
    session_timeout_ms: number;
    log_masking: boolean;
    admin_username: string;
    admin_email: string;
    avatar_base64?: string;
    theme?: string;
}

export default function Settings() {
    const navigate = useNavigate();
    const [settings, setSettings] = useState<SettingsDTO | null>(null);
    const [isSaving, setIsSaving] = useState(false);
    const [toast, setToast] = useState<{msg: string, type: 'success' | 'error'} | null>(null);

    // Security tab password states
    const [oldPass, setOldPass] = useState('');
    const [newPass, setNewPass] = useState('');
    const [confirmPass, setConfirmPass] = useState('');
    
    // Parse tab from URL
    const getInitialTab = (): 'profile' | 'system' | 'security' | 'ui' => {
        const params = new URLSearchParams(window.location.search);
        const tab = params.get('tab');
        if (tab === 'profile' || tab === 'system' || tab === 'security' || tab === 'ui') {
            return tab;
        }
        return 'system';
    };
    
    const [activeTab, setActiveTab] = useState<'profile' | 'system' | 'security' | 'ui'>(getInitialTab());

    const handleUpdatePassword = () => {
        if (!newPass) return alert('请输入新密码');
        if (newPass !== confirmPass) return alert('两次输入的密码不一致');
        fetchApi(API_ROUTES.SETTINGS, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ...settings, admin_password: newPass })
        }).then(res => {
            if (res.ok) {
                alert('密码已成功更新！');
                setOldPass('');
                setNewPass('');
                setConfirmPass('');
            } else {
                alert('更新密码失败');
            }
        });
    };

    const handleLogout = () => {
        localStorage.removeItem('token');
        navigate('/login');
    };

    // Update URL when tab changes without page reload
    useEffect(() => {
        const url = new URL(window.location.href);
        url.searchParams.set('tab', activeTab);
        window.history.pushState({}, '', url.toString());
    }, [activeTab]);

    useEffect(() => {
        fetchApi(API_ROUTES.SETTINGS)
            .then(res => res.json())
            .then(data => setSettings(data))
            .catch(err => console.error(err));
    }, []);

    const showToast = (msg: string, type: 'success' | 'error') => {
        setToast({ msg, type });
        setTimeout(() => setToast(null), 3000);
    };

    const handleSave = () => {
        if (!settings) return;
        if (settings.session_timeout_ms < 0) {
            showToast('超时时间不能为负数', 'error');
            return;
        }
        
        setIsSaving(true);
        fetchApi(API_ROUTES.SETTINGS, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(settings)
        })
        .then(res => {
            if (!res.ok) throw new Error(`HTTP ${res.status}`);
            showToast('Settings saved successfully', 'success');
        })
        .catch(err => {
            console.error(err);
            showToast('Failed to save settings', 'error');
        })
        .finally(() => setIsSaving(false));
    };

    return (
        <div className="main-content" style={{ position: 'relative' }}>
            {toast && (
                <div style={{
                    position: 'absolute', top: '20px', right: '20px', zIndex: 9999,
                    background: toast.type === 'success' ? 'var(--secondary-color)' : 'var(--error-color)',
                    color: '#fff', padding: '12px 24px', borderRadius: '8px',
                    boxShadow: '0 4px 12px rgba(0,0,0,0.15)', fontWeight: 500,
                    animation: 'fadeIn 0.3s ease-out'
                }}>
                    {toast.msg}
                </div>
            )}
            <Header 
                title="核心配置 Settings" 
                description="平台全局参数与安全性设置。" 
            />

            <div style={{ display: 'flex', gap: '2rem', height: '100%' }}>
                <div className="settings-nav" style={{ width: '240px', display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                    <button className={`nav-item ${activeTab === 'profile' ? 'active' : ''}`} onClick={() => setActiveTab('profile')} style={{ background: 'transparent', border: 'none', width: '100%', textAlign: 'left', cursor: 'pointer' }}>
                        <User size={20} /> 个人资料
                    </button>
                    <button className={`nav-item ${activeTab === 'system' ? 'active' : ''}`} onClick={() => setActiveTab('system')} style={{ background: 'transparent', border: 'none', width: '100%', textAlign: 'left', cursor: 'pointer' }}>
                        <HardDrives size={20} /> 系统 & 存储
                    </button>
                    <button className={`nav-item ${activeTab === 'security' ? 'active' : ''}`} onClick={() => setActiveTab('security')} style={{ background: 'transparent', border: 'none', width: '100%', textAlign: 'left', cursor: 'pointer' }}>
                        <ShieldCheck size={20} /> 账号与安全设置
                    </button>
                    <button className={`nav-item ${activeTab === 'ui' ? 'active' : ''}`} onClick={() => setActiveTab('ui')} style={{ background: 'transparent', border: 'none', width: '100%', textAlign: 'left', cursor: 'pointer' }}>
                        <Palette size={20} /> 个性化 (UI)
                    </button>
                </div>

                <div className="panel" style={{ flex: 1 }}>
                    <div className="panel-header" style={{ borderBottom: '1px solid var(--surface-border)', paddingBottom: '1rem' }}>
                        <span className="panel-title">
                            {activeTab === 'profile' ? '个人资料 (Profile)' : activeTab === 'system' ? '系统 & 存储设置' : activeTab === 'security' ? '账号与安全设置' : '个性化设置'}
                        </span>
                    </div>
                    
                    {activeTab === 'system' && (
                        settings ? (
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem', marginTop: '1.5rem' }}>
                                <div>
                                    <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: '0.5rem' }}>
                                        <label style={{ color: 'var(--text-secondary)' }}>OS Storage 底层数据库路径</label>
                                        <span style={{ fontSize: '0.75rem', padding: '4px 10px', background: 'rgba(74, 222, 128, 0.2)', color: '#4ade80', borderRadius: '50px', border: '1px solid rgba(74, 222, 128, 0.3)' }}>● 存储引擎就绪</span>
                                    </div>
                                    <select 
                                        value={settings.db_path} 
                                        onChange={e => setSettings({...settings, db_path: e.target.value})}
                                        className="custom-select"
                                        style={{ fontFamily: 'monospace' }}
                                    >
                                        <option value="sqlite::memory:" style={{ background: 'var(--bg-color)' }}>sqlite::memory: (内存模式 - 极速但数据易失)</option>
                                        <option value="sqlite://data/nexus.db" style={{ background: 'var(--bg-color)' }}>sqlite://data/nexus.db (本地文件持久化 - 安全可靠)</option>
                                    </select>
                                    <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginTop: '0.5rem' }}>AI-Nexus 的核心 Storage 引擎，存储所有的 Dashboard 遥测数据、Token 账单及历史记忆。</p>
                                </div>
                                
                                <div>
                                    <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-secondary)' }}>全局会话超时时间 (ms)</label>
                                    <input 
                                        type="number" 
                                        min="0"
                                        value={settings.session_timeout_ms}
                                        onChange={e => {
                                            const val = parseInt(e.target.value);
                                            setSettings({...settings, session_timeout_ms: isNaN(val) ? 0 : Math.max(0, val)});
                                        }}
                                        style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }} 
                                    />
                                    <p style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginTop: '0.5rem' }}>如果模型在此时长内无响应，则判定为超时并由 Garbage Collector 强制回收。</p>
                                </div>

                                <div style={{ display: 'flex', alignItems: 'center', gap: '1rem', marginTop: '1rem', padding: '1rem', background: 'rgba(255,255,255,0.02)', borderRadius: '8px', border: '1px solid var(--surface-border)' }}>
                                    <div style={{ flex: 1 }}>
                                        <div style={{ fontWeight: 500 }}>启用本地日志脱敏 (Masking)</div>
                                        <div style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>自动对日志文件中的 API Key 等敏感信息打码。</div>
                                    </div>
                                    <div 
                                        onClick={() => setSettings({...settings, log_masking: !settings.log_masking})}
                                        style={{ width: '44px', height: '24px', background: settings.log_masking ? 'var(--primary-color)' : 'var(--surface-border)', borderRadius: '50px', position: 'relative', cursor: 'pointer', transition: 'all 0.2s' }}
                                    >
                                        <div style={{ width: '18px', height: '18px', background: '#fff', borderRadius: '50%', position: 'absolute', top: '3px', right: settings.log_masking ? '3px' : '23px', transition: 'all 0.2s' }}></div>
                                    </div>
                                </div>

                                <div style={{ marginTop: '2rem', display: 'flex', justifyContent: 'flex-end' }}>
                                    <button className="btn-primary" onClick={handleSave} disabled={isSaving}>
                                        {isSaving ? 'Saving...' : '保存设置'}
                                    </button>
                                </div>
                            </div>
                        ) : (
                            <p style={{ marginTop: '1.5rem' }}>Loading settings...</p>
                        )
                    )}

                    {activeTab === 'profile' && (
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem', marginTop: '1.5rem' }}>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '1.5rem', padding: '1.5rem', background: 'rgba(255,255,255,0.02)', borderRadius: '12px', border: '1px solid var(--surface-border)' }}>
                                <div style={{ width: '80px', height: '80px', borderRadius: '50%', background: 'linear-gradient(135deg, var(--primary-color), var(--accent-color))', display: 'flex', alignItems: 'center', justifyContent: 'center', fontSize: '2rem', fontWeight: 600, overflow: 'hidden' }}>
                                    {settings?.avatar_base64 ? (
                                        <img src={settings.avatar_base64} alt="Avatar" style={{ width: '100%', height: '100%', objectFit: 'cover' }} />
                                    ) : (
                                        settings?.admin_username ? settings.admin_username.charAt(0).toUpperCase() : 'A'
                                    )}
                                </div>
                                <div style={{ flex: 1 }}>
                                    <h2 style={{ fontSize: '1.5rem', marginBottom: '0.25rem' }}>{settings?.admin_username}</h2>
                                    <p style={{ color: 'var(--text-secondary)' }}>Super Administrator</p>
                                </div>
                                <div>
                                    <input 
                                        type="file" 
                                        id="avatarUpload" 
                                        accept="image/*" 
                                        style={{ display: 'none' }}
                                        onChange={(e) => {
                                            const file = e.target.files?.[0];
                                            if (file && settings) {
                                                const reader = new FileReader();
                                                reader.onload = (ev) => {
                                                    setSettings({...settings, avatar_base64: ev.target?.result as string});
                                                };
                                                reader.readAsDataURL(file);
                                            }
                                        }}
                                    />
                                    <label htmlFor="avatarUpload" className="btn-outline" style={{ cursor: 'pointer', display: 'inline-block' }}>更换头像</label>
                                </div>
                            </div>
                            
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-secondary)' }}>用户名</label>
                                    <input type="text" value={settings?.admin_username || ''} onChange={e => settings && setSettings({...settings, admin_username: e.target.value})} style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }} />
                                </div>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-secondary)' }}>联系邮箱</label>
                                    <input type="email" value={settings?.admin_email || ''} onChange={e => settings && setSettings({...settings, admin_email: e.target.value})} placeholder="admin@ai-nexus.local" style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }} />
                                </div>
                            </div>
                            
                            <div style={{ display: 'flex', justifyContent: 'flex-end' }}>
                                <button className="btn-primary" onClick={handleSave} disabled={isSaving}>{isSaving ? 'Saving...' : '保存资料'}</button>
                            </div>
                        </div>
                    )}

                    {activeTab === 'security' && (
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem', marginTop: '1.5rem' }}>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
                                <h3>修改密码</h3>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-secondary)' }}>当前密码</label>
                                    <input 
                                        type="password" 
                                        value={oldPass}
                                        onChange={e => setOldPass(e.target.value)}
                                        placeholder="请输入当前密码" 
                                        style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }} 
                                    />
                                </div>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-secondary)' }}>新密码</label>
                                    <input 
                                        type="password" 
                                        value={newPass}
                                        onChange={e => setNewPass(e.target.value)}
                                        placeholder="请输入新密码" 
                                        style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }} 
                                    />
                                </div>
                                <div>
                                    <label style={{ display: 'block', marginBottom: '0.5rem', color: 'var(--text-secondary)' }}>确认新密码</label>
                                    <input 
                                        type="password" 
                                        value={confirmPass}
                                        onChange={e => setConfirmPass(e.target.value)}
                                        placeholder="请再次输入新密码" 
                                        style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }} 
                                    />
                                </div>
                                <div style={{ display: 'flex', justifyContent: 'flex-start' }}>
                                    <button className="btn-outline" onClick={handleUpdatePassword}>更新密码</button>
                                </div>
                            </div>
                            
                            <div style={{ height: '1px', background: 'var(--surface-border)', margin: '1rem 0' }}></div>
                            
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                                <h3>会话与安全</h3>
                                <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', padding: '1rem', background: 'rgba(239, 68, 68, 0.05)', border: '1px solid rgba(239, 68, 68, 0.2)', borderRadius: '8px' }}>
                                    <div>
                                        <div style={{ fontWeight: 500, color: '#ef4444' }}>强制登出所有设备</div>
                                        <div style={{ fontSize: '0.85rem', color: 'var(--text-secondary)', marginTop: '0.25rem' }}>吊销所有当前已签发的 JWT Token，并清空所有活动中的管理员会话。</div>
                                    </div>
                                    <button className="btn-outline" style={{ borderColor: '#ef4444', color: '#ef4444' }} onClick={handleLogout}>执行登出</button>
                                </div>
                            </div>
                        </div>
                    )}

                    {activeTab === 'ui' && (
                        <div style={{ display: 'flex', flexDirection: 'column', gap: '2rem', marginTop: '1.5rem' }}>
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                                <h3>主题色选型 (Theme)</h3>
                                <p style={{ color: 'var(--text-secondary)', fontSize: '0.9rem' }}>选择全局界面配色，设置将多端同步持久化至 NexusDB。</p>
                                <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(140px, 1fr))', gap: '1rem' }}>
                                    {[
                                        { id: 'default', name: 'Indigo (默认)', primary: '#6366f1' },
                                        { id: 'emerald', name: 'Emerald', primary: '#10b981' },
                                        { id: 'sunset', name: 'Sunset', primary: '#f97316' },
                                        { id: 'cyberpunk', name: 'Cyberpunk', primary: '#f43f5e' }
                                    ].map(theme => (
                                        <div 
                                            key={theme.id}
                                            onClick={() => {
                                                const currentSettings = settings || {} as SettingsDTO;
                                                setSettings({...currentSettings, theme: theme.id});
                                                document.documentElement.setAttribute('data-theme', theme.id);
                                            }}
                                            style={{ 
                                                padding: '1rem', 
                                                border: `2px solid ${settings?.theme === theme.id || (!settings?.theme && theme.id === 'default') ? 'var(--primary-color)' : 'var(--surface-border)'}`, 
                                                borderRadius: '12px', 
                                                cursor: 'pointer',
                                                display: 'flex',
                                                flexDirection: 'column',
                                                alignItems: 'center',
                                                gap: '0.75rem',
                                                background: 'rgba(255,255,255,0.02)',
                                                transition: 'all 0.2s'
                                            }}
                                        >
                                            <div style={{ width: '32px', height: '32px', borderRadius: '50%', background: theme.primary, boxShadow: `0 0 10px ${theme.primary}80` }}></div>
                                            <span style={{ fontWeight: 500 }}>{theme.name}</span>
                                        </div>
                                    ))}
                                </div>
                            </div>
                            <div style={{ display: 'flex', justifyContent: 'flex-end', marginTop: '1rem' }}>
                                <button className="btn-primary" onClick={handleSave} disabled={isSaving}>{isSaving ? 'Saving...' : '保存设置'}</button>
                            </div>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
