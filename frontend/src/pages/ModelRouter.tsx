import Header from '../components/Header';
import Modal from '../components/Modal';
import { Cpu, Lightning, Scales, Eye, Network, Key, Trash, Plus } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES, PROVIDER_PRESETS } from '../constants';

interface TierConfig {
    primary: string;
    failover: string[];
}

type CapabilityRouting = Record<string, TierConfig>;

interface Provider {
    id: string;
    name: string;
    api_key: string;
    base_url?: string;
}

const TIER_METADATA: Record<string, { icon: React.ReactNode, name: string, desc: string }> = {
    "Tier-1-Logic": { icon: <Cpu size={24} weight="duotone" color="var(--primary-color)" />, name: "Tier-1-Logic", desc: "最高逻辑推理与代码生成能力" },
    "Tier-2-Balanced": { icon: <Scales size={24} weight="duotone" color="var(--accent-color)" />, name: "Tier-2-Balanced", desc: "均衡成本与性能，适用于常规任务" },
    "Tier-3-Fast": { icon: <Lightning size={24} weight="duotone" color="var(--secondary-color)" />, name: "Tier-3-Fast", desc: "极速响应，适用于闲聊或简单任务" },
    "Multimodal-Vision": { icon: <Eye size={24} weight="duotone" color="#8b5cf6" />, name: "Multimodal-Vision", desc: "多模态视觉处理与分析" },
};

