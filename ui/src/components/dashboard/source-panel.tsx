import { Satellite } from "lucide-react"
import type { AppSnapshot } from "@/adapters/dashboard"
import { Card } from "@/components/ui/card"
import { sourceDetail } from "./dashboard-helpers"
import { PanelHeader } from "./dashboard-status"

export function SourcePanel({ snapshot }: { readonly snapshot: AppSnapshot }): React.JSX.Element {
  return (
    <Card role="region" aria-label="Journal source" className="overflow-hidden rounded-md">
      <PanelHeader
        icon={<Satellite aria-hidden="true" />}
        title="Journal source"
        description={snapshot.journal_source.status_label}
      />
      <div className="grid gap-3 p-4">
        <div className="min-h-[var(--feed-row-min-height)] rounded-md border bg-background p-3">
          <p className="text-sm font-medium">{snapshot.journal_source.status_label}</p>
          <p className="mt-1 break-words font-mono text-xs text-muted-foreground">
            {sourceDetail(snapshot.journal_source.folder, snapshot.journal_source.selected_file)}
          </p>
        </div>
        <div className="grid grid-cols-2 gap-3 text-sm">
          <div className="rounded-md border bg-background p-3">
            <p className="text-muted-foreground">Recent files</p>
            <p className="mt-1 font-mono font-semibold">{snapshot.journal_source.recent_files}</p>
          </div>
          <div className="rounded-md border bg-background p-3">
            <p className="text-muted-foreground">Updated</p>
            <p className="mt-1 truncate font-mono font-semibold">{snapshot.generated_at_display}</p>
          </div>
        </div>
      </div>
    </Card>
  )
}
