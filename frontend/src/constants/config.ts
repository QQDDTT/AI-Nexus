/**
 * 统一集中管理前端全局规则配置与预设
 */
export const PROVIDER_PRESETS = [
    { id: 'openai', name: 'OpenAI (GPT-4o / GPT-3.5)', baseUrl: 'https://api.openai.com/v1' },
    { id: 'gemini', name: 'Google Gemini', baseUrl: 'https://generativelanguage.googleapis.com/v1beta' },
    { id: 'anthropic', name: 'Anthropic Claude', baseUrl: 'https://api.anthropic.com/v1' },
    { id: 'deepseek', name: 'DeepSeek AI', baseUrl: 'https://api.deepseek.com/v1' },
    { id: 'ollama', name: 'Ollama (本地私有部署)', baseUrl: 'http://localhost:11434/v1' },
    { id: 'custom', name: '自定义服务商 (Custom)', baseUrl: '' },
] as const;

export const CAPABILITY_TIERS = {
    TIER_1_LOGIC: 'Tier-1-Logic',
    TIER_2_BALANCED: 'Tier-2-Balanced',
    TIER_3_FAST: 'Tier-3-Fast',
    MULTIMODAL_VISION: 'Multimodal-Vision',
} as const;

export const DEFAULT_AGENT_PROMPT = 'You are a helpful AI agent.';
export const DEFAULT_PERSONA_PROMPT = 'You are a helpful assistant Persona.';
export const DEFAULT_TONE = 'professional';

export interface GatewayPlatform {
    id: string;
    name: string;
    description: string;
    color: string;
    fields: { key: string; label: string; placeholder: string; type?: 'text' | 'password' }[];
}

export const GATEWAY_PLATFORMS: GatewayPlatform[] = [
    {
        id: 'Telegram',
        name: 'Telegram Bot',
        description: 'Telegram 机器人 (via BotFather API)',
        color: '#3b82f6',
        fields: [
            { key: 'bot_token', label: 'Bot Token', placeholder: '123456789:AAH...', type: 'password' }
        ]
    },
    {
        id: 'Lark',
        name: 'Lark / 飞书',
        description: '飞书/Lark 自建应用 (机器人/事件订阅)',
        color: '#00d6b9',
        fields: [
            { key: 'app_id', label: 'App ID', placeholder: 'cli_a123456789...' },
            { key: 'app_secret', label: 'App Secret', placeholder: 'Sec123456...', type: 'password' },
            { key: 'encrypt_key', label: 'Encrypt Key (可选)', placeholder: 'Event Encryption Key', type: 'password' }
        ]
    },
    {
        id: 'LINE',
        name: 'LINE Messaging API',
        description: 'LINE 官方账号 Messaging API 网关',
        color: '#06C755',
        fields: [
            { key: 'channel_token', label: 'Channel Access Token', placeholder: 'eyJhbGciOi...', type: 'password' },
            { key: 'channel_secret', label: 'Channel Secret', placeholder: 'a1b2c3d4...', type: 'password' }
        ]
    },
    {
        id: 'Slack',
        name: 'Slack App',
        description: 'Slack 应用 (Socket Mode / Web API)',
        color: '#e11d48',
        fields: [
            { key: 'app_token', label: 'App Level Token (xapp-)', placeholder: 'xapp-...', type: 'password' },
            { key: 'bot_token', label: 'Bot User OAuth Token (xoxb-)', placeholder: 'xoxb-...', type: 'password' }
        ]
    },
    {
        id: 'Discord',
        name: 'Discord Bot',
        description: 'Discord 社区机器人 (Gateway Intent)',
        color: '#5865F2',
        fields: [
            { key: 'bot_token', label: 'Bot Token', placeholder: 'MTAx...', type: 'password' }
        ]
    },
    {
        id: 'WeChat',
        name: '微信 / 企业微信',
        description: '微信公众号 / 企业微信应用 / 智能微秘书 Webhook',
        color: '#07c160',
        fields: [
            { key: 'app_id', label: 'App ID / Key', placeholder: 'ww123456... 或 webhook_key' },
            { key: 'app_secret', label: 'App Secret / Token', placeholder: 'Secret Token', type: 'password' }
        ]
    },
    {
        id: 'WhatsApp',
        name: 'WhatsApp Business',
        description: 'WhatsApp Business Cloud API',
        color: '#25D366',
        fields: [
            { key: 'phone_number_id', label: 'Phone Number ID', placeholder: '100609...' },
            { key: 'access_token', label: 'Permanent Access Token', placeholder: 'EAAG...', type: 'password' }
        ]
    },
    {
        id: 'Web',
        name: 'Web / Webhook',
        description: 'Web 小组件 / 自定义 HTTP Webhook Endpoint',
        color: '#10b981',
        fields: [
            { key: 'webhook_url', label: 'Webhook URL / API Path', placeholder: 'https://your-domain.com/webhook' },
            { key: 'api_key', label: 'Secret Key (可选)', placeholder: 'Bearer Secret Token', type: 'password' }
        ]
    }
];

