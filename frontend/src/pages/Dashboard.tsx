import Header from '../components/Header';
import { useEffect, useState, useRef } from 'react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES } from '../constants';
import { Users, Coin, Heartbeat, TrendUp, DotsThreeOutlineVertical, TelegramLogo, SlackLogo, Browser, Export, ArrowsClockwise, Robot, Wrench } from '@phosphor-icons/react';
import { AreaChart, Area, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import './Dashboard.css';

interface AgentStatus {
    id: string;
    status?: string;
    state?: string;
    uptime: string;
    tasks_completed: number;
}

interface SkillUsage {
    skill: string;
    calls: number;
    success_rate: number;
}

interface GatewayStatus {
    id: string;
    status: string;
}

interface DashboardStats {
    active_sessions: number;
    active_sessions_trend: string;
    total_tokens: string;
    total_tokens_trend: string;
    api_health: string;
    api_health_trend: string;
    gateways: GatewayStatus[];
    agents?: AgentStatus[];
    skills_usage?: SkillUsage[];
}

export default function Dashboard() {
    const [stats, setStats] = useState<DashboardStats | null>(null);
    const [tokenTrend, setTokenTrend] = useState<any[] | null>(null);
    
    const [isMoreMenuOpen, setIsMoreMenuOpen] = useState(false);
    const moreMenuRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (moreMenuRef.current && !moreMenuRef.current.contains(event.target as Node)) {
                setIsMoreMenuOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    useEffect(() => {
        fetchApi(API_ROUTES.DASHBOARD_STATS)
            .then(res => res.json())
            .then(data => setStats(data))
            .catch(err => console.error('Failed to fetch stats:', err));

        fetchApi(API_ROUTES.DASHBOARD_TOKEN_TREND)
            .then(res => res.json())
            .then(data => setTokenTrend(data.trend || []))
            .catch(err => {
                console.error('Failed to fetch token trend:', err);
                setTokenTrend([]);
            });
    }, []);

    return (
        <div className="main-content dashboard-container">
            <Header 
                title="平台概览 Overview" 
                description="实时监控 AI-Nexus 多智能体协调网络的状态。" 
            />

            <section className="kpi-grid">
                <div className="kpi-card blue">
                    <div className="kpi-header">
                        <span>当前活跃会话 Active Sessions</span>
                        <div className="kpi-icon">
                            <Users size={24} color="var(--primary-color)" />
                        </div>
                    </div>
                    <div className="kpi-value">{stats ? stats.active_sessions.toLocaleString() : '...'}</div>
                    <div className="kpi-trend trend-up">
                        <TrendUp weight="bold" /> {stats ? stats.active_sessions_trend : '...'}
                    </div>
                </div>

                <div className="kpi-card purple">
                    <div className="kpi-header">
                        <span>今日消耗 Total Tokens</span>
                        <div className="kpi-icon">
                            <Coin size={24} color="var(--accent-color)" />
                        </div>
                    </div>
                    <div className="kpi-value">{stats ? stats.total_tokens : '...'}</div>
                    <div className="kpi-trend trend-up">
                        <TrendUp weight="bold" /> {stats ? stats.total_tokens_trend : '...'}
                    </div>
                </div>

                <div className="kpi-card green">
                    <div className="kpi-header">
                        <span>系统心跳 API Health</span>
                        <div className="kpi-icon">
                            <Heartbeat size={24} color="var(--secondary-color)" />
                        </div>
                    </div>
                    <div className="kpi-value">{stats ? stats.api_health : '...'}</div>
                    <div className="kpi-trend trend-up">
                        <Heartbeat weight="bold" /> {stats ? stats.api_health_trend : '...'}
                    </div>
                </div>
            </section>

            <section className="dashboard-grid">
                {/* 趋势图表区 */}
                <div className="panel" style={{ gridColumn: '1 / -1' }}>
                    <div className="panel-header" ref={moreMenuRef}>
                        <span className="panel-title">Token 消耗趋势 (近 7 天)</span>
                        <DotsThreeOutlineVertical size={20} color="var(--text-secondary)" style={{ cursor: 'pointer' }} onClick={() => setIsMoreMenuOpen(!isMoreMenuOpen)} />
                        
                        {isMoreMenuOpen && (
                            <div className="more-menu-dropdown">
                                <div className="dropdown-item" onClick={() => { alert('已刷新最新数据'); setIsMoreMenuOpen(false); }}>
                                    <ArrowsClockwise size={16} /> 刷新数据
                                </div>
                                <div className="dropdown-item" onClick={() => { 
                                    const trendList = tokenTrend || [];
                                    const csvData = [
                                        ['Date', 'Tokens Used'],
                                        ...trendList.map((t) => [`${t.name}`, t.tokens.toString()])
                                    ].map(e => e.join(",")).join("\n");
                                    const blob = new Blob([csvData], { type: 'text/csv' });
                                    const url = window.URL.createObjectURL(blob);
                                    const a = document.createElement('a');
                                    a.setAttribute('href', url);
                                    a.setAttribute('download', `ai-nexus-report-${new Date().toISOString().split('T')[0]}.csv`);
                                    a.click();
                                    setIsMoreMenuOpen(false); 
                                }}>
                                    <Export size={16} /> 导出报表
                                </div>
                            </div>
                        )}
                    </div>
                    <div style={{ width: '100%', height: '300px' }}>
                        {tokenTrend === null ? (
                            <div style={{ color: 'var(--text-secondary)', display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>加载中 Loading...</div>
                        ) : tokenTrend.length > 0 ? (
                            <ResponsiveContainer width="100%" height="100%">
                                <AreaChart data={tokenTrend} margin={{ top: 10, right: 30, left: 0, bottom: 0 }}>
                                    <defs>
                                        <linearGradient id="colorTokens" x1="0" y1="0" x2="0" y2="1">
                                            <stop offset="5%" stopColor="var(--primary-color)" stopOpacity={0.8}/>
                                            <stop offset="95%" stopColor="var(--primary-color)" stopOpacity={0}/>
                                        </linearGradient>
                                    </defs>
                                    <XAxis dataKey="name" stroke="var(--text-secondary)" tick={{fill: 'var(--text-secondary)'}} />
                                    <YAxis stroke="var(--text-secondary)" tick={{fill: 'var(--text-secondary)'}} />
                                    <CartesianGrid strokeDasharray="3 3" stroke="var(--surface-border)" vertical={false} />
                                    <Tooltip 
                                        contentStyle={{ backgroundColor: 'var(--surface-color)', borderColor: 'var(--surface-border)', borderRadius: '8px' }}
                                        itemStyle={{ color: 'var(--text-primary)' }}
                                    />
                                    <Area type="monotone" dataKey="tokens" stroke="var(--primary-color)" fillOpacity={1} fill="url(#colorTokens)" />
                                </AreaChart>
                            </ResponsiveContainer>
                        ) : (
                            <div style={{ color: 'var(--text-secondary)', display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%' }}>暂无记录数据 (No data)</div>
                        )}
                    </div>
                </div>

                {/* Agent 状态区 */}
                <div className="panel">
                    <div className="panel-header">
                        <span className="panel-title">Agent 实例状态</span>
                    </div>
                    <div className="agent-list">
                        {stats?.agents ? (
                            stats.agents.length > 0 ? (
                                stats.agents.map((agent, i) => {
                                    const st = agent.status || agent.state || 'Active';
                                    return (
                                        <div key={i} className="agent-item">
                                            <div className="agent-info">
                                                <span className="agent-name"><Robot size={16} style={{marginRight: '4px', verticalAlign: 'text-bottom'}}/>{agent.id}</span>
                                                <span className="agent-meta">Uptime: {agent.uptime} | Tasks: {agent.tasks_completed}</span>
                                            </div>
                                            <div>
                                                <span className={`status-badge status-${st.toLowerCase()}`}>{st}</span>
                                            </div>
                                        </div>
                                    );
                                })
                            ) : (
                                <div style={{ color: 'var(--text-secondary)' }}>暂无实例</div>
                            )
                        ) : (
                            <div style={{ color: 'var(--text-secondary)' }}>Loading...</div>
                        )}
                    </div>
                </div>

                {/* 技能热度排行榜 */}
                <div className="panel">
                    <div className="panel-header">
                        <span className="panel-title">Skill 调用热度 (Top 5)</span>
                    </div>
                    <div className="skill-list">
                        {stats?.skills_usage ? (
                            stats.skills_usage.length > 0 ? (
                                stats.skills_usage.map((skill, i) => (
                                    <div key={i} className="skill-item">
                                        <span className="skill-name"><Wrench size={16} style={{marginRight: '4px', verticalAlign: 'text-bottom'}}/>{skill.skill}</span>
                                        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'flex-end', gap: '2px' }}>
                                            <span style={{ fontSize: '0.9rem', color: 'var(--text-primary)', fontWeight: 500 }}>{skill.calls} 次</span>
                                            <span style={{ fontSize: '0.75rem', color: skill.success_rate > 90 ? 'var(--secondary-color)' : 'var(--accent-color)' }}>
                                                成功率 {skill.success_rate}%
                                            </span>
                                        </div>
                                    </div>
                                ))
                            ) : (
                                <div style={{ color: 'var(--text-secondary)' }}>暂无技能调用数据</div>
                            )
                        ) : (
                            <div style={{ color: 'var(--text-secondary)' }}>Loading...</div>
                        )}
                    </div>
                </div>

                {/* 网关实时节点 */}
                <div className="panel">
                    <div className="panel-header">
                        <span className="panel-title">网关实时节点</span>
                    </div>
                    <table className="data-table">
                        <thead>
                            <tr>
                                <th>Channel ID</th>
                                <th>Status</th>
                            </tr>
                        </thead>
                        <tbody>
                            {stats ? (
                                stats.gateways.length > 0 ? (
                                    stats.gateways.map((gw, i) => (
                                        <tr key={i}>
                                            <td className="status-cell">
                                                {gw.id.includes('Telegram') ? <TelegramLogo size={20} color="#3b82f6" /> : 
                                                 gw.id.includes('Slack') ? <SlackLogo size={20} color="#e11d48" /> : 
                                                 <Browser size={20} color="#10b981" />} 
                                                {gw.id}
                                            </td>
                                            <td><span className={`status-badge status-${gw.status.toLowerCase()}`}>{gw.status}</span></td>
                                        </tr>
                                    ))
                                ) : (
                                    <tr><td colSpan={2} style={{ textAlign: 'center', color: 'var(--text-secondary)' }}>暂无接入网关</td></tr>
                                )
                            ) : (
                                <tr><td colSpan={2}>Loading...</td></tr>
                            )}
                        </tbody>
                    </table>
                </div>
            </section>
        </div>
    );
}
