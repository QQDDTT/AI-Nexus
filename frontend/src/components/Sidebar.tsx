import { Link, useLocation } from 'react-router-dom';
import { useState, useEffect } from 'react';
import { 
    Planet, 
    SquaresFour, 
    ChatTeardropText, 
    Cpu, 
    Coins, 
    Plugs, 
    Gear,
    Wrench,
    Robot,
    Clock,
    UserCircle,
    CaretLeft,
    CaretRight
} from '@phosphor-icons/react';

export default function Sidebar() {
    const location = useLocation();
    const [isCollapsed, setIsCollapsed] = useState(false);

    useEffect(() => {
        document.documentElement.style.setProperty('--sidebar-width', isCollapsed ? '80px' : '320px');
    }, [isCollapsed]);

    return (
        <aside className={`sidebar ${isCollapsed ? 'collapsed' : ''}`}>
            <div className="brand" style={{ display: 'flex', alignItems: 'center', justifyContent: isCollapsed ? 'center' : 'space-between' }}>
                {!isCollapsed && <div style={{ display: 'flex', alignItems: 'center', gap: '10px' }}><Planet weight="fill" /> AI-Nexus</div>}
                {isCollapsed && <Planet weight="fill" />}
                <button onClick={() => setIsCollapsed(!isCollapsed)} style={{ background: 'transparent', border: 'none', color: 'var(--text-secondary)', cursor: 'pointer', display: 'flex' }}>
                    {isCollapsed ? <CaretRight size={20} /> : <CaretLeft size={20} />}
                </button>
            </div>
            <ul className="nav-menu">
                <Link to="/" className={`nav-item ${location.pathname === '/' ? 'active' : ''}`} title={isCollapsed ? "Overview" : ""}>
                    <SquaresFour size={20} /> {!isCollapsed && "仪表盘 Overview"}
                </Link>
                <Link to="/sessions" className={`nav-item ${location.pathname === '/sessions' ? 'active' : ''}`} title={isCollapsed ? "Sessions" : ""}>
                    <ChatTeardropText size={20} /> {!isCollapsed && "活动会话 Sessions"}
                </Link>
                <Link to="/model-router" className={`nav-item ${location.pathname === '/model-router' ? 'active' : ''}`} title={isCollapsed ? "Model Router" : ""}>
                    <Cpu size={20} /> {!isCollapsed && "算力中心 Model Router"}
                </Link>
                <Link to="/token-ledger" className={`nav-item ${location.pathname === '/token-ledger' ? 'active' : ''}`} title={isCollapsed ? "Token Ledger" : ""}>
                    <Coins size={20} /> {!isCollapsed && "账单明细 Token Ledger"}
                </Link>
                <Link to="/gateways" className={`nav-item ${location.pathname === '/gateways' ? 'active' : ''}`} title={isCollapsed ? "Gateways" : ""}>
                    <Plugs size={20} /> {!isCollapsed && "接入网关 Gateways"}
                </Link>
                <Link to="/skills" className={`nav-item ${location.pathname === '/skills' ? 'active' : ''}`} title={isCollapsed ? "Skills" : ""}>
                    <Wrench size={20} /> {!isCollapsed && "技能仓库 Skills"}
                </Link>
                <Link to="/agent-factory" className={`nav-item ${location.pathname === '/agent-factory' ? 'active' : ''}`} title={isCollapsed ? "Agent Factory" : ""}>
                    <Robot size={20} /> {!isCollapsed && "智能体工厂 Agent Factory"}
                </Link>
                <Link to="/personas" className={`nav-item ${location.pathname === '/personas' ? 'active' : ''}`} title={isCollapsed ? "Personas" : ""}>
                    <UserCircle size={20} /> {!isCollapsed && "人格管理 Personas"}
                </Link>
                <Link to="/task-scheduler" className={`nav-item ${location.pathname === '/task-scheduler' ? 'active' : ''}`} title={isCollapsed ? "Task Scheduler" : ""}>
                    <Clock size={20} /> {!isCollapsed && "任务调度 Task Scheduler"}
                </Link>
                <Link to="/settings" className={`nav-item ${location.pathname === '/settings' ? 'active' : ''}`} style={{ marginTop: 'auto' }} title={isCollapsed ? "Settings" : ""}>
                    <Gear size={20} /> {!isCollapsed && "核心配置 Settings"}
                </Link>
            </ul>
        </aside>
    );
}
