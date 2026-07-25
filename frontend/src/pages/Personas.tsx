import Header from '../components/Header';
import { useState, useEffect } from 'react';
import { UserCircle, Trash, Brain, Link as LinkIcon, Database } from '@phosphor-icons/react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES, DEFAULT_PERSONA_PROMPT, DEFAULT_TONE } from '../constants';

interface Persona {
    id: string;
    name: string;
    base_prompt: string;
    allowed_skills: string[];
    tone: string;
    routing_strategy: string;
    routed_agents: string[];
}

export default function Personas() {
    const [personas, setPersonas] = useState<Persona[]>([]);
    const [selectedPersona, setSelectedPersona] = useState<Persona | null>(null);
    const [isCreating, setIsCreating] = useState(false);
    const [newPersonaName, setNewPersonaName] = useState('');

    useEffect(() => {
        fetchPersonas();
    }, []);

    const fetchPersonas = () => {
        fetchApi(API_ROUTES.PERSONAS)
            .then((res: Response) => res.json())
            .then((data: Persona[]) => setPersonas(data))
            .catch(console.error);
    };

    const handleCreatePersona = () => {
        if (!newPersonaName) return;
        const newId = `persona_${Date.now()}`;
        const newPersona: Persona = {
            id: newId,
            name: newPersonaName,
            base_prompt: DEFAULT_PERSONA_PROMPT,
            allowed_skills: [],
            tone: DEFAULT_TONE,
            routing_strategy: "direct",
            routed_agents: []
        };

        fetchApi(API_ROUTES.PERSONAS, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(newPersona)
        })
        .then(() => {
            fetchPersonas();
            setIsCreating(false);
            setNewPersonaName('');
        })
        .catch(console.error);
    };

    const handleDeletePersona = (id: string, e: React.MouseEvent) => {
        e.stopPropagation();
        if (confirm('Are you sure you want to delete this Persona?')) {
            fetchApi(API_ROUTES.PERSONA_BY_ID(id), { method: 'DELETE' })
                .then(() => {
                    if (selectedPersona?.id === id) setSelectedPersona(null);
                    fetchPersonas();
                })
                .catch(console.error);
        }
    };

    const handleUpdatePersona = (field: keyof Persona, value: any) => {
        if (!selectedPersona) return;
        const updated = { ...selectedPersona, [field]: value };
        setSelectedPersona(updated);
        
        fetchApi(`/api/personas/${selectedPersona.id}`, {
            method: 'PUT',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify(updated)
        })
        .then(() => fetchPersonas())
        .catch(console.error);
    };

    return (
        <main className="main-content">
            <Header 
                title="人格管理 Personas" 
                description="Configure AI personas, their behavior, and orchestration routing."
            />

            <div style={{ display: 'grid', gridTemplateColumns: '320px 1fr', gap: '1.5rem', height: 'calc(100vh - 180px)' }}>
                {/* Left Panel: Persona List */}
                <div className="panel" style={{ display: 'flex', flexDirection: 'column', gap: '1rem', overflowY: 'auto' }}>
                    <div className="panel-header" style={{ marginBottom: 0 }}>
                        <span className="panel-title">人格列表 (Personas)</span>
                        <button className="btn-outline" onClick={() => setIsCreating(true)}>
                            +
                        </button>
                    </div>
                    
                    {personas.length === 0 && !isCreating && <p style={{ color: 'var(--text-secondary)', textAlign: 'center', marginTop: '2rem' }}>No personas found</p>}
                    
                    {isCreating && (
                        <div style={{ padding: '1rem', background: 'rgba(255,255,255,0.02)', border: '1px solid var(--primary-color)', borderRadius: '8px' }}>
                            <input 
                                type="text" 
                                placeholder="Persona Name..." 
                                value={newPersonaName}
                                onChange={(e) => setNewPersonaName(e.target.value)}
                                style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '4px', color: 'var(--text-primary)', marginBottom: '0.75rem' }}
                            />
                            <div style={{ display: 'flex', gap: '0.5rem' }}>
                                <button className="primary-btn" onClick={handleCreatePersona} style={{ flex: 1 }}>创建</button>
                                <button className="secondary-btn" onClick={() => setIsCreating(false)} style={{ flex: 1 }}>取消</button>
                            </div>
                        </div>
                    )}

                    {personas.map(persona => (
                        <div 
                            key={persona.id}
                            onClick={() => setSelectedPersona(persona)}
                            style={{ 
                                padding: '1rem', 
                                background: selectedPersona?.id === persona.id ? 'rgba(99, 102, 241, 0.1)' : 'rgba(255,255,255,0.02)',
                                border: `1px solid ${selectedPersona?.id === persona.id ? 'var(--primary-color)' : 'var(--surface-border)'}`,
                                borderRadius: '8px',
                                cursor: 'pointer',
                                transition: 'all 0.2s',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '0.75rem'
                            }}
                        >
                            <div style={{ padding: '0.5rem', background: 'rgba(255,255,255,0.05)', borderRadius: '6px' }}>
                                <UserCircle size={24} color={selectedPersona?.id === persona.id ? 'var(--primary-color)' : 'var(--text-secondary)'} />
                            </div>
                            <div style={{ flex: 1, minWidth: 0 }}>
                                <div style={{ fontWeight: 500, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{persona.name}</div>
                            </div>
                            <button 
                                onClick={(e) => handleDeletePersona(persona.id, e)}
                                style={{ background: 'transparent', border: 'none', color: 'var(--danger-color)', cursor: 'pointer', padding: '4px' }}
                            >
                                <Trash size={16} />
                            </button>
                        </div>
                    ))}
                </div>

                {/* Right Panel: Persona Configuration */}
                <div className="panel" style={{ display: 'flex', flexDirection: 'column', gap: '1.5rem', overflowY: 'auto' }}>
                    {selectedPersona ? (
                        <>
                            <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', borderBottom: '1px solid var(--surface-border)', paddingBottom: '1rem' }}>
                                <div>
                                    <h2 style={{ fontSize: '1.5rem', fontWeight: 600, display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                                        {selectedPersona.name}
                                        <span style={{ fontSize: '0.8rem', padding: '0.2rem 0.5rem', background: 'rgba(99, 102, 241, 0.2)', color: 'var(--primary-color)', borderRadius: '4px' }}>
                                            ID: {selectedPersona.id}
                                        </span>
                                    </h2>
                                    <p style={{ color: 'var(--text-secondary)', marginTop: '0.5rem', fontSize: '0.9rem' }}>Configure instructions and routing logic for this Persona.</p>
                                </div>
                            </div>

                            <div style={{ display: 'grid', gridTemplateColumns: '1fr', gap: '1.5rem' }}>
                                <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
                                    <div>
                                        <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                                            <Brain size={18} /> Base Prompt (System Instructions)
                                        </label>
                                        <textarea 
                                            value={selectedPersona.base_prompt}
                                            onChange={(e) => handleUpdatePersona('base_prompt', e.target.value)}
                                            style={{ width: '100%', height: '200px', padding: '1rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)', resize: 'vertical', fontFamily: 'monospace' }}
                                        />
                                    </div>

                                    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '1rem' }}>
                                        <div>
                                            <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                                                <Database size={18} /> Tone / Voice
                                            </label>
                                            <input 
                                                type="text" 
                                                value={selectedPersona.tone}
                                                onChange={(e) => handleUpdatePersona('tone', e.target.value)}
                                                style={{ width: '100%', padding: '0.75rem', background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '8px', color: 'var(--text-primary)' }}
                                            />
                                        </div>
                                        <div>
                                            <label style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', color: 'var(--text-secondary)', marginBottom: '0.5rem', fontSize: '0.9rem' }}>
                                                <LinkIcon size={18} /> Routing Strategy
                                            </label>
                                            <select 
                                                value={selectedPersona.routing_strategy}
                                                onChange={(e) => handleUpdatePersona('routing_strategy', e.target.value)}
                                                className="custom-select"
                                            >
                                                <option value="direct">Direct Answering (No Routing)</option>
                                                <option value="orchestrator">Orchestrator (Route to Agents)</option>
                                            </select>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </>
                    ) : (
                        <div style={{ display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'var(--text-secondary)' }}>
                            <UserCircle size={48} style={{ marginBottom: '1rem', opacity: 0.5 }} />
                            <p>Select a persona from the list to view and edit its configuration.</p>
                        </div>
                    )}
                </div>
            </div>
        </main>
    );
}
