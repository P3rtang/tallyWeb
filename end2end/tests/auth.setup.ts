import { test as setup } from '@playwright/test';

const authFile = 'playwright/.auth/user.json';

setup('authenticate', async ({ page }) => {
    // Perform authentication steps. Replace these actions with your own.
    await page.goto('login');

    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle");

    await page.getByLabel('Username').fill('user');
    await page.getByLabel('Password').fill('testuser');
    await page.getByLabel('button-sign-in').click();
    // Wait until the page receives the cookies.
    //
    // Sometimes login flow sets cookies in the process of several redirects.
    // Wait for the final URL to ensure that the cookies are actually set.
    await page.waitForURL('http://localhost:3000/');
    // Alternatively, you can wait until the page reaches a state where all cookies are set.
    // await expect(page.getByRole('button', { name: 'View profile and more' })).toBeVisible();

    // End of authentication steps.

    await page.context().storageState({ path: authFile });
});
