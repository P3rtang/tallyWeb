import { test, expect } from '@playwright/test'

const CUSTOM_MESSAGE = 'custom message text'

test('show messages', async ({ page }) => {
    await page.goto('http://localhost:3000/test?topic=notifications')

    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState('networkidle')

    const sendMessageButton = page.getByTestId('send-message')

    await sendMessageButton.click().then(async () => {
        const notification = page.getByTestId('notification')
        await expect(notification).toBeVisible()
        await expect(notification).toContainText('Message')
    })

    const messageInput = page.getByTestId('message-input')

    await messageInput
        .fill(CUSTOM_MESSAGE)
        .then(() => sendMessageButton.click())
        .then(async () => {
            const notification = page.getByTestId('notification').first()
            await expect(notification).toBeVisible()
            await expect(notification).toContainText(CUSTOM_MESSAGE)
        });

    // this message has a timeout and will disappear with a fade
    // therefore this should always run first after load
    // let timoutMessage = notificationBox.locator("dialog", { hasText: "message 4" })
    // await expect(timoutMessage).toBeVisible()
    // await timoutMessage.waitFor({ state: "detached" })
    // await expect(timoutMessage).toBeHidden()

    // let message1 = notificationBox.locator("dialog", { hasText: "message 1" })
    // await expect(message1).toBeVisible()

    // let closeMessage = message1.locator("button")
    // await expect(closeMessage).toBeHidden()
    // message1.hover()
    // await expect(closeMessage).toBeVisible()

    // await closeMessage.click()
    // await expect(message1).toHaveClass("fade-out")
    // await expect(message1).toBeHidden()

    // let errorMessage = notificationBox.locator("dialog", { hasText: "An error occurred" })
    // await expect(errorMessage).toHaveCSS("border", "2px solid rgb(255, 99, 71)")

    // let serverMessage = notificationBox.locator("dialog", { hasText: "Internal server Error" })
    // await expect(serverMessage).toHaveCSS("border", "2px solid rgb(255, 99, 71)")
})
