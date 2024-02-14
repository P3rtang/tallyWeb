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

    // test overlay on click
    await accountIcon.click()
    await expect(page.getByTestId("test-account-overlay")).toBeVisible()
})
