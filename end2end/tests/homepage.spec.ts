import { test, expect } from "@playwright/test";

test("homepage has title and links to intro page", async ({ page }) => {
    await page.goto("http://localhost:3000/");
    await expect(page).toHaveTitle("TallyWeb");
});

test("homepage has account icon and overlay", async ({ page }) => {
    await page.goto("http://localhost:3000/");
    let accountIcon = page.getByTestId("test-account-icon")
    await expect(accountIcon).toBeVisible()
    await expect(accountIcon).toHaveText("U")

    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle");
    // test overlay on click
    await accountIcon.click()
    await expect(page.getByTestId("test-account-overlay")).toBeVisible()
})

test("sidebar", async ({ browser }) => {
    let full_size_page = await browser.newContext({ viewport: { width: 1920, height: 1080 } }).then((ctx) => ctx.newPage());
    let small_page = await browser.newContext({ viewport: { width: 1024, height: 576 } }).then((ctx) => ctx.newPage());

    {
        await full_size_page.goto("http://localhost:3000");
        let sidebar = full_size_page.getByTestId("test-sidebar")
        await expect(sidebar).toHaveCSS("position", "relative")
    }
    {
        await small_page.goto("http://localhost:3000");
        let sidebar = small_page.getByTestId("test-sidebar")
        await expect(sidebar).toHaveCSS("position", "fixed")
    }
})
