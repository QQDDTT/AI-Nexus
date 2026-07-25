/**
 * 统一集中管理全站 Backend API 路由常量
 */
export const API_ROUTES = {
    // Auth
    LOGIN: '/api/auth/login',

    // Dashboard
    DASHBOARD_STATS: '/api/dashboard/stats',
    DASHBOARD_TOKEN_TREND: '/api/dashboard/token-trend',

    // Gateways
    GATEWAYS: '/api/gateways',
    GATEWAY_BY_ID: (id: string) => `/api/gateways/${encodeURIComponent(id)}`,
    GATEWAY_TOGGLE: (id: string) => `/api/gateways/${encodeURIComponent(id)}/toggle`,
    GATEWAY_CONFIG: (id: string) => `/api/gateways/${encodeURIComponent(id)}/config`,

    // Settings
    SETTINGS: '/api/settings',

    // Sessions
    SESSIONS: '/api/sessions',
    SESSION_BY_ID: (id: string) => `/api/sessions/${encodeURIComponent(id)}`,

    // Ledger
    LEDGER: '/api/ledger',

    // Model Router
    MODELS_ROUTING: '/api/models/routing',
    MODELS_PROVIDERS: '/api/models/providers',
    MODEL_PROVIDER_BY_ID: (id: string) => `/api/models/providers/${encodeURIComponent(id)}`,
    MODEL_FAILOVER_TOGGLE: (name: string) => `/api/models/failover/${encodeURIComponent(name)}/toggle`,

    // Skills
    SKILLS: '/api/skills',
    SKILL_COMPILE: '/api/skills/compile',
    SKILL_SAVE_MD: '/api/skills/save_md',
    SKILL_AI_ASSIST: '/api/skills/ai-assist',
    SKILL_BY_ID: (id: string) => `/api/skills/${encodeURIComponent(id)}`,
    SKILL_TOGGLE: (id: string) => `/api/skills/${encodeURIComponent(id)}/toggle`,

    // Agents
    AGENTS: '/api/agents',
    AGENT_BY_ID: (id: string) => `/api/agents/${encodeURIComponent(id)}`,

    // Personas
    PERSONAS: '/api/personas',
    PERSONA_BY_ID: (id: string) => `/api/personas/${encodeURIComponent(id)}`,

    // Triggers / Task Scheduler
    TRIGGERS: '/api/triggers',
    TRIGGER_BY_ID: (id: string) => `/api/triggers/${encodeURIComponent(id)}`,
} as const;
