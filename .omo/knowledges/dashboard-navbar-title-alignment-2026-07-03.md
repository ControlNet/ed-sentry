# Dashboard navbar title alignment

- Dashboard shell workspace headings are derived from the active navbar tab label in `ui/src/components/dashboard/dashboard-shell.tsx` as `{activeDefinition.label} Interface`.
- Do not add separate per-tab `title` strings unless the product explicitly wants title copy to diverge from the navbar label.
- Current heading outcomes: `Dashboard Interface`, `Missions Interface`, `Comms Feed Interface`, `Systems Interface`, and `About Interface`.
- E2E assertions that previously expected `Telemetry Interface`, `Events Interface`, or `Config Interface` should use the navbar-aligned names instead.
