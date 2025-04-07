import { test, expect } from "@playwright/test";

test("infobox keyevents", async ({ page }) => {
    await page.goto("/")
    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle")

    let row = page.locator("side-bar").locator(".row").nth(0)
    row.click()
    page.waitForURL("/8d0604b5-6be2-4346-a03c-c8fa4f8bb6b9")

    let count_num = page.locator("#infobox").locator("*[data-testid=box]", { hasText: "Counter 1" }).locator("*[data-testid=info]")
    let timer = page.locator("#infobox").locator("*[data-testid=box]").nth(1).locator("*[data-testid=info]")

    // when pressing equal the counter should increase and the timer should start
    let current_count = await count_num.textContent().then((text) => text ? text : "")
    await page.press("body", "Equal")
    await expect(count_num).toHaveText((parseInt(current_count) + 1).toString())
    await expect(timer).not.toHaveText("00:00:00.000")

    // after pressing p the timer should pause and have the same text
    await page.press("body", "KeyP")
    let timerText = await timer.textContent().then((text) => text ? text : "")
    await page.waitForTimeout(100)
    await expect(timer).toHaveText(timerText)

    current_count = await count_num.textContent().then((text) => text ? text : "")
    await page.press("body", "Minus")
    await expect(count_num).toHaveText((parseInt(current_count) - 1).toString())
})

test("load page with selection", async ({ page, isMobile }) => {
    page.goto("/8d0604b5-6be2-4346-a03c-c8fa4f8bb6b9")
    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle")

    if (isMobile) {
        let sidebar = page.locator("side-bar")
        await expect(sidebar).not.toBeInViewport()
    }

    let count_info_box = page.locator("#infobox > div > div").nth(0)
    await expect(count_info_box).toBeVisible()
    await expect(count_info_box).toHaveClass(/rowbox/)

    let count_info = count_info_box.locator("*[data-testid=info]")
    let current_count = await count_info.textContent().then((text) => text ? text : "")
    if (!isMobile) {
        count_info.click()
    } else {
        count_info.tap()
    }
    await expect(count_info).toHaveText((parseInt(current_count) + 1).toString())
})
