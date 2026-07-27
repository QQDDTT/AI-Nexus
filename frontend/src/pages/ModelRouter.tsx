import Header from '../components/Header';
import Modal from '../components/Modal';
import { Cpu, Lightning, Scales, Eye, Network, Key, Trash, Plus, PencilSimple, Code, BracketsCurly, WarningCircle, CheckSquare, Square } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES, PROVIDER_PRESETS } from '../constants';

interface RoutingRules {
    context_overflow_model?: string;
    max_token_threshold?: number;
    timeout_ms?: number;
}

interface CapabilityProfile {
    name?: string;
    description?: string;
    task_types?: string[];
    primary: string;
    failover: string[];
    routing_rules?: RoutingRules;
}

type CapabilityRouting = Record<string, CapabilityProfile>;

interface Provider {
    id: string;
    name: string;
    api_key: string;
    base_url?: string;
}

const PRESET_TASK_TYPES = [
    { id: 'Tier-1-Logic', label: '深度推理 (Tier-1-Logic)', color: '#3b82f6' },
    { id: 'Code-Generation', label: '代码生成 (Code-Generation)', color: '#10b981' },
    { id: 'Tier-2-Balanced', label: '通用平衡 (Tier-2-Balanced)', color: '#f59e0b' },
    { id: 'Tier-3-Fast', label: '极速响应 (Tier-3-Fast)', color: '#ec4899' },
    { id: 'Multimodal-Vision', label: '多模态 (Multimodal-Vision)', color: '#8b5cf6' },
    { id: 'Structured-Output', label: '结构化输出 (Structured-Output)', color: '#06b6d4' },
];

