import { test, expect, Locator } from '@playwright/test'

const CUSTOM_MESSAGE = 'custom message text'

test('show messages', async ({ page, isMobile }) => {
    await page.goto('http://localhost:3000/test?topic=notifications')

    // make sure the wasm binary is loaded before clicking login
    await page.waitForLoadState('networkidle')

    isMobile && await page.getByLabel("toggle sidebar").first().click()


    const sendMessageButton = page.getByTestId('send-message')

    await sendMessageButton.click().then(async () => {
        const notification = page.getByTestId('notification');
        await expect(notification).toBeVisible();
        await expect(notification).toContainText('Message');
        return closeMessage(notification, {isMobile});
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

})

const closeMessage = async (notification: Locator, config) => {
    (config.isMobile ? notification.click() : notification.hover()).then(
        () => notification.getByRole("button").click()
    )
}
