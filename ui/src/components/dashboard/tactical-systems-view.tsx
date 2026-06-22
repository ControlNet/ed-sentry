import { Settings } from "lucide-react"
import type { DashboardAdapter } from "@/adapters/dashboard"
import { ConfigPanel } from "./config-panel"
import { TacticalPanel } from "./tactical-ui"

export function TacticalSystemsView({
  adapter,
}: {
  readonly adapter: DashboardAdapter
}): React.JSX.Element {
  return (
    <div className="mx-auto max-w-4xl pb-12 animate-in fade-in duration-500">
      <TacticalPanel
        title="System Configuration"
        icon={Settings}
        className="overflow-visible"
        bodyClassName="overflow-visible"
      >
        <ConfigPanel adapter={adapter} />
      </TacticalPanel>
    </div>
  )
}
