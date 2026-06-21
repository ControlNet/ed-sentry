import { Binary, Database, LayoutDashboard, Settings } from "lucide-react"
import type { DashboardConnectionState } from "@/adapters/dashboard"
import { cn } from "@/lib/utils"

export function ShellNavigation({
  connection,
  activeView,
  onViewChange,
}: {
  readonly connection: DashboardConnectionState
  readonly activeView: ShellView
  readonly onViewChange: (view: ShellView) => void
}): React.JSX.Element {
  return (
    <aside className="border-b bg-muted lg:border-b-0 lg:border-r">
      <div className="flex min-h-[var(--shell-topbar-height)] items-center justify-between gap-3 px-4 lg:min-h-0 lg:flex-col lg:items-stretch lg:px-6 lg:py-6">
        <div className="min-w-0">
          <div className="flex items-center gap-2">
            <Binary aria-hidden="true" className="size-5 text-primary" />
            <p className="font-semibold tracking-normal">ed-sentry</p>
          </div>
          <p className="mt-1 hidden text-xs text-muted-foreground lg:block">
            local monitor console
          </p>
        </div>
        <nav
          aria-label="Primary"
          className="flex items-center gap-1 lg:mt-6 lg:flex-col lg:items-stretch"
        >
          <NavItem
            active={activeView === "dashboard"}
            icon={<LayoutDashboard aria-hidden="true" />}
            label="Dashboard"
            onClick={() => onViewChange("dashboard")}
          />
          <NavItem
            active={activeView === "config"}
            icon={<Settings aria-hidden="true" />}
            label="Config"
            onClick={() => onViewChange("config")}
          />
          <NavItem icon={<Database aria-hidden="true" />} label="Sources" disabled />
        </nav>
        <div className="hidden rounded-md border bg-card p-3 lg:mt-auto lg:block">
          <p className="text-xs font-medium text-muted-foreground">Connection</p>
          <p className="mt-1 truncate text-sm font-semibold">{connection.label}</p>
          <p className="mt-1 line-clamp-2 text-xs text-muted-foreground">{connection.detail}</p>
        </div>
      </div>
    </aside>
  )
}

type NavItemProps = {
  readonly active?: boolean
  readonly disabled?: boolean
  readonly icon: React.ReactElement
  readonly label: string
  readonly onClick?: () => void
}

export type ShellView = "dashboard" | "config"

function NavItem({
  active = false,
  disabled = false,
  icon,
  label,
  onClick,
}: NavItemProps): React.JSX.Element {
  return (
    <button
      type="button"
      disabled={disabled}
      onClick={onClick}
      className={cn(
        "inline-flex h-9 items-center gap-2 rounded-md border px-3 text-sm font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50 lg:w-full lg:justify-start",
        active
          ? "border-input bg-accent text-accent-foreground"
          : "border-transparent text-muted-foreground hover:bg-accent hover:text-accent-foreground",
      )}
    >
      {icon}
      <span className="hidden sm:inline lg:inline">{label}</span>
    </button>
  )
}
