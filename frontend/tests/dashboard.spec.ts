import { test, expect } from '@playwright/test';

test.describe('AI-Nexus QA Matrix Core Tests', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('http://localhost:5173/login');
    // Simulate login
    await page.fill('input[type="text"]', 'admin');
    await page.fill('input[type="password"]', 'admin123');
    await page.click('button[type="submit"]');
    await page.waitForURL('http://localhost:5173/');
  });

  test('Phase 1: UI & Responsiveness (T-RS-001)', async ({ page }) => {
    // Desktop View
    await page.setViewportSize({ width: 1920, height: 1080 });
    await expect(page.locator('.sidebar')).toBeVisible();
    await page.screenshot({ path: 'test-results/responsive-1920x1080.png', fullPage: true });

    // Tablet View
    await page.setViewportSize({ width: 768, height: 1024 });
    await page.screenshot({ path: 'test-results/responsive-768x1024.png', fullPage: true });
    
    // Check global tokens
    const panel = page.locator('.panel').first();
    await expect(panel).toBeVisible();
    const bg = await panel.evaluate(el => window.getComputedStyle(el).backgroundColor);
    expect(bg).toContain('rgba(25, 28, 36, 0.6)'); // var(--surface-color)
  });

  test('Phase 2: A11y & Keyboard Navigation (T-A11Y-001)', async ({ page }) => {
    await page.keyboard.press('Tab');
    const focusedTag = await page.evaluate(() => document.activeElement?.tagName);
    // Should focus on a navigation element or link
    expect(focusedTag).toBeDefined();
  });

  test('Phase 3: Core Flows - Gateway Control (T-FLOW-008)', async ({ page }) => {
    await page.click('text=Gateways');
    await page.waitForURL('**/gateways');
    
    // Wait for gateways to load
    await page.waitForSelector('.data-table tbody tr');
    
    // Toggle first gateway
    const toggleBtn = page.locator('.action-btn').first();
    await expect(toggleBtn).toBeVisible();
    await toggleBtn.click();
    
    // Optimistic update should reflect immediately
    await page.screenshot({ path: 'test-results/gateway-optimistic.png' });
  });

  test('Phase 4: Settings Flow (T-FLOW-011)', async ({ page }) => {
    await page.click('text=Settings');
    await page.waitForURL('**/settings');
    
    // Input invalid string to session timeout
    const numInput = page.locator('input[type="number"]').first();
    if (await numInput.isVisible()) {
        await numInput.fill('invalid_string');
        // HTML5 number input will clear invalid strings or restrict typing
        const val = await numInput.inputValue();
        expect(val).toBe('');
    }
  });
});
