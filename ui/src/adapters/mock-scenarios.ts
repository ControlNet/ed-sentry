import { mockDashboardSnapshot } from "@/adapters/mock-data"
import type { AppSnapshot } from "@/adapters/types"

export const emptyMockDashboardSnapshot = {
  ...mockDashboardSnapshot,
  missions: {
    ...mockDashboardSnapshot.missions,
    active_count: 0,
    completed_count: 0,
    total_count: 0,
    status_label: "No tracked missions",
    items: [],
  },
  event_feed: [],
} satisfies AppSnapshot

export const longFeedMockDashboardSnapshot = {
  ...mockDashboardSnapshot,
  event_feed: Array.from({ length: 60 }, (_, index) => {
    const eventNumber = index + 1
    const eventLabel = String(eventNumber).padStart(2, "0")
    const timestamp = new Date(Date.UTC(2026, 5, 20, 13, index, 0)).toISOString()
    return {
      id: `mock-long-event-${eventLabel}`,
      source: "notification",
      event_type: "long_feed_fixture",
      level: eventNumber % 10 === 0 ? 2 : 1,
      summary: `Long feed event ${eventLabel}`,
      timestamp,
      timestamp_display: timestamp.slice(11, 19),
    }
  }),
} satisfies AppSnapshot

export const afkChecklistUnknownMockDashboardSnapshot = {
  ...mockDashboardSnapshot,
  afk_checklist: {
    rows: mockDashboardSnapshot.afk_checklist.rows.map((row) =>
      row.id === "hardpoints_deployed"
        ? {
            ...row,
            detail: "Status Flags are unavailable",
            state: "unknown",
            source: "unknown",
          }
        : row,
    ),
  },
} satisfies AppSnapshot

export const privatePathMockDashboardSnapshot = {
  ...mockDashboardSnapshot,
  journal_source: {
    ...mockDashboardSnapshot.journal_source,
    folder: "/home/private-journal-root/Elite Dangerous",
    selected_file: "Journal.private.2036-01-02.log",
  },
} satisfies AppSnapshot

export const serviceStatusesMockDashboardSnapshot = {
  ...mockDashboardSnapshot,
  matrix: {
    ...mockDashboardSnapshot.matrix,
    kind: "running",
    status_label: "Running",
    message: "Matrix relay connected",
  },
  web: {
    ...mockDashboardSnapshot.web,
    kind: "disabled",
    status_label: "Disabled",
    message: "Web interface disabled by config",
  },
} satisfies AppSnapshot

export const webUrlMockDashboardSnapshot = {
  ...mockDashboardSnapshot,
  web: {
    ...mockDashboardSnapshot.web,
    kind: "running",
    status_label: "Running",
    message: null,
    bind_address: "0.0.0.0:8765",
    url: "http://0.0.0.0:8765",
  },
} satisfies AppSnapshot

export const webLanUrlMockDashboardSnapshot = {
  ...mockDashboardSnapshot,
  web: {
    ...mockDashboardSnapshot.web,
    kind: "running",
    status_label: "Running",
    message: null,
    bind_address: "192.168.50.10:8765",
    url: "http://192.168.50.10:8765",
  },
} satisfies AppSnapshot