const TIER_METADATA: Record<string, { icon: React.ReactNode, name: string, desc: string }> = {
    "High-Reasoning-Profile": { icon: <Cpu size={24} weight="duotone" color="var(--primary-color)" />, name: "深度智力与代码算力组", desc: "适用于高逻辑、复杂代码分析与多步骤推理任务" },
    "General-Balanced-Profile": { icon: <Scales size={24} weight="duotone" color="var(--accent-color)" />, name: "通用对话与极速响应组", desc: "适用于日常对话、低延迟回复与轻量结构化提取" },
    "Multimodal-Vision-Profile": { icon: <Eye size={24} weight="duotone" color="#8b5cf6" />, name: "多模态视觉处理组", desc: "视觉分析、图像理解与多模态生成" },
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

    // Strategy Modal Form State
    const [isStrategyModalOpen, setIsStrategyModalOpen] = useState(false);
    const [editingProfileKey, setEditingProfileKey] = useState<string | null>(null); // null = Add, string = Edit
    const [profileKeyInput, setProfileKeyInput] = useState('');
    const [profileNameInput, setProfileNameInput] = useState('');
    const [profileDescInput, setProfileDescInput] = useState('');
    const [selectedTaskTypes, setSelectedTaskTypes] = useState<string[]>([]);
    const [primaryModelInput, setPrimaryModelInput] = useState('');
    const [failoverModelsInput, setFailoverModelsInput] = useState('');
    const [overflowModelInput, setOverflowModelInput] = useState('');
    const [tokenThresholdInput, setTokenThresholdInput] = useState<string>('32768');
    const [timeoutMsInput, setTimeoutMsInput] = useState<string>('10000');

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

    const handleOpenAddStrategy = () => {
        setEditingProfileKey(null);
        setProfileKeyInput('');
        setProfileNameInput('');
        setProfileDescInput('');
        setSelectedTaskTypes(['Tier-1-Logic']);
        setPrimaryModelInput('gemini-2.5-pro');
        setFailoverModelsInput('gpt-4o, claude-3-5-sonnet');
        setOverflowModelInput('gemini-1.5-pro');
        setTokenThresholdInput('32768');
        setTimeoutMsInput('10000');
        setIsStrategyModalOpen(true);
    };

    const handleOpenEditStrategy = (profileKey: string, config: CapabilityProfile) => {
        setEditingProfileKey(profileKey);
        setProfileKeyInput(profileKey);
        setProfileNameInput(config.name || profileKey);
        setProfileDescInput(config.description || '');
        setSelectedTaskTypes(config.task_types || [profileKey]);
        setPrimaryModelInput(config.primary || '');
        setFailoverModelsInput((config.failover || []).join(', '));
        setOverflowModelInput(config.routing_rules?.context_overflow_model || '');
        setTokenThresholdInput(config.routing_rules?.max_token_threshold?.toString() || '32768');
        setTimeoutMsInput(config.routing_rules?.timeout_ms?.toString() || '10000');
        setIsStrategyModalOpen(true);
    };

    const handleSaveStrategy = () => {
        const key = profileKeyInput.trim();
        if (!key || !primaryModelInput.trim() || !strategy) return;

        const failoverList = failoverModelsInput.split(',').map(s => s.trim()).filter(Boolean);
        const threshold = parseInt(tokenThresholdInput, 10);
        const timeout = parseInt(timeoutMsInput, 10);

        const newProfile: CapabilityProfile = {
            name: profileNameInput.trim() || key,
            description: profileDescInput.trim() || undefined,
            task_types: selectedTaskTypes.length > 0 ? selectedTaskTypes : [key],
            primary: primaryModelInput.trim(),
            failover: failoverList,
            routing_rules: (overflowModelInput.trim() || !isNaN(threshold) || !isNaN(timeout)) ? {
                context_overflow_model: overflowModelInput.trim() || undefined,
                max_token_threshold: !isNaN(threshold) ? threshold : undefined,
                timeout_ms: !isNaN(timeout) ? timeout : undefined,
            } : undefined
        };

        const updated = { ...strategy };
        if (editingProfileKey && editingProfileKey !== key) {
            delete updated[editingProfileKey];
        }
        updated[key] = newProfile;

        fetchApi(API_ROUTES.MODELS_ROUTING, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(updated)
        }).then(res => {
            if (res.ok) {
                setStrategy(updated);
                setIsStrategyModalOpen(false);
            } else {
                alert('保存算力 Profile 策略失败');
            }
        });
    };

    const handleDeleteStrategy = (profileKey: string) => {
        if (!strategy) return;
        if (!confirm(`确定删除算力 Profile "${profileKey}" 吗？`)) return;
        const updated = { ...strategy };
        delete updated[profileKey];
        fetchApi(API_ROUTES.MODELS_ROUTING, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(updated)
        }).then(res => {
            if (res.ok) {
                setStrategy(updated);
            } else {
                alert('删除算力 Profile 失败');
            }
        });
    };

    const toggleTaskTypeSelection = (taskTypeId: string) => {
        if (selectedTaskTypes.includes(taskTypeId)) {
            setSelectedTaskTypes(selectedTaskTypes.filter(t => t !== taskTypeId));
        } else {
            setSelectedTaskTypes([...selectedTaskTypes, taskTypeId]);
        }
    };

    useEffect(() => {
        fetchData();
    }, []);

    const handlePresetChange = (presetId: string) => {
        setSelectedPreset(presetId);
        const preset = PROVIDER_PRESETS.find(p => p.id === presetId);
        if (preset) {
            setNewProviderId(preset.id);
            setNewProviderName(preset.name);
            setNewProviderBaseUrl(preset.base_url || '');
        }
    };

    const handleAddProvider = () => {
        if (!newProviderId.trim() || !newProviderName.trim() || !newProviderApiKey.trim()) {
            alert('请填写完整的服务商凭证信息');
            return;
        }
        const payload = {
            id: newProviderId.trim(),
            name: newProviderName.trim(),
            api_key: newProviderApiKey.trim(),
            base_url: newProviderBaseUrl.trim() || undefined
        };
        fetchApi(API_ROUTES.MODELS_PROVIDERS, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(payload)
        }).then(res => {
            if (res.ok) {
                fetchData();
                setIsAddProviderModalOpen(false);
                setNewProviderId('');
                setNewProviderName('');
                setNewProviderApiKey('');
                setNewProviderBaseUrl('');
                setSelectedPreset('');
            } else {
                alert('添加服务商凭证失败');
            }
        });
    };

    const handleDeleteProvider = (id: string) => {
        if (!confirm(`确定删除服务商凭证 "${id}" 吗？`)) return;
        fetchApi(`${API_ROUTES.MODELS_PROVIDERS}/${id}`, {
            method: 'DELETE'
        }).then(res => {
            if (res.ok) {
                setProviders(providers.filter(p => p.id !== id));
            } else {
                alert('删除服务商失败');
            }
        });
    };

    return (
        <div className="main-content">
            <Header 
                title="算力中心 Model Router" 
                description="基于能力 Profile 算力组进行动态多任务调度控制，支持关联多任务类型、备用容灾与长文本溢出分流。" 
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

            {/* Capability Profiles Section */}
            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '1rem' }}>
                <div>
                    <h2 style={{ margin: 0, fontSize: '1.2rem', fontWeight: 600 }}>算力 Profile 映射策略 (Capability Profiles)</h2>
                    <span style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>支持单个算力 Profile 关联绑定多个任务类型，按规则自动分配计算资源</span>
                </div>
                <button className="primary-btn" onClick={handleOpenAddStrategy} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', padding: '0.5rem 1rem' }}>
                    <Plus size={16} /> 新增算力 Profile
                </button>
            </div>

            <section style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(420px, 1fr))', gap: '1.5rem' }}>
                {strategy ? Object.entries(strategy).map(([profileKey, config]) => {
                    const meta = TIER_METADATA[profileKey] || {
                        icon: <Network size={24} color="var(--primary-color)" />,
                        name: config.name || profileKey,
                        desc: config.description || "自定义算力 Profile 策略"
                    };
                    const taskTypes = config.task_types && config.task_types.length > 0 ? config.task_types : [profileKey];
                    const failoverList = config.failover || [];
                    const rules = config.routing_rules;

                    return (
                        <div key={profileKey} className="panel" style={{ display: 'flex', flexDirection: 'column', position: 'relative' }}>
                            <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start' }}>
                                <div style={{ display: 'flex', gap: '1rem', alignItems: 'center' }}>
                                    <div style={{ background: 'rgba(255,255,255,0.05)', padding: '0.75rem', borderRadius: '12px' }}>
                                        {meta.icon}
                                    </div>
                                    <div>
                                        <h3 style={{ margin: 0, fontSize: '1.15rem', fontWeight: 600 }}>{config.name || meta.name}</h3>
                                        <p style={{ margin: 0, fontSize: '0.8rem', color: 'var(--text-secondary)', marginTop: '0.2rem' }}>
                                            {config.description || meta.desc}
                                        </p>
                                    </div>
                                </div>
                                <div style={{ display: 'flex', gap: '0.5rem' }}>
                                    <button 
                                        className="action-btn" 
                                        onClick={() => handleOpenEditStrategy(profileKey, config)}
                                        title="编辑此算力 Profile"
                                        style={{ margin: 0 }}
                                    >
                                        <PencilSimple size={18} />
                                    </button>
                                    <button 
                                        className="action-btn danger" 
                                        onClick={() => handleDeleteStrategy(profileKey)}
                                        title="删除此 Profile"
                                        style={{ margin: 0 }}
                                    >
                                        <Trash size={18} />
                                    </button>
                                </div>
                            </div>
                            
                            {/* Associated Multi-Task Types Badges */}
                            <div style={{ marginTop: '1rem', display: 'flex', flexWrap: 'wrap', gap: '0.4rem', alignItems: 'center' }}>
                                <span style={{ fontSize: '0.75rem', color: 'var(--text-secondary)', marginRight: '0.2rem' }}>关联任务:</span>
                                {taskTypes.map(tt => {
                                    const preset = PRESET_TASK_TYPES.find(p => p.id === tt);
                                    return (
                                        <span 
                                            key={tt} 
                                            style={{ 
                                                fontSize: '0.75rem', 
                                                padding: '0.2rem 0.6rem', 
                                                borderRadius: '20px', 
                                                background: preset ? `${preset.color}15` : 'rgba(255,255,255,0.06)', 
                                                color: preset ? preset.color : 'var(--text-secondary)',
                                                border: `1px solid ${preset ? `${preset.color}40` : 'var(--surface-border)'}`,
                                                fontWeight: 500
                                            }}
                                        >
                                            {preset ? preset.label : tt}
                                        </span>
                                    );
                                })}
                            </div>

                            <div style={{ marginTop: '1.2rem', flex: 1, display: 'flex', flexDirection: 'column', gap: '0.8rem' }}>
                                <div>
                                    <div style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginBottom: '0.4rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>主路由模型 (Primary)</div>
                                    <div style={{ background: 'rgba(59, 130, 246, 0.08)', border: '1px solid rgba(59, 130, 246, 0.25)', padding: '0.75rem 1rem', borderRadius: '8px', display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                                        <div style={{ width: '8px', height: '8px', borderRadius: '50%', background: 'var(--primary-color)' }}></div>
                                        <span style={{ fontWeight: 600, color: 'var(--primary-color)', fontSize: '0.95rem' }}>{config.primary}</span>
                                    </div>
                                </div>

                                <div>
                                    <div style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginBottom: '0.4rem', textTransform: 'uppercase', letterSpacing: '0.05em' }}>备用节点链 (Failover Chain)</div>
                                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.4rem' }}>
                                        {failoverList.map((f, idx) => (
                                            <div key={idx} style={{ background: 'rgba(255,255,255,0.02)', border: '1px solid var(--surface-border)', padding: '0.5rem 0.75rem', borderRadius: '6px', display: 'flex', alignItems: 'center', gap: '0.5rem', fontSize: '0.85rem' }}>
                                                <div style={{ width: '5px', height: '5px', borderRadius: '50%', background: 'var(--text-secondary)' }}></div>
                                                <span style={{ color: 'var(--text-secondary)' }}>{f}</span>
                                                <span style={{ marginLeft: 'auto', fontSize: '0.7rem', padding: '0.15rem 0.4rem', background: 'rgba(0,0,0,0.3)', borderRadius: '4px', color: 'var(--text-secondary)' }}>备用 {idx + 1}</span>
                                            </div>
                                        ))}
                                        {failoverList.length === 0 && (
                                            <div style={{ color: 'var(--text-secondary)', fontStyle: 'italic', fontSize: '0.85rem' }}>无备用节点</div>
                                        )}
                                    </div>
                                </div>

                                {/* Advanced Routing Rules Banner */}
                                {rules && rules.context_overflow_model && (
                                    <div style={{ marginTop: '0.5rem', background: 'rgba(139, 92, 246, 0.08)', border: '1px dashed rgba(139, 92, 246, 0.3)', padding: '0.6rem 0.8rem', borderRadius: '8px', fontSize: '0.8rem', display: 'flex', alignItems: 'center', gap: '0.5rem', color: '#c084fc' }}>
                                        <WarningCircle size={16} color="#c084fc" />
                                        <span>文本 &gt; {rules.max_token_threshold || 32768} tokens 自动分流重定向至 <strong>{rules.context_overflow_model}</strong></span>
                                    </div>
                                )}
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
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>选择服务商预设</label>
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
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>提供商 ID</label>
                        <input 
                            type="text" 
                            value={newProviderId}
                            onChange={e => setNewProviderId(e.target.value)}
                            className="custom-input"
                            placeholder="如 openai, deepseek"
                        />
                    </div>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>显示名称</label>
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
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem' }}>Base URL (可选)</label>
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

            {/* Add / Edit Strategy Profile Modal */}
            <Modal
                isOpen={isStrategyModalOpen}
                onClose={() => setIsStrategyModalOpen(false)}
                title={editingProfileKey ? "编辑算力 Profile" : "新增算力 Profile"}
                footer={<>
                    <button className="btn-outline" onClick={() => setIsStrategyModalOpen(false)}>取消</button>
                    <button className="btn-primary" onClick={handleSaveStrategy}>保存策略</button>
                </>}
            >
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1.2rem', maxHeight: '70vh', overflowY: 'auto', paddingRight: '0.25rem' }}>
                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.4rem', fontSize: '0.9rem' }}>Profile 标识 Key (唯一标识)</label>
                        <input 
                            type="text" 
                            value={profileKeyInput}
                            onChange={e => setProfileKeyInput(e.target.value)}
                            className="custom-input"
                            placeholder="如: High-Reasoning-Profile 或 Custom-Group"
                        />
                    </div>
                    
                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
                        <div>
                            <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.4rem', fontSize: '0.9rem' }}>显示名称</label>
                            <input 
                                type="text" 
                                value={profileNameInput}
                                onChange={e => setProfileNameInput(e.target.value)}
                                className="custom-input"
                                placeholder="如: 深度智力与代码算力组"
                            />
                        </div>
                        <div>
                            <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.4rem', fontSize: '0.9rem' }}>策略描述</label>
                            <input 
                                type="text" 
                                value={profileDescInput}
                                onChange={e => setProfileDescInput(e.target.value)}
                                className="custom-input"
                                placeholder="如: 适用于高逻辑复杂分析"
                            />
                        </div>
                    </div>

                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.5rem', fontSize: '0.9rem' }}>关联关联任务类型 (可多选勾选)</label>
                        <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(180px, 1fr))', gap: '0.5rem', background: 'rgba(0,0,0,0.2)', padding: '0.8rem', borderRadius: '8px', border: '1px solid var(--surface-border)' }}>
                            {PRESET_TASK_TYPES.map(pt => {
                                const isChecked = selectedTaskTypes.includes(pt.id);
                                return (
                                    <div 
                                        key={pt.id} 
                                        onClick={() => toggleTaskTypeSelection(pt.id)}
                                        style={{ 
                                            display: 'flex', 
                                            alignItems: 'center', 
                                            gap: '0.5rem', 
                                            cursor: 'pointer', 
                                            fontSize: '0.85rem',
                                            color: isChecked ? 'var(--text-primary)' : 'var(--text-secondary)',
                                            userSelect: 'none'
                                        }}
                                    >
                                        {isChecked ? <CheckSquare size={18} color={pt.color} weight="fill" /> : <Square size={18} color="var(--text-secondary)" />}
                                        <span>{pt.label}</span>
                                    </div>
                                );
                            })}
                        </div>
                    </div>

                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.4rem', fontSize: '0.9rem' }}>主路由模型 (Primary Model)</label>
                        <input 
                            type="text" 
                            value={primaryModelInput}
                            onChange={e => setPrimaryModelInput(e.target.value)}
                            className="custom-input"
                            placeholder="如: claude-3-5-sonnet, gemini-2.5-pro"
                        />
                    </div>

                    <div>
                        <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.4rem', fontSize: '0.9rem' }}>备用节点链 (Failover Models, 逗号分隔)</label>
                        <input 
                            type="text" 
                            value={failoverModelsInput}
                            onChange={e => setFailoverModelsInput(e.target.value)}
                            className="custom-input"
                            placeholder="如: gemini-2.5-pro, gpt-4o"
                        />
                    </div>

                    {/* Advanced Rules Section */}
                    <div style={{ borderTop: '1px solid var(--surface-border)', paddingTop: '1rem', marginTop: '0.5rem' }}>
                        <div style={{ fontWeight: 600, fontSize: '0.95rem', marginBottom: '0.8rem', color: '#c084fc', display: 'flex', alignItems: 'center', gap: '0.4rem' }}>
                            <WarningCircle size={18} /> 高级模型调度规则 (Advanced Routing Rules)
                        </div>
                        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
                            <div>
                                <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.4rem', fontSize: '0.85rem' }}>长文本分流模型 (Overflow Model)</label>
                                <input 
                                    type="text" 
                                    value={overflowModelInput}
                                    onChange={e => setOverflowModelInput(e.target.value)}
                                    className="custom-input"
                                    placeholder="如: gemini-1.5-pro"
                                />
                            </div>
                            <div>
                                <label style={{ color: 'var(--text-secondary)', display: 'block', marginBottom: '0.4rem', fontSize: '0.85rem' }}>Token 触发阈值</label>
                                <input 
                                    type="number" 
                                    value={tokenThresholdInput}
                                    onChange={e => setTokenThresholdInput(e.target.value)}
                                    className="custom-input"
                                    placeholder="32768"
                                />
                            </div>
                        </div>
                    </div>
                </div>
            </Modal>
        </div>
    );
}
