import { test, expect } from "@playwright/test";

test("homepage has title and links to intro page", async ({ page }) => {
    await page.goto("/")
    await expect(page).toHaveTitle("TallyWeb");
});

test("homepage has account icon and overlay", async ({ page }) => {
    await page.goto("/")
    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle");

    let accountIcon = page.getByTestId("test-account-icon")
    await expect(accountIcon).toBeVisible()
    await expect(accountIcon).toHaveText("U")

    // test overlay on click
    await accountIcon.click()
    await expect(page.getByTestId("test-account-overlay")).toBeVisible()
})

test("sidebar style", async ({ browser, isMobile }) => {
    if (isMobile) {
        return
    }

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
        await page.page.goto("/")
        // make sure the wasm binary is loaded before clicking login
        await page.page.waitForLoadState("networkidle")

        let sidebar = page.page.locator("aside")
        await expect(sidebar).toBeVisible()
        await expect(sidebar).toHaveCSS("position", page.sidebar.position)
        await expect(sidebar).toHaveCSS("--accent", "#66eecc")
    }
})

test("load page with selection", async ({ page, isMobile }) => {
    page.goto("/a7602280-e3af-4d03-ac44-c130886cc59b")
    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle")

    if (isMobile) {
        let sidebar = page.locator("side-bar")
        await expect(sidebar).not.toBeInViewport()
    }

    let count_info_box = page.locator("#infobox > div > div").nth(0)
    await expect(count_info_box).toBeVisible()
    await expect(count_info_box).toHaveClass(/rowbox/)

    let count_info = count_info_box.locator(".info")
    let current_count = await count_info.textContent().then((text) => text ? text : "")
    if (!isMobile) {
        count_info.click()
    } else {
        count_info.tap()
    }
    await expect(count_info).toHaveText((parseInt(current_count) + 1).toString())
})

test("create counter", async ({ page }) => {
    await page.goto("/")
    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle")

    let sidebar = page.locator("side-bar")
    let newCounterButton = sidebar.getByRole("button", { name: "New Counter" })

    await expect(newCounterButton).toBeVisible()
    await newCounterButton.click()

    let treeviewElement = sidebar.locator(".row span").locator("nth=2")
    await expect(treeviewElement).toContainText('Counter 2')
})
