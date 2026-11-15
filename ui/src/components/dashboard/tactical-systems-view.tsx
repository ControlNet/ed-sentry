import type { DashboardAdapter } from "@/adapters/dashboard"
import { ConfigPanel } from "./config-panel"

export function TacticalSystemsView({
  adapter,
}: {
  readonly adapter: DashboardAdapter
}): React.JSX.Element {
  return <ConfigPanel adapter={adapter} />
}
