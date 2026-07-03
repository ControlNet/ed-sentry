# Dashboard shared kill progress header

- Dashboard `Active Missions` header kill progress is computed in `ui/src/components/dashboard/tactical-telemetry-summary.tsx`, not in parser/backend DTOs.
- It follows the ODEliteTracker-style shared massacre model: group active massacre missions by `target_faction`, then by `issuing_faction`; issuer-group requirements and remaining kills are summed, while each target-faction stack contributes the maximum issuer-group total/remaining because one real kill can progress one mission per issuing faction.
- Global dashboard progress sums independent target-faction stacks: `total = sum(max(required per issuer))`, `remaining = sum(max(remaining per issuer))`, `completed = total - remaining`.
- ETA uses the same rate source as Combat Analytics: `snapshot.session.kill_total_rate_per_hour.value`. Format uses rounded minutes without a trailing `left`, with `Complete` at zero remaining and `-` when rate is zero.
- Header pills use the default `TacticalBadge` visual language: active mission count renders as `ACTIVE <count>`, while `Kills` and `ETA` render as same-height horizontal chips so the kill progress text and mini progress bar do not make the header taller.
- `ProgressBar` now exposes `role="progressbar"` with `aria-valuemin`, `aria-valuemax`, and `aria-valuenow`; mission-header e2e asserts this so the mini bar cannot disappear while text still passes.
- `TacticalPanel` header wraps on narrow screens and moves `rightElement` to a full-width second row below `sm`; this preserves the full `Active Missions` title at 375px while keeping count/Kills/ETA chips visible.
- Regression coverage lives in `ui/e2e/mission-header-progress.spec.ts`; avoid adding more checks to the already oversized `dashboard-smoke.spec.ts`.
