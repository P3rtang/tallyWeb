import { test, expect } from "@playwright/test";

test("show messages", async ({ page }) => {
    await page.goto("http://localhost:3000/test/messages")

    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState("networkidle");

    let notificationBox = page.locator("notification-box")
    await expect(notificationBox).toBeVisible()

    // this message has a timeout and will disappear with a fade
    // therefore this should always run first after load
    let timoutMessage = notificationBox.locator("dialog", { hasText: "message 4" })
    await expect(timoutMessage).toHaveClass("fade-out")
    await expect(timoutMessage).toBeHidden()

    let message1 = notificationBox.locator("dialog", { hasText: "message 1" })
    await expect(message1).toBeVisible()

    let closeMessage = message1.locator("button")
    await expect(closeMessage).toBeHidden()
    message1.hover()
    await expect(closeMessage).toBeVisible()

    await closeMessage.click()
    await expect(message1).toHaveClass("fade-out")
    await expect(message1).toBeHidden()

    let errorMessage = notificationBox.locator("dialog", { hasText: "error" })
    await expect(errorMessage).toHaveCSS("border", "2px solid rgb(255, 99, 71)")
})
