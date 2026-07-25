import Header from '../components/Header';
import { DownloadSimple } from '@phosphor-icons/react';
import { useEffect, useState } from 'react';
import { fetchApi } from '../utils/auth';
import { API_ROUTES } from '../constants';

interface LedgerItem {
    time: string;
    user_id: string;
    model: string;
    input_tokens: number;
    output_tokens: number;
    est_cost_usd: number;
}

export default function TokenLedger() {
    const [ledger, setLedger] = useState<LedgerItem[]>([]);

    useEffect(() => {
        fetchApi(API_ROUTES.LEDGER)
            .then(res => res.json())
            .then(data => setLedger(data))
            .catch(err => console.error(err));
    }, []);

    const exportToCSV = () => {
        if (ledger.length === 0) return;
        const headers = ['Time,User ID,Model,Input Tokens,Output Tokens,Est. Cost USD'];
        const rows = ledger.map(item => 
            `"${item.time}","${item.user_id}","${item.model}",${item.input_tokens},${item.output_tokens},${item.est_cost_usd}`
        );
        const csvContent = "data:text/csv;charset=utf-8,\uFEFF" + headers.concat(rows).join('\n');
        const encodedUri = encodeURI(csvContent);
        const link = document.createElement('a');
        link.setAttribute('href', encodedUri);
        link.setAttribute('download', `token_ledger_${new Date().toISOString().slice(0, 10)}.csv`);
        document.body.appendChild(link);
        link.click();
        document.body.removeChild(link);
    };

    return (
        <div className="main-content">
            <Header 
                title="账单明细 Token Ledger" 
                description="查看所有请求的 Token 消耗记录与成本估算。" 
            />

            <section className="panel">
                <div className="panel-header">
                    <span className="panel-title">最近 24 小时明细</span>
                    <button className="btn-outline" onClick={exportToCSV} disabled={ledger.length === 0}>
                        <DownloadSimple size={18} style={{marginRight: '4px'}} /> 导出 CSV
                    </button>
                </div>
                <table className="data-table">
                    <thead>
                        <tr>
                            <th>Time</th>
                            <th>User ID</th>
                            <th>Model</th>
                            <th>Input Tokens</th>
                            <th>Output Tokens</th>
                            <th>Est. Cost</th>
                        </tr>
                    </thead>
                    <tbody>
                        {ledger.length > 0 ? ledger.map((item, i) => (
                            <tr key={i}>
                                <td style={{ color: 'var(--text-secondary)' }}>{item.time}</td>
                                <td>{item.user_id}</td>
                                <td>{item.model}</td>
                                <td style={{ color: 'var(--secondary-color)' }}>{item.input_tokens.toLocaleString()}</td>
                                <td style={{ color: 'var(--primary-color)' }}>{item.output_tokens.toLocaleString()}</td>
                                <td style={{ fontWeight: 600 }}>${item.est_cost_usd.toFixed(3)}</td>
                            </tr>
                        )) : (
                            <tr><td colSpan={6}>Loading...</td></tr>
                        )}
                    </tbody>
                </table>
            </section>
        </div>
    );
}
