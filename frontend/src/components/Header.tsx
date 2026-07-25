import { CaretDown, SignOut, User, Gear } from '@phosphor-icons/react';
import { useState, useRef, useEffect } from 'react';
import { removeToken } from '../utils/auth';

interface HeaderProps {
    title: string;
    description: string;
}

export default function Header({ title, description }: HeaderProps) {
    const [isMenuOpen, setIsMenuOpen] = useState(false);
    const menuRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (menuRef.current && !menuRef.current.contains(event.target as Node)) {
                setIsMenuOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    return (
        <header>
            <div className="page-title">
                <h1>{title}</h1>
                <p>{description}</p>
            </div>
            <div className="header-actions">
                <div className="user-profile" style={{ position: 'relative', cursor: 'pointer' }} onClick={() => setIsMenuOpen(!isMenuOpen)} ref={menuRef}>
                    <div className="avatar">A</div>
                    <span style={{ fontSize: '0.9rem', fontWeight: 500 }}>Admin</span>
                    <CaretDown weight="bold" color="var(--text-secondary)" />
                    
                    {isMenuOpen && (
                        <div style={{ 
                            position: 'absolute', 
                            top: '120%', 
                            right: 0, 
                            background: 'rgba(25, 28, 36, 0.95)', 
                            backdropFilter: 'blur(16px)', 
                            WebkitBackdropFilter: 'blur(16px)', 
                            border: '1px solid var(--surface-border)', 
                            borderRadius: '8px', 
                            padding: '0.5rem', 
                            width: '160px', 
                            boxShadow: '0 10px 25px rgba(0,0,0,0.5)', 
                            zIndex: 9999 
                        }}>
                            <div style={{ padding: '0.6rem 0.8rem', display: 'flex', alignItems: 'center', gap: '0.5rem', borderRadius: '4px', cursor: 'pointer' }} onClick={() => { setIsMenuOpen(false); window.location.href = '/settings?tab=profile'; }} className="dropdown-item"><User size={18} /> 个人资料</div>
                            <div style={{ padding: '0.6rem 0.8rem', display: 'flex', alignItems: 'center', gap: '0.5rem', borderRadius: '4px', cursor: 'pointer' }} onClick={() => { setIsMenuOpen(false); window.location.href = '/settings?tab=security'; }} className="dropdown-item"><Gear size={18} /> 账号设置</div>
                            <div style={{ height: '1px', background: 'var(--surface-border)', margin: '0.5rem 0' }}></div>
                            <div style={{ padding: '0.6rem 0.8rem', display: 'flex', alignItems: 'center', gap: '0.5rem', borderRadius: '4px', cursor: 'pointer', color: '#ef4444' }} onClick={() => { removeToken(); window.location.href = '/login'; }} className="dropdown-item"><SignOut size={18} /> 退出登录</div>
                        </div>
                    )}
                </div>
            </div>
        </header>
    );
}
