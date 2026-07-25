import Header from '../components/Header';
import { Code, Wrench, CheckCircle, Warning, Play, ArrowClockwise, Book, FloppyDisk, MagicWand } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES } from '../constants';

interface Skill {
    id: string;
    name: string;
    status: string;
    source_code: string;
    type?: string;
    is_core?: boolean;
}

export default function Skills() {
    const [skills, setSkills] = useState<Skill[]>([]);
    const [selectedSkill, setSelectedSkill] = useState<Skill | null>(null);
    const [editorCode, setEditorCode] = useState('');
    const [isCompiling, setIsCompiling] = useState(false);
    const [compileOutput, setCompileOutput] = useState('');
    
    // AI Assist state
    const [aiInstruction, setAiInstruction] = useState('');
    const [isAiLoading, setIsAiLoading] = useState(false);

    const isReadOnly = !selectedSkill || selectedSkill.type === 'Native' || selectedSkill.is_core;

    const handleAiAssist = () => {
        if (!selectedSkill || !aiInstruction.trim()) return;
        setIsAiLoading(true);
        setCompileOutput('AI is analyzing and rewriting code... please wait.');
        
        fetchApi(API_ROUTES.SKILL_AI_ASSIST, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ 
                skill_name: selectedSkill.name, 
                current_code: editorCode,
                instruction: aiInstruction 
            })
        })
        .then(res => res.json())
        .then(data => {
            if (data.status === 'success') {
                setEditorCode(data.suggested_code);
                setCompileOutput('AI assistance applied successfully. You can review the code and save/compile it.');
                setAiInstruction('');
            } else {
                setCompileOutput(`AI Error: ${data.error}`);
            }
        })
        .catch(err => {
            setCompileOutput(`Network error: ${err.message}`);
        })
        .finally(() => {
            setIsAiLoading(false);
        });
    };

    const fetchSkills = () => {
        fetchApi(API_ROUTES.SKILLS)
            .then(res => res.json())
            .then(data => {
                setSkills(data);
                if (data.length > 0 && !selectedSkill) {
                    setSelectedSkill(data[0]);
                    setEditorCode(data[0].source_code);
                }
            })
            .catch(console.error);
    };

    useEffect(() => {
        fetchSkills();
    }, []);

    const handleAction = () => {
        if (!selectedSkill) return;
        setIsCompiling(true);
        
        if (selectedSkill.type === 'Markdown') {
            setCompileOutput('Saving...');
            fetchApi(API_ROUTES.SKILL_SAVE_MD, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ name: selectedSkill.name, source_code: editorCode })
            })
            .then(res => res.json())
            .then(data => {
                if (data.status === 'success') {
                    setCompileOutput('Markdown saved successfully.');
                    fetchSkills();
                } else {
                    setCompileOutput(`Error: ${data.error}`);
                }
            })
            .catch(err => {
                setCompileOutput(`Network error: ${err.message}`);
            })
            .finally(() => {
                setIsCompiling(false);
            });
            return;
        }

        setCompileOutput('Compiling...');
        fetchApi(API_ROUTES.SKILL_COMPILE, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ name: selectedSkill.name, source_code: editorCode })
        })
        .then(res => res.json())
        .then(data => {
            if (data.status === 'success') {
                setCompileOutput('Compilation successful. WASM injected into Sandbox.');
                fetchSkills();
            } else {
                setCompileOutput(`Error: ${data.error}`);
            }
        })
        .catch(err => {
            setCompileOutput(`Network error: ${err.message}`);
        })
        .finally(() => {
            setIsCompiling(false);
        });
    };

    return (
        <div className="main-content">
            <Header 
                title="技能仓库 Skill Marketplace" 
                description="基于 GraphSkillRegistry 与 Wasm Sandbox 的底层执行能力池。" 
            />

            <div style={{ display: 'grid', gridTemplateColumns: '300px 1fr', gap: '1.5rem', height: 'calc(100vh - 180px)' }}>
                {/* Left Panel: Skill Inventory */}
                <div className="panel" style={{ display: 'flex', flexDirection: 'column', gap: '1rem', overflowY: 'auto' }}>
                    <div className="panel-header" style={{ marginBottom: 0, display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <span className="panel-title">已安装技能 (Installed)</span>
                        <button 
                            className="btn-outline" 
                            onClick={async () => {
                                const intent = window.prompt("请输入新技能的功能描述，AI 将自动命名并生成模板 (留空则创建默认模板):");
                                if (intent === null) return;
                                
                                let newName = `new_skill_${Math.floor(Math.random() * 1000)}`;
                                let newCode = `---\nname: ${newName}\ndescription: 这是一个新建的草稿技能\n---\n\n# ${newName}\n\n`;

                                if (intent) {
                                    try {
                                        setCompileOutput('AI is generating skill template and name...');
                                        const res = await fetchApi(API_ROUTES.SKILL_AI_ASSIST, {
                                            method: 'POST',
                                            headers: { 'Content-Type': 'application/json' },
                                            body: JSON.stringify({
                                                skill_name: 'draft',
                                                current_code: '',
                                                instruction: `Create a Markdown skill based on this intent: "${intent}". Output MUST include frontmatter with a suitable 'name' (snake_case) and 'description', followed by the markdown body.`
                                            })
                                        });
                                        const data = await res.json();
                                        if (data.status === 'success' && data.suggested_code) {
                                            newCode = data.suggested_code.replace(/```markdown\n/g, '').replace(/```\n?/g, '').trim();
                                            const nameMatch = newCode.match(/name:\s*([a-zA-Z0-9_]+)/);
                                            if (nameMatch && nameMatch[1]) {
                                                newName = nameMatch[1];
                                            }
                                            setCompileOutput('AI generation complete.');
                                        }
                                    } catch (err: any) {
                                        setCompileOutput(`AI generation failed: ${err.message}. Using default template.`);
                                    }
                                }

                                const newSkill = {
                                    id: newName,
                                    name: newName,
                                    status: 'Draft',
                                    type: 'Markdown',
                                    source_code: newCode
                                };
                                
                                setSkills(prev => [newSkill, ...prev]);
                                setSelectedSkill(newSkill);
                                setEditorCode(newCode);
                                if (!intent) setCompileOutput('');
                            }}
                        >
                            +
                        </button>
                    </div>
                    
                    {skills.length === 0 && <p style={{ color: 'var(--text-secondary)', textAlign: 'center', marginTop: '2rem' }}>No skills installed</p>}
                    
                    {skills.map(skill => (
                        <div 
                            key={skill.id}
                            style={{ 
                                padding: '1rem', 
                                background: selectedSkill?.id === skill.id ? 'rgba(99, 102, 241, 0.1)' : 'rgba(255,255,255,0.02)',
                                border: `1px solid ${selectedSkill?.id === skill.id ? 'var(--primary-color)' : 'var(--surface-border)'}`,
                                borderRadius: '12px',
                                cursor: 'pointer',
                                transition: 'all 0.2s',
                                display: 'flex',
                                alignItems: 'center',
                                gap: '0.75rem',
                                position: 'relative'
                            }}
                            onClick={() => {
                                setSelectedSkill(skill);
                                setEditorCode(skill.source_code);
                                setCompileOutput('');
                            }}
                        >
                            <div style={{ padding: '0.5rem', background: 'rgba(255,255,255,0.05)', borderRadius: '6px' }}>
                                {skill.type === 'Markdown' 
                                    ? <Book size={20} color={skill.status === 'Active' ? 'var(--secondary-color)' : 'var(--text-secondary)'} />
                                    : <Wrench size={20} color={skill.status === 'Active' ? 'var(--secondary-color)' : 'var(--text-secondary)'} />
                                }
                            </div>
                            <div style={{ flex: 1, overflow: 'hidden' }}>
                                <div style={{ fontWeight: 500, whiteSpace: 'nowrap', textOverflow: 'ellipsis', overflow: 'hidden' }}>{skill.name}</div>
                                <div style={{ fontSize: '0.8rem', color: 'var(--text-secondary)', marginTop: '0.25rem', display: 'flex', alignItems: 'center', gap: '4px' }}>
                                    {skill.status === 'Active' ? <CheckCircle color="var(--secondary-color)" weight="fill" /> : <Warning color="#f59e0b" weight="fill" />} 
                                    {skill.type === 'Markdown' ? 'Active (Markdown)' : skill.status}
                                </div>
                            </div>
                            { !skill.is_core && (
                                <button
                                    style={{
                                        background: 'transparent',
                                        border: 'none',
                                        color: 'var(--text-secondary)',
                                        cursor: 'pointer',
                                        padding: '0.5rem',
                                        opacity: 0.6,
                                        transition: 'opacity 0.2s',
                                    }}
                                    onMouseEnter={(e) => (e.currentTarget.style.opacity = '1')}
                                    onMouseLeave={(e) => (e.currentTarget.style.opacity = '0.6')}
                                    onClick={async (e) => {
                                        e.stopPropagation();
                                        if (window.confirm(`Are you sure you want to delete ${skill.name}?`)) {
                                            try {
                                                const res = await fetchApi(API_ROUTES.SKILL_BY_ID(skill.name), { method: 'DELETE' });
                                                if (res.ok) {
                                                    setSkills(prev => prev.filter(s => s.id !== skill.id));
                                                    if (selectedSkill?.id === skill.id) {
                                                        setSelectedSkill(null);
                                                        setEditorCode('');
                                                    }
                                                } else {
                                                    alert('Failed to delete skill.');
                                                }
                                            } catch (err) {
                                                console.error(err);
                                                // Handle draft deletion that hasn't been saved yet
                                                setSkills(prev => prev.filter(s => s.id !== skill.id));
                                                if (selectedSkill?.id === skill.id) {
                                                    setSelectedSkill(null);
                                                    setEditorCode('');
                                                }
                                            }
                                        }
                                    }}
                                    title="Delete Skill"
                                >
                                    <svg width="18" height="18" fill="currentColor" viewBox="0 0 256 256">
                                        <path d="M216,48H176V40a24,24,0,0,0-24-24H104A24,24,0,0,0,80,40v8H40a8,8,0,0,0,0,16h8V208a16,16,0,0,0,16,16H192a16,16,0,0,0,16-16V64h8a8,8,0,0,0,0-16ZM96,40a8,8,0,0,1,8-8h48a8,8,0,0,1,8,8v8H96Zm96,168H64V64H192ZM112,104v64a8,8,0,0,1-16,0V104a8,8,0,0,1,16,0Zm48,0v64a8,8,0,0,1-16,0V104a8,8,0,0,1,16,0Z"></path>
                                    </svg>
                                </button>
                            )}
                        </div>
                    ))}
                </div>

                {/* Right Panel: Code Editor */}
                <div className="panel" style={{ display: 'flex', flexDirection: 'column', padding: '1.5rem', gap: '1rem' }}>
                    <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem' }}>
                            <Code size={20} color="var(--text-secondary)" />
                            <span style={{ fontWeight: 500, fontFamily: 'monospace' }}>
                                {selectedSkill 
                                    ? (selectedSkill.type === 'Markdown' ? `${selectedSkill.name}/SKILL.md` : `${selectedSkill.name}.rs`) 
                                    : 'Select a skill'
                                }
                            </span>
                        </div>
                        <div style={{ display: 'flex', gap: '0.5rem' }}>
                            <button 
                                className="btn-primary"
                                onClick={handleAction}
                                disabled={isReadOnly || isCompiling}
                                style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', opacity: (isReadOnly || isCompiling) ? 0.7 : 1, cursor: (isReadOnly || isCompiling) ? 'not-allowed' : 'pointer' }}
                            >
                                {isCompiling 
                                    ? <ArrowClockwise className="spin" /> 
                                    : (selectedSkill?.type === 'Markdown' ? <FloppyDisk weight="fill" /> : <Play weight="fill" />)
                                }
                                {isCompiling 
                                    ? (selectedSkill?.type === 'Markdown' ? 'Saving...' : 'Compiling to WASM...') 
                                    : (selectedSkill?.type === 'Markdown' ? 'Save Markdown' : 'Compile & Inject')
                                }
                            </button>
                        </div>
                    </div>
                    
                    {/* AI Assist Bar */}
                    <div style={{ display: 'flex', gap: '0.5rem', background: 'rgba(99, 102, 241, 0.05)', padding: '0.75rem', borderRadius: '8px', border: '1px solid rgba(99, 102, 241, 0.2)' }}>
                        <input
                            type="text"
                            placeholder="Ask AI to modify this skill (e.g. 'Add a log statement')..."
                            value={aiInstruction}
                            onChange={(e) => setAiInstruction(e.target.value)}
                            disabled={isReadOnly || isAiLoading}
                            style={{ flex: 1, background: 'rgba(0,0,0,0.2)', border: '1px solid var(--surface-border)', borderRadius: '6px', padding: '0.5rem', color: '#fff', outline: 'none' }}
                        />
                        <button
                            className="btn-primary"
                            onClick={handleAiAssist}
                            disabled={isReadOnly || !aiInstruction.trim() || isAiLoading}
                            style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', opacity: (isReadOnly || !aiInstruction.trim() || isAiLoading) ? 0.7 : 1, cursor: (isReadOnly || !aiInstruction.trim() || isAiLoading) ? 'not-allowed' : 'pointer' }}
                        >
                            {isAiLoading ? <ArrowClockwise className="spin" /> : <MagicWand weight="fill" />}
                            ✨ AI Auto Edit
                        </button>
                    </div>
                    
                    <textarea 
                        value={editorCode || ''}
                        onChange={(e) => setEditorCode(e.target.value)}
                        disabled={isReadOnly}
                        style={{
                            flex: 1,
                            width: '100%',
                            background: 'rgba(0,0,0,0.3)',
                            color: '#e2e8f0',
                            fontFamily: '"Fira Code", monospace',
                            fontSize: '0.95rem',
                            border: '1px solid var(--surface-border)',
                            borderRadius: '12px',
                            padding: '1.5rem',
                            resize: 'none',
                            outline: 'none',
                            lineHeight: 1.5,
                            boxShadow: 'inset 0 2px 10px rgba(0,0,0,0.2)'
                        }}
                        spellCheck={false}
                    />

                    {compileOutput && (
                        <div style={{
                            padding: '1rem',
                            background: compileOutput.includes('Error') ? 'rgba(239, 68, 68, 0.1)' : 'rgba(16, 185, 129, 0.1)',
                            border: `1px solid ${compileOutput.includes('Error') ? 'var(--danger-color)' : 'var(--secondary-color)'}`,
                            borderRadius: '12px',
                            color: compileOutput.includes('Error') ? 'var(--danger-color)' : 'var(--secondary-color)',
                            fontFamily: 'monospace',
                            fontSize: '0.9rem',
                            display: 'flex',
                            alignItems: 'flex-start',
                            gap: '0.5rem'
                        }}>
                            <pre style={{ whiteSpace: 'pre-wrap', wordBreak: 'break-all' }}>{compileOutput}</pre>
                        </div>
                    )}
                </div>
            </div>
            
            <style>{`
                .spin { animation: spin 1s linear infinite; }
                @keyframes spin { 100% { transform: rotate(360deg); } }
            `}</style>
        </div>
    );
}
