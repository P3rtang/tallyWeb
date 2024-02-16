import { test, expect } from "@playwright/test";

test("show messages", async ({ page }) => {
    await page.goto("http://localhost:3000/test/messages")

    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle");

    let message1 = page.getByText("message 1")
    await expect(message1).toBeVisible()
})
