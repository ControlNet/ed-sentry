import { expect, test } from "@playwright/test"

test("@todo10-dashboard AFK readiness visibly renders the Unknown state", async ({ page }) => {
  await page.goto("/?mock_state=afk_checklist_unknown")

  const shipStatus = page.getByRole("region", { name: "Ship status and AFK readiness" })
  await expect(shipStatus).toBeVisible()

  const checklistRows = shipStatus.getByTestId("afk-checklist-row")
  const expectedChecklistRows = [
    ["Hardpoints deployed", "Unknown", "unknown"],
    ["Engine pips zero", "Pass", "pass"],
    ["Cargo loaded", "Fail", "fail"],
  ] as const

  await expect(checklistRows).toHaveCount(expectedChecklistRows.length)
  for (const [index, [label, stateLabel, state]] of expectedChecklistRows.entries()) {
    await expect(checklistRows.nth(index)).toContainText(label)
    await expect(checklistRows.nth(index)).toContainText(stateLabel)
    await expect(checklistRows.nth(index)).toHaveAttribute("data-state", state)
  }
  await expect(page.getByTestId("afk-readiness-count")).toHaveText("1 / 3 pass")
  await expect(shipStatus).not.toContainText("Status Flags")

  await shipStatus.screenshot({
    path: "../.omo/evidence/afk-checklist-watcher/task-10-checklist-unknown-panel.png",
  })
})
