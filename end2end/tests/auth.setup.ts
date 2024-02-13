import { test as setup, expect } from '@playwright/test';

const authFile = 'playwright/.auth/user.json';

setup('authenticate', async ({ page }) => {
    // Perform authentication steps. Replace these actions with your own.
    await page.goto('localhost:3000/login');
    await page.getByLabel('username').fill('user');
    await page.getByLabel('password').fill('testuser');
    await page.getByRole('button', { name: 'Sign in' }).click();
    // Wait until the page receives the cookies.
    //
    // Sometimes login flow sets cookies in the process of several redirects.
    // Wait for the final URL to ensure that the cookies are actually set.
    await page.waitForURL('localhost:3000');
    // Alternatively, you can wait until the page reaches a state where all cookies are set.
    // await expect(page.getByRole('button', { name: 'View profile and more' })).toBeVisible();

// End of authentication steps.

await page.context().storageState({ path: authFile });
});
