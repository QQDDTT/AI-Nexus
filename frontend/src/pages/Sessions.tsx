import Header from '../components/Header';
import Modal from '../components/Modal';
import { Funnel, ArrowsClockwise, Eye, XCircle, TelegramLogo, SlackLogo, Browser } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES } from '../constants';

interface SessionDTO {
    session_id: string;
    source: string;
    model: string;
    tokens: number;
    status: string;
}

export default function Sessions() {
    const [sessions, setSessions] = useState<SessionDTO[]>([]);
    const [loading, setLoading] = useState(false);
    
    // Filter State
    const [isFilterModalOpen, setIsFilterModalOpen] = useState(false);
    const [filterSource, setFilterSource] = useState<string>('All');

    // Detail Modal State
    const [detailSession, setDetailSession] = useState<SessionDTO | null>(null);

    const fetchSessions = () => {
        setLoading(true);
        fetchApi(API_ROUTES.SESSIONS)
            .then(res => res.json())
            .then(data => {
                setSessions(data);
                setLoading(false);
            })
            .catch(err => {
                console.error(err);
                setLoading(false);
            });
    };

    useEffect(() => {
        fetchSessions();
    }, []);

    const killSession = (id: string) => {
        if (!window.confirm(`确定要强制终止会话 ${id} 吗？`)) return;
        setLoading(true);
        fetchApi(API_ROUTES.SESSION_BY_ID(id), { method: 'DELETE' })
            .then(res => {
                if (!res.ok) throw new Error(`HTTP ${res.status}`);
                return fetchSessions();
            })
            .catch(err => {
                console.error('Failed to kill session:', err);
                alert(`终止会话失败: ${err.message}`);
                setLoading(false);
            });
    };

    const filteredSessions = sessions.filter(s => {
        if (filterSource === 'All') return true;
        if (filterSource === 'Telegram') return s.source.includes('Telegram');
        if (filterSource === 'Slack') return s.source.includes('Slack');
        if (filterSource === 'Web') return !s.source.includes('Telegram') && !s.source.includes('Slack');
        return true;
    });

    return (
        <div className="main-content">
            <Header 
                title="活动会话 Sessions" 
                description="管理并监控当前正在进行的对话会话。" 
            />

            <section className="panel">
                <div className="panel-header">
                    <span className="panel-title">会话列表 {filterSource !== 'All' && <span style={{fontSize: '0.8rem', color: 'var(--secondary-color)'}}>(Filtered by {filterSource})</span>}</span>
                    <div style={{ display: 'flex', gap: '1rem' }}>
                        <button className="action-btn" onClick={() => setIsFilterModalOpen(true)}><Funnel size={18} style={{marginRight: '4px'}} /> 筛选</button>
                        <button className="action-btn" onClick={fetchSessions} disabled={loading}>
                            <ArrowsClockwise size={18} style={{marginRight: '4px', animation: loading ? 'spin 1s linear infinite' : 'none'}} /> 刷新
                        </button>
                    </div>
                </div>
                <table className="data-table">
                    <thead>
                        <tr>
                            <th>Session ID</th>
                            <th>User / Source</th>
                            <th>Model Routing</th>
                            <th>Tokens</th>
                            <th>Status</th>
                            <th>Actions</th>
                        </tr>
                    </thead>
                    <tbody>
                        {filteredSessions.length > 0 ? filteredSessions.map((s, i) => {
                            const isTg = s.source.includes('Telegram');
                            const isSlack = s.source.includes('Slack');
                            return (
                                <tr key={i}>
                                    <td style={{ fontFamily: 'monospace', color: 'var(--text-secondary)' }}>{s.session_id}</td>
                                    <td style={{ display: 'flex', alignItems: 'center', gap: '4px' }}>
                                        {isTg ? <TelegramLogo size={18} color="#3b82f6" /> : isSlack ? <SlackLogo size={18} color="#e11d48" /> : <Browser size={18} color="#10b981" />} 
                                        {s.source}
                                    </td>
                                    <td>{s.model}</td>
                                    <td>{s.tokens.toLocaleString()}</td>
                                    <td><span className={`status-badge status-${s.status === 'Processing' || s.status === 'Thinking' || s.status === 'Acting' ? 'active' : 'waiting'}`}>{s.status}</span></td>
                                    <td>
                                        <button className="action-btn" onClick={() => setDetailSession(s)}><Eye size={18} /></button>
                                        <button className="action-btn danger" onClick={() => killSession(s.session_id)}><XCircle size={18} /></button>
                                    </td>
                                </tr>
                            );
                        }) : <tr><td colSpan={6}>{loading ? 'Loading...' : 'No sessions found'}</td></tr>}
                    </tbody>
                </table>
            </section>

            <Modal
                isOpen={isFilterModalOpen}
                onClose={() => setIsFilterModalOpen(false)}
                title="筛选会话"
                footer={<button className="btn-primary" onClick={() => setIsFilterModalOpen(false)}>完成</button>}
            >
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                    <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer' }}>
                        <input type="radio" name="source" checked={filterSource === 'All'} onChange={() => setFilterSource('All')} /> 全部 (All)
                    </label>
                    <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer' }}>
                        <input type="radio" name="source" checked={filterSource === 'Telegram'} onChange={() => setFilterSource('Telegram')} /> 仅 Telegram
                    </label>
                    <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer' }}>
                        <input type="radio" name="source" checked={filterSource === 'Slack'} onChange={() => setFilterSource('Slack')} /> 仅 Slack
                    </label>
                    <label style={{ display: 'flex', alignItems: 'center', gap: '8px', cursor: 'pointer' }}>
                        <input type="radio" name="source" checked={filterSource === 'Web'} onChange={() => setFilterSource('Web')} /> 仅 Web 客户端
                    </label>
                </div>
            </Modal>

            <Modal
                isOpen={!!detailSession}
                onClose={() => setDetailSession(null)}
                title="会话详细信息 (Raw Payload)"
                width="600px"
            >
                {detailSession && (
                    <pre style={{ 
                        background: 'rgba(0,0,0,0.3)', 
                        padding: '1rem', 
                        borderRadius: '8px', 
                        fontFamily: 'monospace',
                        color: 'var(--secondary-color)',
                        whiteSpace: 'pre-wrap',
                        wordBreak: 'break-all'
                    }}>
                        {JSON.stringify(detailSession, null, 2)}
                    </pre>
                )}
            </Modal>
        </div>
    );
}
