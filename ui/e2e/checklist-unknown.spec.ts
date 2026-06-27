import { expect, test } from "@playwright/test"

test("@todo10-dashboard Checklist visibly renders UNKNOWN state", async ({ page }) => {
  await page.goto("/?mock_state=afk_checklist_unknown")

  const checklist = page.getByRole("region", { name: "Checklist" })

  await expect(page.getByRole("region", { name: "Ship Integrity" })).toHaveCount(0)
  await expect(checklist).toBeVisible()

  const checklistRows = checklist.getByTestId("afk-checklist-row")
  const expectedChecklistRows = [
    ["Hardpoints deployed", "UNKNOWN"],
    ["Engine pips zero", "PASS"],
    ["Cargo loaded", "FAIL"],
  ] as const

  await expect(checklistRows).toHaveCount(expectedChecklistRows.length)
  for (const [index, [label, stateLabel]] of expectedChecklistRows.entries()) {
    await expect(checklistRows.nth(index)).toContainText(label)
    await expect(checklistRows.nth(index)).toContainText(stateLabel)
  }
  await expect(checklist).not.toContainText("Status Flags")
  await expect(checklist).not.toContainText("Status.json")
  await expect(checklist).not.toContainText("Cargo.json")

  await checklist.screenshot({
    path: "../.omo/evidence/afk-checklist-watcher/task-10-checklist-unknown-panel.png",
  })
})
