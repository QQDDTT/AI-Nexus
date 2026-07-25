import Header from '../components/Header';
import Modal from '../components/Modal';
import { useState, useEffect } from 'react';
import { Clock, Lightning, HardDrives, PlayCircle, Robot, Trash, Plus } from '@phosphor-icons/react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES } from '../constants';

interface EventTrigger {
    id: string;
    type: string;
    source: string;
    status: string;
    lastRun: string;
    nextRun?: string;
    targetAgent: string;
}

export default function TaskScheduler() {
    const [triggers, setTriggers] = useState<EventTrigger[]>([]);
    const [agents, setAgents] = useState<{ id: string; name: string }[]>([]);
    
    // Add Modal State
    const [isAddModalOpen, setIsAddModalOpen] = useState(false);
    const [triggerType, setTriggerType] = useState('cron');
    const [triggerSource, setTriggerSource] = useState('0 */1 * * *');
    const [targetAgent, setTargetAgent] = useState('');

    useEffect(() => {
        fetchTriggers();
        fetchAgents();
    }, []);

    const fetchTriggers = () => {
        fetchApi(API_ROUTES.TRIGGERS)
            .then((res: Response) => res.json())
            .then((data: EventTrigger[]) => setTriggers(data))
            .catch(console.error);
    };

    const fetchAgents = () => {
        fetchApi(API_ROUTES.AGENTS)
            .then((res: Response) => res.json())
            .then((data: any[]) => setAgents(Array.isArray(data) ? data : []))
            .catch(console.error);
    };

    const handleToggleStatus = (trigger: EventTrigger) => {
        const newStatus = trigger.status === 'active' ? 'suspended' : 'active';
        fetchApi(API_ROUTES.TRIGGER_BY_ID(trigger.id), {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ ...trigger, status: newStatus })
        })
        .then(() => fetchTriggers())
        .catch(console.error);
    };

    const handleDeleteTrigger = (id: string) => {
        if (!window.confirm(`确定要删除触发器 ${id} 吗？`)) return;
        fetchApi(API_ROUTES.TRIGGER_BY_ID(id), { method: 'DELETE' })
            .then(() => fetchTriggers())
            .catch(console.error);
    };

    const handleAddTrigger = () => {
        if (!triggerSource.trim()) return;
        const newId = `trig_${Date.now()}`;
        const newTrig: EventTrigger = {
            id: newId,
            type: triggerType,
            source: triggerSource,
            status: 'active',
            lastRun: 'Never',
            targetAgent: targetAgent || 'Local Admin'
        };

        fetchApi(API_ROUTES.TRIGGERS, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(newTrig)
        })
        .then(() => {
            fetchTriggers();
            setIsAddModalOpen(false);
            setTriggerSource('0 */1 * * *');
        })
        .catch(console.error);
    };

    const getTypeIcon = (type: string) => {
        switch (type) {
            case 'cron': return <Clock size={20} color="var(--primary-color)" />;
            case 'event': 
            case 'webhook': return <Lightning size={20} color="var(--accent-color)" />;
            case 'poll': return <HardDrives size={20} color="var(--secondary-color)" />;
            default: return <Clock size={20} />;
        }
    };

    return (
        <div className="main-content">
            <Header 
                title="任务调度 Task Scheduler" 
                description="基于 GraphRAG 事件驱动与 Wasm 原生 Poll 模型的系统调度面板。" 
            />

            <div style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem' }}>
                <div className="panel">
                    <div className="panel-header" style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <span className="panel-title">后台事件追踪 (Event Triggers)</span>
                        <button className="btn-outline" onClick={() => setIsAddModalOpen(true)} style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                            <Plus size={16} /> 新增触发器
                        </button>
                    </div>
                    
                    <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(350px, 1fr))', gap: '1.5rem' }}>
                        {triggers.map(trigger => (
                            <div key={trigger.id} style={{ padding: '1.5rem', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--surface-border)', borderRadius: '12px', position: 'relative', overflow: 'hidden' }}>
                                {trigger.status === 'active' && <div style={{ position: 'absolute', top: 0, left: 0, width: '4px', height: '100%', background: 'var(--secondary-color)' }}></div>}
                                
                                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'flex-start', marginBottom: '1rem' }}>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.75rem' }}>
                                        <div style={{ padding: '0.5rem', background: 'rgba(255,255,255,0.05)', borderRadius: '8px' }}>
                                            {getTypeIcon(trigger.type)}
                                        </div>
                                        <div>
                                            <div style={{ fontWeight: 600, fontSize: '1.1rem', textTransform: 'capitalize' }}>{trigger.type} Trigger</div>
                                            <div style={{ fontSize: '0.85rem', color: 'var(--text-secondary)' }}>ID: {trigger.id}</div>
                                        </div>
                                    </div>
                                    <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                        <span style={{ padding: '0.2rem 0.6rem', fontSize: '0.75rem', borderRadius: '12px', background: trigger.status === 'active' ? 'rgba(16, 185, 129, 0.15)' : 'rgba(239, 68, 68, 0.15)', color: trigger.status === 'active' ? 'var(--secondary-color)' : 'var(--danger-color)' }}>
                                            {trigger.status}
                                        </span>
                                        <button 
                                            onClick={() => handleDeleteTrigger(trigger.id)} 
                                            style={{ background: 'transparent', border: 'none', color: 'var(--danger-color)', cursor: 'pointer', padding: '0.25rem', opacity: 0.7 }}
                                            title="删除触发器"
                                        >
                                            <Trash size={18} />
                                        </button>
                                    </div>
                                </div>

                                <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem', fontSize: '0.9rem' }}>
                                    <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                                        <span style={{ color: 'var(--text-secondary)' }}>Source / Schedule</span>
                                        <span style={{ fontFamily: 'monospace', color: 'var(--primary-color)' }}>{trigger.source}</span>
                                    </div>
                                    <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                                        <span style={{ color: 'var(--text-secondary)' }}>Target Agent</span>
                                        <span style={{ display: 'flex', alignItems: 'center', gap: '0.25rem' }}>
                                            <Robot size={14} /> {trigger.targetAgent}
                                        </span>
                                    </div>
                                    <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                                        <span style={{ color: 'var(--text-secondary)' }}>Last Run</span>
                                        <span>{trigger.lastRun}</span>
                                    </div>
                                    {trigger.nextRun && (
                                        <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                                            <span style={{ color: 'var(--text-secondary)' }}>Next Run</span>
                                            <span style={{ color: 'var(--accent-color)' }}>{trigger.nextRun}</span>
                                        </div>
                                    )}
                                </div>
                                
                                <div style={{ marginTop: '1.5rem', display: 'flex', gap: '0.5rem' }}>
                                    {trigger.status === 'suspended' ? (
                                        <button onClick={() => handleToggleStatus(trigger)} className="primary-btn" style={{ flex: 1, padding: '0.5rem', fontSize: '0.85rem', display: 'flex', justifyContent: 'center', alignItems: 'center', gap: '0.25rem' }}>
                                            <PlayCircle size={16} /> Resume
                                        </button>
                                    ) : (
                                        <button onClick={() => handleToggleStatus(trigger)} className="secondary-btn" style={{ flex: 1, padding: '0.5rem', fontSize: '0.85rem' }}>Suspend</button>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>

                    {triggers.length === 0 && (
                        <div style={{ color: 'var(--text-secondary)', textAlign: 'center', padding: '3rem 1rem', border: '1px dashed var(--surface-border)', borderRadius: '12px', marginTop: '1rem' }}>
                            暂无事件触发器。点击右上角“新增触发器”创建 Cron / Webhook / Poll 任务。
                        </div>
                    )}
                </div>
            </div>

            <Modal
                isOpen={isAddModalOpen}
                onClose={() => setIsAddModalOpen(false)}
                title="新增后台事件触发器"
                footer={<>
                    <button className="btn-outline" onClick={() => setIsAddModalOpen(false)}>取消</button>
                    <button className="btn-primary" onClick={handleAddTrigger}>保存创建</button>
                </>}
            >
                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        <label style={{ color: 'var(--text-secondary)' }}>触发器类型 (Trigger Type)</label>
                        <select 
                            value={triggerType}
                            onChange={e => setTriggerType(e.target.value)}
                            className="custom-select"
                        >
                            <option value="cron">Cron Schedule (定时表达式)</option>
                            <option value="webhook">WebHook (外部 HTTP 事件响应)</option>
                            <option value="poll">Poll (向量空间 / 数据库轮询)</option>
                        </select>
                    </div>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        <label style={{ color: 'var(--text-secondary)' }}>触发源/表达式 (Source / Expression)</label>
                        <input 
                            type="text" 
                            value={triggerSource}
                            onChange={e => setTriggerSource(e.target.value)}
                            placeholder={triggerType === 'cron' ? '如: 0 */1 * * *' : triggerType === 'webhook' ? '如: WebHook: GitHub Push' : '如: NexusDB Vector Poll'}
                            style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', outline: 'none' }}
                        />
                    </div>

                    <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
                        <label style={{ color: 'var(--text-secondary)' }}>目标智能体 (Target Agent)</label>
                        <select 
                            value={targetAgent}
                            onChange={e => setTargetAgent(e.target.value)}
                            className="custom-select"
                        >
                            {agents.map(a => (
                                <option key={a.id} value={a.name || a.id}>{a.name || a.id}</option>
                            ))}
                            {agents.length === 0 && <option value="Local Admin">Local Admin</option>}
                        </select>
                    </div>
                </div>
            </Modal>
        </div>
    );
}
