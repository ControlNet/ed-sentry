# Mission active-only display

The mission tracker keeps `completed_count` and `total_count` in the serialized mission snapshot for schema compatibility, but user-facing mission count text should show only the current active mission count.

Implementation points:

- Backend source: `src/app/missions.rs`, `MissionListView::from_tracker`.
- `status_label` is now `active_count.to_string()` so a snapshot with 20 active missions displays `20`, not `110/20 active` or `130`.
- The tactical mission summary panel renders `snapshot.missions.status_label` in its header badge, so Telemetry also shows the active-only number.
- The tactical mission summary panel must never render `TOTAL {snapshot.missions.total_count}`.
- Mock dashboard fixtures use active-only status labels (`"2"`, `"0"`) so browser/e2e surfaces match production.

Regression coverage:

- `tests/mission_tracker.rs::mission_list_view_status_label_reports_active_count_only` locks the backend label while preserving internal completed/total fields.
- `ui/e2e/dashboard-smoke.spec.ts` asserts the mission progress region does not show `TOTAL` or the old `1/2 active` format.

Manual QA surface used:

- Mock WebUI preview at `http://127.0.0.1:4173/`.
- Telemetry page mission progress region showed active mission rows and the active-only header badge (`"2"`) with no `TOTAL` or old completed/active ratio.
- Missions page `Mission Directory` badge showed only `"2"`.
