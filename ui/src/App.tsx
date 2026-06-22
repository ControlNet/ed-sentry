import { useEffect } from "react"
import { DashboardShell } from "@/components/dashboard/dashboard-shell"
import { LoadingScreen } from "@/components/dashboard/loading-screen"
import { Button } from "@/components/ui/button"
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
      <main className="flex min-h-[100dvh] items-center justify-center bg-background p-6 text-foreground">
        <section className="w-full max-w-md rounded-lg border bg-card p-5">
          <h1 className="text-lg font-semibold tracking-normal">Dashboard unavailable</h1>
          <p className="mt-2 text-sm text-muted-foreground">
            {errorMessage ?? "The active adapter did not return a dashboard snapshot."}
          </p>
          <Button type="button" className="mt-4" onClick={() => void refresh()}>
            Retry
          </Button>
        </section>
      </main>
    )
  }

  if (snapshot === null) {
    return <LoadingScreen detail={connection.detail} />
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
