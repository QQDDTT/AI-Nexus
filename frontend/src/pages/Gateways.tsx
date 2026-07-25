import Header from '../components/Header';
import Modal from '../components/Modal';
import { TelegramLogo, SlackLogo, Browser, GearSix, Power, Trash, ChatTeardropText, ChatCircleText, GameController, WechatLogo, WhatsappLogo } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES, GATEWAY_PLATFORMS } from '../constants';

interface GatewayStatus {
    id: string;
    status: string;
    requests_24h: number;
    latency_ms: number;
    bound_persona?: string;
    platform?: string;
    credentials?: Record<string, string>;
}

export default function Gateways() {
    const [gateways, setGateways] = useState<GatewayStatus[]>([]);
    const [personas, setPersonas] = useState<{id: string, name: string}[]>([]);

    useEffect(() => {
        fetchApi(API_ROUTES.GATEWAYS)
            .then(res => res.json())
            .then(data => setGateways(data))
            .catch(err => console.error(err));
            
        fetchApi(API_ROUTES.PERSONAS)
            .then(res => res.json())
            .then(data => setPersonas(Array.isArray(data) ? data : []))
            .catch(err => console.error(err));
    }, []);

    const [isAddModalOpen, setIsAddModalOpen] = useState(false);
    const [newGatewayId, setNewGatewayId] = useState('');
    const [gatewayType, setGatewayType] = useState('Telegram');
    const [boundPersona, setBoundPersona] = useState('');
    const [fieldValues, setFieldValues] = useState<Record<string, string>>({});
    
    const [configModalGw, setConfigModalGw] = useState<string | null>(null);
    const [configKey, setConfigKey] = useState('');
    const [configPersona, setConfigPersona] = useState('');

    const currentPlatform = GATEWAY_PLATFORMS.find(p => p.id === gatewayType) || GATEWAY_PLATFORMS[0];

    const toggleGateway = (id: string) => {
        fetchApi(API_ROUTES.GATEWAY_TOGGLE(id), { method: 'POST' })
            .then(res => {
                if (res.ok) {
                    const updated = gateways.map(g => 
                        g.id === id ? { ...g, status: g.status === 'Active' ? 'Idle' : 'Active' } : g
                    );
                    setGateways(updated);
                }
            });
    };

    const deleteGateway = (id: string) => {
        if (!window.confirm(`确定要删除网关 ${id} 吗？`)) return;
        fetchApi(API_ROUTES.GATEWAY_BY_ID(id), { method: 'DELETE' })
            .then(res => {
                if (res.ok) {
                    setGateways(gateways.filter(g => g.id !== id));
                } else {
                    alert('删除网关失败');
                }
            });
    };

    const handleAddGateway = () => {
        if (!newGatewayId.trim()) return;
        const formattedId = gatewayType === 'Web' ? newGatewayId : `${gatewayType}: ${newGatewayId}`;
        const newGw: GatewayStatus = {
            id: formattedId,
            status: 'Idle',
            requests_24h: 0,
            latency_ms: 0,
            bound_persona: boundPersona || undefined,
            platform: gatewayType,
            credentials: fieldValues
        };
        fetchApi(API_ROUTES.GATEWAYS, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(newGw)
        }).then(res => {
            if (res.ok) {
                setGateways([...gateways, newGw]);
                setIsAddModalOpen(false);
                setNewGatewayId('');
                setBoundPersona('');
                setFieldValues({});
            } else {
                alert('Failed to add gateway');
            }
        });
    };

    const handleConfigGateway = () => {
        if (!configModalGw) return;
        fetchApi(API_ROUTES.GATEWAY_CONFIG(configModalGw), {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ key: configKey, bound_persona: configPersona || undefined })
        }).then(res => {
            if (res.ok) {
                const updated = gateways.map(g => 
                    g.id === configModalGw ? { ...g, bound_persona: configPersona || undefined } : g
                );
                setGateways(updated);
                setConfigModalGw(null);
                setConfigKey('');
                setConfigPersona('');
                alert('Configuration saved');
            } else {
                alert('Failed to save configuration');
            }
        });
    };

    const getPlatformIcon = (platformId: string, color: string) => {
        switch (platformId) {
            case 'Telegram': return <TelegramLogo size={32} color={color} weight="duotone" />;
            case 'Lark': return <ChatTeardropText size={32} color={color} weight="duotone" />;
            case 'LINE': return <ChatCircleText size={32} color={color} weight="duotone" />;
            case 'Slack': return <SlackLogo size={32} color={color} weight="duotone" />;
            case 'Discord': return <GameController size={32} color={color} weight="duotone" />;
            case 'WeChat': return <WechatLogo size={32} color={color} weight="duotone" />;
            case 'WhatsApp': return <WhatsappLogo size={32} color={color} weight="duotone" />;
            default: return <Browser size={32} color={color} weight="duotone" />;
        }
    };

    return (
        <div className="main-content">
            <Header 
                title="渠道网关 Gateways" 
                description="管理各个外部即时通讯平台（Telegram、飞书/Lark、LINE、Slack、Discord、微信等）与 Web 客户端的接入点。" 
            />

            <div style={{ display: 'flex', justifyContent: 'flex-end', marginBottom: '1.5rem' }}>
                <button className="btn-outline" onClick={() => setIsAddModalOpen(true)}>
                    + 新增渠道网关
                </button>
            </div>

            <section className="dashboard-grid" style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(340px, 1fr))', gap: '1.5rem' }}>
                {gateways.map((gw, i) => {
                    const platformId = gw.platform || (gw.id.includes(':') ? gw.id.split(':')[0].trim() : 'Web');
                    const platformInfo = GATEWAY_PLATFORMS.find(p => p.id === platformId) || GATEWAY_PLATFORMS[GATEWAY_PLATFORMS.length - 1];
                    const color = platformInfo.color;
                    const isActive = gw.status === 'Active';

                    return (
                        <div key={i} className="panel" style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                                <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                                    <div style={{ padding: '0.75rem', background: `${color}15`, borderRadius: '12px' }}>
                                        {getPlatformIcon(platformInfo.id, color)}
                                    </div>
                                    <div>
                                        <h3 style={{ fontSize: '1.1rem', marginBottom: '0.25rem' }}>{platformInfo.name}</h3>
                                        <p style={{ color: 'var(--text-secondary)', fontSize: '0.85rem' }}>{gw.id.includes(':') ? gw.id.split(':')[1].trim() : gw.id}</p>
                                    </div>
                                </div>
                                <span className={`status-badge status-${isActive ? 'active' : 'idle'}`}>{isActive ? 'Running' : 'Stopped'}</span>
                            </div>
                            <div style={{ background: 'rgba(255,255,255,0.02)', padding: '1rem', borderRadius: '8px', border: '1px solid var(--surface-border)' }}>
                                <div style={{ display: 'flex', justifyContent: 'space-between', marginBottom: '0.5rem', fontSize: '0.85rem' }}>
                                    <span style={{ color: 'var(--text-secondary)' }}>请求数 (24h)</span>
                                    <span style={{ fontWeight: 600 }}>{(gw.requests_24h || 0).toLocaleString()}</span>
                                </div>
                                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.85rem' }}>
                                    <span style={{ color: 'var(--text-secondary)' }}>平均延迟</span>
                                    <span style={{ fontWeight: 600, color: isActive ? 'var(--secondary-color)' : 'var(--text-secondary)' }}>{isActive ? `${gw.latency_ms || 0}ms` : '-'}</span>
                                </div>
                                <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: '0.85rem', marginTop: '0.5rem' }}>
                                    <span style={{ color: 'var(--text-secondary)' }}>绑定的 Persona</span>
                                    <span style={{ fontWeight: 600, color: gw.bound_persona ? 'var(--primary-color)' : 'var(--text-secondary)' }}>
                                        {gw.bound_persona ? personas.find(p => p.id === gw.bound_persona)?.name || gw.bound_persona : '未绑定'}
                                    </span>
                                </div>
                            </div>
                            <div style={{ display: 'flex', gap: '0.5rem' }}>
                                <button className="btn-outline" style={{ flex: 1, display: 'flex', justifyContent: 'center', alignItems: 'center', gap: '8px' }} onClick={() => {
                                    setConfigModalGw(gw.id);
                                    setConfigPersona(gw.bound_persona || '');
                                }}><GearSix /> 配置</button>
                                <button className={`action-btn ${isActive ? 'danger' : ''}`} style={{ margin: 0 }} onClick={() => toggleGateway(gw.id)}><Power size={20} /></button>
                                <button className="action-btn danger" style={{ margin: 0, padding: '0 0.5rem', background: 'rgba(239, 68, 68, 0.1)', color: 'var(--danger-color)' }} onClick={() => deleteGateway(gw.id)}><Trash size={20} /></button>
                            </div>
                        </div>
                    );
                })}
            </section>

            <Modal 
                isOpen={isAddModalOpen} 
                onClose={() => setIsAddModalOpen(false)}
                title="新增渠道网关"
                footer={<>
                    <button className="btn-outline" onClick={() => setIsAddModalOpen(false)}>取消</button>
                    <button className="btn-primary" onClick={handleAddGateway}>确认添加</button>
                </>}
            >
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        <label style={{ color: 'var(--text-secondary)' }}>聊天平台 (Channel Platform)</label>
                        <select 
                            value={gatewayType}
                            onChange={e => {
                                setGatewayType(e.target.value);
                                setFieldValues({});
                            }}
                            className="custom-select"
                        >
                            {GATEWAY_PLATFORMS.map(p => (
                                <option key={p.id} value={p.id}>{p.name} - {p.description}</option>
                            ))}
                        </select>
                    </div>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        <label style={{ color: 'var(--text-secondary)' }}>网关标识/名称 (Gateway Identifier)</label>
                        <input 
                            type="text" 
                            value={newGatewayId}
                            onChange={e => setNewGatewayId(e.target.value)}
                            placeholder="如: SupportBot_01, Lark_HR_Assistant"
                            style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }}
                        />
                    </div>
                    
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        <label style={{ color: 'var(--text-secondary)' }}>绑定 Persona (可选)</label>
                        <select
                            value={boundPersona}
                            onChange={e => setBoundPersona(e.target.value)}
                            className="custom-select"
                        >
                            <option value="">-- 不绑定 (默认) --</option>
                            {personas.map(p => (
                                <option key={p.id} value={p.id}>{p.name}</option>
                            ))}
                        </select>
                    </div>

                    {/* 动态平台凭证配置区 */}
                    {currentPlatform.fields.map(field => (
                        <div key={field.key} style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                            <label style={{ color: 'var(--text-secondary)' }}>{field.label}</label>
                            <input 
                                type={field.type || 'text'} 
                                value={fieldValues[field.key] || ''}
                                onChange={e => setFieldValues({ ...fieldValues, [field.key]: e.target.value })}
                                placeholder={field.placeholder}
                                style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }}
                            />
                        </div>
                    ))}
                </div>
            </Modal>

            <Modal
                isOpen={!!configModalGw}
                onClose={() => setConfigModalGw(null)}
                title={`配置网关: ${configModalGw}`}
                footer={<>
                    <button className="btn-outline" onClick={() => setConfigModalGw(null)}>取消</button>
                    <button className="btn-primary" onClick={handleConfigGateway}>保存配置</button>
                </>}
            >
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        <label style={{ color: 'var(--text-secondary)' }}>API Key / Token / Connection Secret</label>
                        <input 
                            type="password" 
                            value={configKey}
                            onChange={e => setConfigKey(e.target.value)}
                            placeholder="保留为空则不修改"
                            style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }}
                        />
                    </div>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        <label style={{ color: 'var(--text-secondary)' }}>绑定的 Persona</label>
                        <select
                            value={configPersona}
                            onChange={e => setConfigPersona(e.target.value)}
                            className="custom-select"
                        >
                            <option value="">-- 不绑定 (默认) --</option>
                            {personas.map(p => (
                                <option key={p.id} value={p.id}>{p.name}</option>
                            ))}
                        </select>
                    </div>
                </div>
            </Modal>
        </div>
    );
}
