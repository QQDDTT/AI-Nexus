export const setToken = (token: string) => localStorage.setItem('token', token);
export const getToken = () => localStorage.getItem('token');
export const removeToken = () => localStorage.removeItem('token');

export const fetchApi = async (url: string, options: RequestInit = {}) => {
    const token = getToken();
    const headers = new Headers(options.headers || {});
    if (token) {
        headers.set('Authorization', `Bearer ${token}`);
    }

    const response = await fetch(url, { ...options, headers });
    
    if (response.status === 401) {
        removeToken();
        // If not already on login page, redirect
        if (window.location.pathname !== '/login') {
            window.location.href = '/login';
        }
    }
    
    return response;
};
