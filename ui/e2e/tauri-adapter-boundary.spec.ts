import { expect, test } from "@playwright/test"
import {
  createTauriDashboardAdapter,
  type DashboardAdapterEvent,
  type EditableConfigUpdate,
} from "@/adapters/dashboard"
import { mockDashboardSnapshot } from "@/adapters/mock-data"
import { mockConfigView } from "./adapter-boundary-fixtures"

test("tauri adapter parses loaded snapshots and reports malformed stream payloads", async () => {
  const snapshotListeners: ((payload: unknown) => void)[] = []
  const events: DashboardAdapterEvent[] = []
  const adapter = createTauriDashboardAdapter({
    loadSnapshot: async () => mockDashboardSnapshot,
    loadConfig: async () => mockConfigView(),
    saveConfig: async (_update: EditableConfigUpdate) => mockConfigView(),
    listenSnapshot(onSnapshot) {
      snapshotListeners.push(onSnapshot)
      return () => {
        snapshotListeners.length = 0
      }
    },
  })

  await expect(adapter.loadSnapshot()).resolves.toEqual(mockDashboardSnapshot)
  const unsubscribe = subscribeForTest(adapter, events)
  const snapshotListener = snapshotListeners[0]
  if (snapshotListener === undefined) {
    throw new Error("Tauri test transport did not register a snapshot listener")
  }
  snapshotListener({ generated_at: "invalid" })
  unsubscribe()

  expect(events).toContainEqual({
    type: "connection",
    connection: {
      status: "degraded",
      label: "Desktop payload ignored",
      detail: expect.stringContaining("Invalid input"),
      checkedAtDisplay: null,
    },
  })
})

function subscribeForTest(
  adapter: { readonly subscribe?: (onEvent: (event: DashboardAdapterEvent) => void) => () => void },
  events: DashboardAdapterEvent[],
): () => void {
  if (adapter.subscribe === undefined) {
    throw new Error("Adapter does not expose a subscription")
  }
  return adapter.subscribe((event) => events.push(event))
}
