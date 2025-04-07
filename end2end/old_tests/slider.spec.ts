import { test, expect, Page } from "@playwright/test";

async function loadPage(page: Page) {
    await page.goto("http://localhost:3000/test/slider")

    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle");
}

test("background", async ({ page }) => {
    await loadPage(page)

    let slider = page.locator("#test-background+slider-el")
    await expect(slider).toHaveCSS("background", /rgb(53, 132, 228)*/)
    await expect(slider).toHaveCSS("cursor", "pointer")

    await page.evaluate(() => {
        document.documentElement.style.setProperty("--accent", "#66eecc")
    })
    await expect(slider).toHaveCSS("background", /rgb(102, 238, 204)*/)

    await slider.click()
    expect(await page.isChecked("#test-background+slider-el")).toBeFalsy()
    await expect(slider).toHaveCSS("background", /rgb(204, 204, 204)*/)
})

test("signals", async ({ page }) => {
    await loadPage(page)

    let toggle = page.getByTestId("toggle")
    expect(await page.isChecked("#test-managed+slider-el")).toBeFalsy()
    await toggle.click()
    expect(await page.isChecked("#test-managed+slider-el")).toBeTruthy()
    await toggle.click()
    expect(await page.isChecked("#test-managed+slider-el")).toBeFalsy()
})

test("disable", async ({ page }) => {
    await loadPage(page)

    let slider = page.locator("#test-disable+slider-el")
    let check = page.getByTestId("check-disable")

    expect(await page.isChecked("#test-disable")).toBeTruthy()

    await check.click()
    await expect(slider).toHaveCSS("filter", "brightness(0.6)")
    await slider.isDisabled()

    await check.click()
    await slider.click()
    expect(await page.isChecked("#test-disable")).toBeFalsy()
})

test("on_checked", async ({ page }) => {
    await loadPage(page)


    let slider = page.locator("#test-on_checked + slider-el")
    let colored_div = page.getByTestId("colored_div")

    await expect(colored_div).toHaveCSS("background", /rgb(255, 0, 0)*/)
    await slider.click()
    await expect(colored_div).toHaveCSS("background", /rgb(0, 255, 0)*/)
})
