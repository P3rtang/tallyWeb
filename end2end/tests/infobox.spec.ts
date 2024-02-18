import { test, expect } from "@playwright/test";

test("infobox keyevents", async ({ page }) => {
    await page.goto("/")
    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle")

    let row = page.locator("side-bar").locator(".row").nth(0)
    row.click()
    page.waitForURL("/a7602280-e3af-4d03-ac44-c130886cc59b")

    let count_num = page.locator("#infobox").locator(".rowbox", { hasText: "Counter 1" }).locator(".info")
    let timer = page.locator("#infobox").locator(".rowbox").nth(1).locator(".info")

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
