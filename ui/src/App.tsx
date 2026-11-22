import { useEffect } from "react"
import { DashboardShell } from "@/components/dashboard/dashboard-shell"
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
  const retry = useDashboardStore((state) => state.retry)

  useEffect(() => {
    void start()
  }, [start])

  if (snapshot === null && status !== "error") {
    return <LoadingScreen detail={connection.detail} isTauri={adapter.mode === "tauri"} />
  }

  return (
    <DashboardShell
      snapshot={snapshot}
      errorMessage={errorMessage}
      adapter={adapter}
      connection={connection}
      isRefreshing={status === "loading"}
      onRefresh={() => void (status === "error" ? retry() : refresh())}
    />
  )
}
