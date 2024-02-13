import { test, expect } from "@playwright/test";
import { chromium } from 'playwright';

test("homepage has title and links to intro page", async ({ browser }) => {
    // Create a new incognito browser context
    const context = await browser.newContext();
    
    context.addCookies([{ name: "session", value: '{"user_uuid":"9acd920d-d3d6-4353-82c0-cb293f8e8e1f","username":"p3rtang","token":"64e14f27-5fa1-4dcd-9182-b6452cda49cd"}', url: "https://localhost:3000" }])

    // Create a new page inside context.
    const page = await context.newPage();

    await page.goto("http://localhost:3000/");

    await expect(page).toHaveTitle("TallyWeb");
});
