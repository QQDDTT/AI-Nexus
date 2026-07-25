# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: tests/dashboard.spec.ts >> AI-Nexus QA Matrix Core Tests >> Phase 2: A11y & Keyboard Navigation (T-A11Y-001)
- Location: tests/dashboard.spec.ts:30:3

# Error details

```
Error: page.goto: net::ERR_CONNECTION_REFUSED at http://localhost:5173/login
Call log:
  - navigating to "http://localhost:5173/login", waiting until "load"

```

# Test source

```ts
  1  | import { test, expect } from '@playwright/test';
  2  | 
  3  | test.describe('AI-Nexus QA Matrix Core Tests', () => {
  4  |   test.beforeEach(async ({ page }) => {
> 5  |     await page.goto('http://localhost:5173/login');
     |                ^ Error: page.goto: net::ERR_CONNECTION_REFUSED at http://localhost:5173/login
  6  |     // Simulate login
  7  |     await page.fill('input[type="text"]', 'admin');
  8  |     await page.fill('input[type="password"]', 'admin123');
  9  |     await page.click('button[type="submit"]');
  10 |     await page.waitForURL('http://localhost:5173/');
  11 |   });
  12 | 
  13 |   test('Phase 1: UI & Responsiveness (T-RS-001)', async ({ page }) => {
  14 |     // Desktop View
  15 |     await page.setViewportSize({ width: 1920, height: 1080 });
  16 |     await expect(page.locator('.sidebar')).toBeVisible();
  17 |     await page.screenshot({ path: 'test-results/responsive-1920x1080.png', fullPage: true });
  18 | 
  19 |     // Tablet View
  20 |     await page.setViewportSize({ width: 768, height: 1024 });
  21 |     await page.screenshot({ path: 'test-results/responsive-768x1024.png', fullPage: true });
  22 |     
  23 |     // Check global tokens
  24 |     const panel = page.locator('.panel').first();
  25 |     await expect(panel).toBeVisible();
  26 |     const bg = await panel.evaluate(el => window.getComputedStyle(el).backgroundColor);
  27 |     expect(bg).toContain('rgba(25, 28, 36, 0.6)'); // var(--surface-color)
  28 |   });
  29 | 
  30 |   test('Phase 2: A11y & Keyboard Navigation (T-A11Y-001)', async ({ page }) => {
  31 |     await page.keyboard.press('Tab');
  32 |     const focusedTag = await page.evaluate(() => document.activeElement?.tagName);
  33 |     // Should focus on a navigation element or link
  34 |     expect(focusedTag).toBeDefined();
  35 |   });
  36 | 
  37 |   test('Phase 3: Core Flows - Gateway Control (T-FLOW-008)', async ({ page }) => {
  38 |     await page.click('text=Gateways');
  39 |     await page.waitForURL('**/gateways');
  40 |     
  41 |     // Wait for gateways to load
  42 |     await page.waitForSelector('.data-table tbody tr');
  43 |     
  44 |     // Toggle first gateway
  45 |     const toggleBtn = page.locator('.action-btn').first();
  46 |     await expect(toggleBtn).toBeVisible();
  47 |     await toggleBtn.click();
  48 |     
  49 |     // Optimistic update should reflect immediately
  50 |     await page.screenshot({ path: 'test-results/gateway-optimistic.png' });
  51 |   });
  52 | 
  53 |   test('Phase 4: Settings Flow (T-FLOW-011)', async ({ page }) => {
  54 |     await page.click('text=Settings');
  55 |     await page.waitForURL('**/settings');
  56 |     
  57 |     // Input invalid string to session timeout
  58 |     const numInput = page.locator('input[type="number"]').first();
  59 |     if (await numInput.isVisible()) {
  60 |         await numInput.fill('invalid_string');
  61 |         // HTML5 number input will clear invalid strings or restrict typing
  62 |         const val = await numInput.inputValue();
  63 |         expect(val).toBe('');
  64 |     }
  65 |   });
  66 | });
  67 | 
```