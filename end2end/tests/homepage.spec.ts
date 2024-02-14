import { test, expect } from "@playwright/test";

test("homepage has title and links to intro page", async ({ page }) => {
    await page.goto("http://localhost:3000/");
    await expect(page).toHaveTitle("TallyWeb");
});

test("homepage has account icon and overlay", async ({ page }) => {
    await page.goto("http://localhost:3000/");
    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle");

    let accountIcon = page.getByTestId("test-account-icon")
    await expect(accountIcon).toBeVisible()
    await expect(accountIcon).toHaveText("U")

    // test overlay on click
    await accountIcon.click()
    await expect(page.getByTestId("test-account-overlay")).toBeVisible()
})

test("sidebar style", async ({ browser }) => {
    let pages = [
        {
            page: await browser.newContext({ viewport: { width: 1920, height: 1080 } }).then((ctx) => ctx.newPage()),
            sidebar: { position: "relative" }
        },
        {
            page: await browser.newContext({ viewport: { width: 1024, height: 576 } }).then((ctx) => ctx.newPage()),
            sidebar: { position: "fixed" }
        },
    ];

    for (const page of pages) {
        await page.page.goto("http://localhost:3000")
        // make sure the wasm binary is loaded before clicking login
        await page.page.waitForLoadState("networkidle")

        let sidebar = page.page.locator("side-bar")
        await expect(sidebar).toBeVisible()
        await expect(sidebar).toHaveCSS("position", page.sidebar.position)
    }
})

test("create counter", async ({ page }) => {
    await page.goto("http://localhost:3000")
    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle")

    let sidebar = page.locator("side-bar")
    let newCounterButton = sidebar.getByRole("button", { name: "New Counter" })

    await expect(newCounterButton).toBeVisible()
    await newCounterButton.click()

    let treeviewElement = sidebar.locator(".row span").locator("nth=0")
    await expect(treeviewElement).toContainText('Counter 1')
})
