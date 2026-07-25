import Header from '../components/Header';
import Modal from '../components/Modal';
import { useState, useEffect } from 'react';
import { Robot, Plus, Code, Play, Stop, Brain, Database, Sliders, FloppyDisk, X } from '@phosphor-icons/react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES, DEFAULT_AGENT_PROMPT, DEFAULT_TONE } from '../constants';

interface Persona {
    base_prompt: string;
    allowed_skills: string[];
    tone: string;
}

interface Agent {
    id: string;
    name: string;
    status: string;
    capability_requirement: string;
    persona: Persona;
}

export default function AgentFactory() {
    const [agents, setAgents] = useState<Agent[]>([]);
    const [selectedAgent, setSelectedAgent] = useState<Agent | null>(null);
    const [isCreating, setIsCreating] = useState(false);
    const [newAgentName, setNewAgentName] = useState('');
    const [capabilityRequirement, setCapabilityRequirement] = useState('Tier-1-Logic');
    
    // Dynamic Capability Tiers Strategy Options
    const [capabilityTiers, setCapabilityTiers] = useState<string[]>(['Tier-1-Logic', 'Tier-2-Balanced', 'Tier-3-Fast', 'Multimodal-Vision']);
    
    // Available Skills Pool
    const [allAvailableSkills, setAllAvailableSkills] = useState<{ id: string; name: string }[]>([]);

    // Edit Form State
    const [editCapabilityRequirement, setEditCapabilityRequirement] = useState('Tier-1-Logic');
    const [editBasePrompt, setEditBasePrompt] = useState('');
    const [editAllowedSkills, setEditAllowedSkills] = useState<string[]>([]);
    
    // Attach Skill Modal
    const [isAttachSkillModalOpen, setIsAttachSkillModalOpen] = useState(false);

    useEffect(() => {
        fetchAgents();
        fetchRoutingTiers();
        fetchAllSkills();
    }, []);

    const fetchAgents = () => {
        fetchApi(API_ROUTES.AGENTS)
            .then((res: Response) => res.json())
            .then((data: Agent[]) => setAgents(data))
            .catch(console.error);
    };

    const fetchRoutingTiers = () => {
        fetchApi(API_ROUTES.MODELS_ROUTING)
            .then((res: Response) => res.json())
            .then(data => {
                if (data && typeof data === 'object') {
                    const keys = Object.keys(data);
                    if (keys.length > 0) {
                        setCapabilityTiers(keys);
                    }
                }
            })
            .catch(console.error);
    };

    const fetchAllSkills = () => {
        fetchApi(API_ROUTES.SKILLS)
            .then((res: Response) => res.json())
            .then((data: any[]) => {
                if (Array.isArray(data)) {
                    setAllAvailableSkills(data.map(s => ({ id: s.id || s.name, name: s.name || s.id })));
                }
            })
            .catch(console.error);
    };

    const handleSelectAgent = (agent: Agent) => {
        setSelectedAgent(agent);
        setEditCapabilityRequirement(agent.capability_requirement || 'Tier-1-Logic');
        setEditBasePrompt(agent.persona?.base_prompt || DEFAULT_AGENT_PROMPT);
        setEditAllowedSkills(agent.persona?.allowed_skills || []);
    };

    const handleCreateAgent = () => {
        if (!newAgentName) return;
        const newId = `agent_${Date.now()}`;
        const newAgent: Agent = {
            id: newId,
            name: newAgentName,
            status: 'Active',
            capability_requirement: capabilityRequirement,
            persona: {
                base_prompt: DEFAULT_AGENT_PROMPT,
                allowed_skills: [],
                tone: DEFAULT_TONE
            }
        };

        fetchApi(API_ROUTES.AGENTS, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(newAgent)
        })
        .then(() => {
            fetchAgents();
            setIsCreating(false);
            setNewAgentName('');
        })
        .catch(console.error);
    };

    const handleUpdateAgent = () => {
        if (!selectedAgent) return;
        const updated: Agent = {
            ...selectedAgent,
            capability_requirement: editCapabilityRequirement,
            persona: {
                ...selectedAgent.persona,
                base_prompt: editBasePrompt,
                allowed_skills: editAllowedSkills
            }
        };

        fetchApi(API_ROUTES.AGENT_BY_ID(selectedAgent.id), {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(updated)
        })
        .then((res) => {
            if (res.ok) {
                setSelectedAgent(updated);
                setAgents(agents.map(a => a.id === updated.id ? updated : a));
                alert('Agent 配置与技能许可列表已成功更新！');
            } else {
                alert('更新 Agent 策略失败');
            }
        })
        .catch(console.error);
    };

    const handleRemoveSkill = (skillName: string) => {
        setEditAllowedSkills(editAllowedSkills.filter(s => s !== skillName));
    };

    const handleAttachSkill = (skillName: string) => {
        if (!editAllowedSkills.includes(skillName)) {
            setEditAllowedSkills([...editAllowedSkills, skillName]);
        }
        setIsAttachSkillModalOpen(false);
    };

    return (
        <div className="main-content">
            <Header 
                title="智能体工厂 Agent Factory" 
                description="设计、装配和管理定制化的自主智能体。" 
            />

            <div style={{ display: 'grid', gridTemplateColumns: '320px 1fr', gap: '1.5rem', height: 'calc(100vh - 180px)' }}>
                {/* Left Panel: Agent List */}
                <div className="panel" style={{ display: 'flex', flexDirection: 'column', gap: '1rem', overflowY: 'auto' }}>
                    <div className="panel-header" style={{ marginBottom: 0 }}>
                        <span className="panel-title">我的智能体 (Agents)</span>
                        <button className="btn-outline" onClick={() => setIsCreating(true)}>
                            +
                        </button>
                    </div>
                    
                    {agents.length === 0 && !isCreating && <p style={{ color: 'var(--text-secondary)', textAlign: 'center', marginTop: '2rem' }}>No agents found</p>}
                    
                    {isCreating && (
                        <div style={{ padding: '1rem', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--primary-color)', borderRadius: '8px' }}>
                            <input 
                                type="text" 
                                placeholder="Agent Name..." 
                                value={newAgentName}
                                onChange={(e) => setNewAgentName(e.target.value)}
                                style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '4px', color: 'var(--text-primary)', marginBottom: '0.75rem' }}
                            />
                            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                            <label style={{ color: 'var(--text-secondary)' }}>算力能力要求 (Capability Tier)</label>
                            <select 
                                value={capabilityRequirement}
                                onChange={(e) => setCapabilityRequirement(e.target.value)}
                                className="custom-select"
                                style={{ marginBottom: '0.75rem' }}
                            >
                                {capabilityTiers.map(tier => (
                                    <option key={tier} value={tier}>{tier}</option>
                                ))}
                            </select>
                        </div>
                            <div style={{ display: 'flex', gap: '0.5rem' }}>
                                <button className="primary-btn" onClick={handleCreateAgent} style={{ flex: 1 }}>创建</button>
                                <button className="secondary-btn" onClick={() => setIsCreating(false)} style={{ flex: 1 }}>取消</button>
                            </div>
                        </div>
                    )}

                    {agents.map(agent => (
                        <div 
                            key={agent.id}
                            onClick={() => handleSelectAgent(agent)}
                            style={{ 
                                padding: '1rem', 
                                background: selectedAgent?.id === agent.id ? 'rgba(99, 102, 241, 0.1)' : 'rgba(255,255,255,0.02)',
                                border: `1px solid ${selectedAgent?.id === agent.id ? 'var(--primary-color)' : 'var(--surface-border)'}`,
                                borderRadius: '8px',
                                cursor: 'pointer',
                                transition: 'all 0.2s',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '0.75rem'
                            }}
                        >
                            <div style={{ padding: '0.5rem', background: 'rgba(255,255,255,0.05)', borderRadius: '6px' }}>
                                <Robot size={24} color={agent.status === 'running' ? 'var(--secondary-color)' : 'var(--primary-color)'} />
                            </div>
                            <div style={{ flex: 1, minWidth: 0 }}>
                                <div style={{ fontWeight: 500, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{agent.name}</div>
                                <div style={{ fontSize: '0.75rem', color: 'var(--text-secondary)' }}>{agent.capability_requirement || 'Default'}</div>
                            </div>
                            {agent.status === 'Active' && <div style={{ width: '8px', height: '8px', borderRadius: '50%', background: 'var(--secondary-color)', boxShadow: '0 0 8px var(--secondary-color)' }}></div>}
                        </div>
                    ))}
                </div>

                {/* Right Panel: Agent Configuration */}
                <div className="panel" style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem', overflowY: 'auto' }}>
                    {selectedAgent ? (
                        <>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--surface-border)', paddingBottom: '1rem' }}>
                                <div>
                                    <h2 style={{ fontSize: '1.5rem', fontWeight: 600, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                        {selectedAgent.name}
                                        <span style={{ fontSize: '0.8rem', padding: '0.2rem 0.5rem', background: 'rgba(99, 102, 241, 0.2)', color: 'var(--primary-color)', borderRadius: '4px' }}>
                                            ID: {selectedAgent.id}
                                        </span>
                                    </h2>
                                    <p style={{ color: 'var(--text-secondary)', marginTop: '0.5rem', fontSize: '0.9rem' }}>Configure the brain and capabilities of this agent.</p>
                                </div>
                                <div style={{ display: 'flex', gap: '0.5rem' }}>
                                    <button className="primary-btn" onClick={handleUpdateAgent} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                        <FloppyDisk size={18} /> 保存配置
                                    </button>
                                </div>
                            </div>

                            <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1.5rem' }}>
                                {/* Configuration */}
                                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                                    <div>
                                        <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                                            <Sliders size={18} /> 能力标签映射策略 (Capability Strategy)
                                        </label>
                                        <select 
                                            value={editCapabilityRequirement}
                                            onChange={(e) => setEditCapabilityRequirement(e.target.value)}
                                            className="custom-select"
                                        >
                                            {capabilityTiers.map(tier => (
                                                <option key={tier} value={tier}>{tier}</option>
                                            ))}
                                        </select>
                                    </div>

                                    <div>
                                        <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                                            <Brain size={18} /> Base Prompt (System Instructions)
                                        </label>
                                        <textarea 
                                            value={editBasePrompt}
                                            onChange={(e) => setEditBasePrompt(e.target.value)}
                                            style={{ width: '100%', height: '150px', padding: '1rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', resize: 'vertical', fontFamily: 'monospace' }}
                                        />
                                    </div>

                                    <div>
                                        <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                                            <Database size={18} /> Context Window Size
                                        </label>
                                        <input 
                                            type="number" 
                                            defaultValue={8192}
                                            style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)' }}
                                        />
                                    </div>
                                </div>

                                {/* Skills */}
                                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                                    <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                                        <Code size={18} /> Equipped Skills
                                    </label>
                                    <div style={{ background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', padding: '1rem', minHeight: '150px' }}>
                                        {editAllowedSkills.length === 0 ? (
                                            <p style={{ color: 'var(--text-secondary)', textAlign: 'center', marginTop: '2rem' }}>No skills equipped.</p>
                                        ) : (
                                            <div style={{ display: 'flex', flexWrap: 'wrap', gap: '0.5rem' }}>
                                                {editAllowedSkills.map(skill => (
                                                    <span key={skill} style={{ padding: '0.4rem 0.8rem', background: 'rgba(99, 102, 241, 0.15)', color: 'var(--primary-color)', border: '1px solid rgba(99, 102, 241, 0.3)', borderRadius: '20px', fontSize: '0.85rem', display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                                        <Code size={14} /> {skill}
                                                        <X 
                                                            size={14} 
                                                            style={{ cursor: 'pointer', opacity: 0.7 }} 
                                                            onClick={() => handleRemoveSkill(skill)}
                                                            title="移除技能"
                                                        />
                                                    </span>
                                                ))}
                                            </div>
                                        )}
                                        <button 
                                            className="secondary-btn" 
                                            onClick={() => setIsAttachSkillModalOpen(true)}
                                            style={{ width: '100%', marginTop: '1rem', display: 'flex', alignItems: 'center', justifyItems: 'center', justifyContent: 'center', gap: '0.5rem', padding: '0.5rem' }}
                                        >
                                            <Plus /> Attach New Skill
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </>
                    ) : (
                        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--text-secondary)' }}>
                            <Robot size={64} opacity={0.2} style={{ marginBottom: '1rem' }} />
                            <p>Select an agent from the list to view and edit its configuration.</p>
                        </div>
                    )}
                </div>
            </div>

            {/* Attach Skill Modal */}
            <Modal
                isOpen={isAttachSkillModalOpen}
                onClose={() => setIsAttachSkillModalOpen(false)}
                title="选择并装配新技能 (Attach Skill)"
                footer={<>
                    <button className="btn-outline" onClick={() => setIsAttachSkillModalOpen(false)}>取消</button>
                </>}
            >
                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.75rem', maxHeight: '350px', overflowY: 'auto' }}>
                    {allAvailableSkills.map(skill => {
                        const isEquipped = editAllowedSkills.includes(skill.name);
                        return (
                            <div 
                                key={skill.id}
                                style={{ 
                                    padding: '0.75rem 1rem', 
                                    background: 'rgba(255,255,255,0.02)', 
                                    border: '1px solid var(--surface-border)', 
                                    borderRadius: '8px',
                                    display: 'flex',
                                    justify: 'space-between',
                                    alignItems: 'center'
                                }}
                            >
                                <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                    <Code size={18} color="var(--primary-color)" />
                                    <span style={{ fontWeight: 500 }}>{skill.name}</span>
                                </div>
                                {isEquipped ? (
                                    <span style={{ fontSize: '0.8rem', color: 'var(--secondary-color)' }}>已装配 (Equipped)</span>
                                ) : (
                                    <button 
                                        className="btn-outline" 
                                        onClick={() => handleAttachSkill(skill.name)}
                                        style={{ padding: '0.3rem 0.8rem', fontSize: '0.8rem' }}
                                    >
                                        + 装配
                                    </button>
                                )}
                            </div>
                        );
                    })}

                    {allAvailableSkills.length === 0 && (
                        <p style={{ color: 'var(--text-secondary)', textAlign: 'center', padding: '1rem' }}>暂无可用技能</p>
                    )}
                </div>
            </Modal>
        </div>
    );
}