export default function ModelRouter() {
    const [strategy, setStrategy] = useState<CapabilityRouting | null>(null);
    const [providers, setProviders] = useState<Provider[]>([]);
    
    // Add Provider Modal State
    const [isAddProviderModalOpen, setIsAddProviderModalOpen] = useState(false);
    const [selectedPreset, setSelectedPreset] = useState('');
    const [newProviderId, setNewProviderId] = useState('');
    const [newProviderName, setNewProviderName] = useState('');
    const [newProviderApiKey, setNewProviderApiKey] = useState('');
    const [newProviderBaseUrl, setNewProviderBaseUrl] = useState('');

    // Add Strategy Modal State
    const [isAddStrategyModalOpen, setIsAddStrategyModalOpen] = useState(false);
    const [newTierName, setNewTierName] = useState('');
    const [newTierPrimary, setNewTierPrimary] = useState('');
    const [newTierFailover, setNewTierFailover] = useState('');

    const fetchData = () => {
        fetchApi(API_ROUTES.MODELS_ROUTING)
            .then(res => res.json())
            .then(data => setStrategy(data))
            .catch(err => console.error(err));

        fetchApi(API_ROUTES.MODELS_PROVIDERS)
            .then(res => res.json())
            .then(data => setProviders(data))
            .catch(err => console.error(err));
    };

    const handleAddStrategy = () => {
        if (!newTierName.trim() || !newTierPrimary.trim() || !strategy) return;
        const failoverList = newTierFailover.split(',').map(s => s.trim()).filter(Boolean);
        const updated = {
            ...strategy,
            [newTierName.trim()]: {
                primary: newTierPrimary.trim(),
                failover: failoverList
            }
        };
        fetchApi(API_ROUTES.MODELS_ROUTING, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(updated)
        }).then(res => {
            if (res.ok) {
                setStrategy(updated);
                setIsAddStrategyModalOpen(false);
                setNewTierName('');
                setNewTierPrimary('');
                setNewTierFailover('');
            } else {
                alert('添加策略失败');
            }
        });
    };

    const handleDeleteStrategy = (tierKey: string) => {
        if (!strategy) return;
        if (!confirm(`确定删除能力策略 "${tierKey}" 吗？`)) return;
        const updated = { ...strategy };
        delete updated[tierKey];
        fetchApi(API_ROUTES.MODELS_ROUTING, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(updated)
        }).then(res => {
            if (res.ok) {
                setStrategy(updated);
            } else {
                alert('删除策略失败');
            }
        });
    };

    useEffect(() => {
        fetchData();
    }, []);

    const handlePresetChange = (presetId: string) => {
        setSelectedPreset(presetId);
        const preset = PROVIDER_PRESETS.find(p => p.id === presetId);
        if (preset && presetId !== 'custom') {
            setNewProviderId(preset.id);
            setNewProviderName(preset.name);
            setNewProviderBaseUrl(preset.baseUrl);
        }
    };

    const handleAddProvider = () => {
        if (!newProviderId || !newProviderName || !newProviderApiKey) return;
        fetchApi(API_ROUTES.MODELS_PROVIDERS, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ 
                id: newProviderId, 
                name: newProviderName, 
                api_key: newProviderApiKey,
                base_url: newProviderBaseUrl 
            })
        }).then(res => {
            if (res.ok) {
                fetchData();
                setIsAddProviderModalOpen(false);
                setSelectedPreset('');
                setNewProviderId('');
                setNewProviderName('');
                setNewProviderApiKey('');
                setNewProviderBaseUrl('');
            } else {
                alert('添加服务商失败');
            }
        });
    };

    const handleDeleteProvider = (id: string) => {
        if (confirm('确定删除该服务商凭证吗？')) {
            fetchApi(API_ROUTES.MODEL_PROVIDER_BY_ID(id), { method: 'DELETE' })
                .then(res => {
                    if (res.ok) {
                        fetchData();
                    } else {
                        alert('删除服务商失败');
                    }
                });
        }
    };

    return (
        <div className="main-content">
            <Header 
                title="算力中心 Model Router" 
                description="配置基于能力标签的动态模型路由策略，并统一管理各大模型服务商凭证。" 
            />

            {/* Providers Section */}
            <section style={{ marginBottom: '2.5rem' }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                    <h2 style={{ margin: 0, fontSize: '1.2rem', fontWeight: 600 }}>服务商凭证池 (Providers Registry)</h2>
                    <button className="primary-btn" onClick={() => setIsAddProviderModalOpen(true)} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', padding: '0.5rem 1rem' }}>
                        <Plus size={16} /> 新增服务商
                    </button>
                </div>
                
                <div style={{ display: 'flex', gap: '1rem', overflowX: 'auto', paddingBottom: '0.5rem' }}>
                    {providers.map(p => (
                        <div key={p.id} style={{ minWidth: '280px', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--surface-border)', padding: '1rem', borderRadius: '12px', display: 'flex', flexDirection: 'column' }}>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '1rem' }}>
                                <div>
                                    <div style={{ fontWeight: 600, fontSize: '1.1rem' }}>{p.name}</div>
                                    <div style={{ fontSize: '0.8rem', color: 'var(--text-secondary)' }}>ID: {p.id}</div>
                                </div>
                                <button className="action-btn danger" onClick={() => handleDeleteProvider(p.id)}>
                                    <Trash size={18} />
                                </button>
                            </div>
                            <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', background: 'rgba(0,0,0,0.3)', padding: '0.5rem 0.75rem', borderRadius: '6px', fontSize: '0.85rem' }}>
                                <Key size={16} color="var(--secondary-color)" />
                                <span style={{ color: 'var(--text-secondary)', fontFamily: 'monospace' }}>{p.api_key}</span>
                            </div>
                        </div>
                    ))}
                    {providers.length === 0 && (
                        <div style={{ padding: '2rem', textAlign: 'center', width: '100%', color: 'var(--text-secondary)', border: '1px dashed var(--surface-border)', borderRadius: '12px' }}>
                            暂未配置任何服务商凭证。请点击上方按钮添加。
                        </div>
                    )}
                </div>
            </section>

            {/* Routing Strategies Section */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                <h2 style={{ margin: 0, fontSize: '1.2rem', fontWeight: 600 }}>能力标签映射策略 (Capability Routing)</h2>
                <button className="primary-btn" onClick={() => setIsAddStrategyModalOpen(true)} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', padding: '0.5rem 1rem' }}>
                    <Plus size={16} /> 新增映射策略
                </button>
            </div>

            <section style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(400px, 1fr))', gap: '1.5rem' }}>
                {strategy ? Object.entries(strategy).map(([tierKey, config]) => {
                    const meta = TIER_METADATA[tierKey] || { icon: <Network size={24} />, name: tierKey, desc: "自定义能力标签策略" };
                    return (
                        <div key={tierKey} className="panel" style={{ display: 'flex', flexDirection: 'column' }}>
                            <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                                <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                                    <div style={{ background: 'rgba(255,255,255,0.05)', padding: '0.75rem', borderRadius: '12px' }}>
                                        {meta.icon}
                                    </div>
                                    <div>
                                        <h3 style={{ margin: 0, fontSize: '1.2rem', fontWeight: 600 }}>{meta.name}</h3>
                                        <p style={{ margin: 0, fontSize: '0.85rem', color: 'var(--text-secondary)' }}>{meta.desc}</p>
                                    </div>
                                </div>
                                <button 
                                    className="action-btn danger" 
                                    onClick={() => handleDeleteStrategy(tierKey)}
                                    title="删除此映射策略"
                                    style={{ margin: 0 }}
                                >
                                    <Trash size={18} />
                                </button>
                            </div>
                            
                            <div style={{ marginTop: '1.5rem', flex: 1 }}>
                                <div style={{ marginBottom: '1rem' }}>
                                    <div style={{ fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>主路由 (Primary)</div>
                                    <div style={{ background: 'rgba(59, 130, 246, 0.1)', border: '1px solid rgba(59, 130, 246, 0.2)', padding: '1rem', borderRadius: '8px', display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                                        <div style={{ width: '8px', height: '8px', borderRadius: '50%', background: 'var(--primary-color)' }}></div>
                                        <span style={{ fontWeight: 500, color: 'var(--primary-color)' }}>{config.primary}</span>
                                    </div>
                                </div>

                                <div>
                                    <div style={{ fontSize: '0.85rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>备用节点 (Failover)</div>
                                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                                        {config.failover.map((f, idx) => (
                                            <div key={idx} style={{ background: 'rgba(255,255,255,0.02)', border: '1px solid var(--surface-border)', padding: '0.75rem 1rem', borderRadius: '8px', display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                                                <div style={{ width: '6px', height: '6px', borderRadius: '50%', background: 'var(--text-secondary)' }}></div>
                                                <span style={{ color: 'var(--text-secondary)' }}>{f}</span>
                                                <span style={{ marginLeft: 'auto', fontSize: '0.75rem', padding: '0.2rem 0.5rem', background: 'rgba(0,0,0,0.3)', borderRadius: '4px' }}>优先级 {idx + 1}</span>
                                            </div>
                                        ))}
                                        {config.failover.length === 0 && (
                                            <div style={{ color: 'var(--text-secondary)', fontStyle: 'italic', fontSize: '0.9rem' }}>无备用节点</div>
                                        )}
                                    </div>
                                </div>
                            </div>
                        </div>
                    );
                }) : <p>Loading Routing Configurations...</p>}
            </section>

            {/* Add Provider Modal */}
            <Modal
                isOpen={isAddProviderModalOpen}
                onClose={() => setIsAddProviderModalOpen(false)}
                title="添加新服务商凭证"
                footer={<>
                    <button className="btn-outline" onClick={() => setIsAddProviderModalOpen(false)}>取消</button>
                    <button className="btn-primary" onClick={handleAddProvider}>保存</button>
                </>}
            >
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>选择服务商预设 (下拉菜单)</label>
                        <select 
                            value={selectedPreset}
                            onChange={e => handlePresetChange(e.target.value)}
                            className="custom-select"
                        >
                            <option value="">-- 请选择服务商模板 --</option>
                            {PROVIDER_PRESETS.map(p => (
                                <option key={p.id} value={p.id}>{p.name} ({p.id})</option>
                            ))}
                        </select>
                    </div>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>提供商 ID (如 openai)</label>
                        <input 
                            type="text" 
                            value={newProviderId}
                            onChange={e => setNewProviderId(e.target.value)}
                            className="custom-input"
                            placeholder="如 openai, deepseek"
                        />
                    </div>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>显示名称 (如 OpenAI GPT-4)</label>
                        <input 
                            type="text" 
                            value={newProviderName}
                            onChange={e => setNewProviderName(e.target.value)}
                            className="custom-input"
                            placeholder="如 OpenAI GPT-4"
                        />
                    </div>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>API Key</label>
                        <input 
                            type="password" 
                            value={newProviderApiKey}
                            onChange={e => setNewProviderApiKey(e.target.value)}
                            className="custom-input"
                            placeholder="sk-..."
                        />
                    </div>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>Base URL (可选, 私有部署使用)</label>
                        <input 
                            type="text" 
                            value={newProviderBaseUrl}
                            onChange={e => setNewProviderBaseUrl(e.target.value)}
                            className="custom-input"
                            placeholder="https://api.openai.com/v1"
                        />
                    </div>
                </div>
            </Modal>

            {/* Add Strategy Modal */}
            <Modal
                isOpen={isAddStrategyModalOpen}
                onClose={() => setIsAddStrategyModalOpen(false)}
                title="新增能力标签映射策略"
                footer={<>
                    <button className="btn-outline" onClick={() => setIsAddStrategyModalOpen(false)}>取消</button>
                    <button className="btn-primary" onClick={handleAddStrategy}>保存创建</button>
                </>}
            >
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>策略名称 (Tier Key)</label>
                        <input 
                            type="text" 
                            value={newTierName}
                            onChange={e => setNewTierName(e.target.value)}
                            className="custom-input"
                            placeholder="如: Tier-4-Code 或 Custom-Fast"
                        />
                    </div>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>主路由模型 (Primary Model)</label>
                        <input 
                            type="text" 
                            value={newTierPrimary}
                            onChange={e => setNewTierPrimary(e.target.value)}
                            className="custom-input"
                            placeholder="如: gemini-2.5-pro, gpt-4o"
                        />
                    </div>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>备用节点模型 (Failover Models, 逗号分隔)</label>
                        <input 
                            type="text" 
                            value={newTierFailover}
                            onChange={e => setNewTierFailover(e.target.value)}
                            className="custom-input"
                            placeholder="如: claude-3-5-sonnet, deepseek-v3"
                        />
                    </div>
                </div>
            </Modal>
        </div>
    );
}
