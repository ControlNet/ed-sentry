import { useEffect } from "react"
import { DashboardShell } from "@/components/dashboard/dashboard-shell"
import { AppFrame } from "@/components/dashboard/g1-ui"
import { LoadingScreen } from "@/components/dashboard/loading-screen"
import { useDashboardStore } from "@/store/dashboard-store"

export function App(): React.JSX.Element {
  const snapshot = useDashboardStore((state) => state.snapshot)
  const adapter = useDashboardStore((state) => state.adapter)
  const status = useDashboardStore((state) => state.status)
  const connection = useDashboardStore((state) => state.connection)
  const errorMessage = useDashboardStore((state) => state.errorMessage)
  const start = useDashboardStore((state) => state.start)
  const refresh = useDashboardStore((state) => state.refresh)

  useEffect(() => {
    void start()
  }, [start])

  if (status === "error") {
    return (
      <AppFrame>
        <main className="layer err-wrap">
          <section
            className="pnl"
            aria-label="Dashboard unavailable"
            style={{ width: "min(100%, 560px)" }}
          >
            <h1 className="pt">
              <span>Dashboard unavailable</span>
            </h1>
            <p className="sec2" style={{ margin: "0 0 16px" }}>
              {errorMessage ?? "The active adapter did not return a dashboard snapshot."}
            </p>
            <button type="button" className="btn" onClick={() => void refresh()}>
              Retry
            </button>
          </section>
        </main>
      </AppFrame>
    )
  }

  if (snapshot === null) {
    return <LoadingScreen detail={connection.detail} isTauri={adapter.mode === "tauri"} />
  }

  return (
    <DashboardShell
      snapshot={snapshot}
      adapter={adapter}
      connection={connection}
      isRefreshing={status === "loading"}
      onRefresh={() => void refresh()}
    />
  )
}
