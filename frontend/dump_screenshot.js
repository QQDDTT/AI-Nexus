import { chromium } from '@playwright/test';

(async () => {
  const browser = await chromium.launch();
  const page = await browser.newPage();
  
  await page.goto('http://localhost:5173/login');
  
  // Try to login if needed, or just set token
  await page.evaluate(() => {
    localStorage.setItem('token', 'stub');
  });
  
  await page.goto('http://localhost:5173/personas');
  await page.waitForTimeout(2000);
  
  await page.screenshot({ path: '/home/nick/.gemini/antigravity-ide/brain/c7fc441d-20c9-4673-bbe3-0e1aa20ed2a1/scratch/screenshot.png' });
  
  await browser.close();
})();
