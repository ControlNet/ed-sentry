# Frontend content inventory: mission detail surface

- The frontend should not be described as only two content views if that hides mission drill-down needs. Top-level navigation may stay Dashboard/Config, but mission content needs a selectable detail surface inside Dashboard or as a Missions route.
- Existing plan/design mentions first-milestone mission progress and a Mission Table, not an explicit top-level mission detail route.
- Mission modeling knowledge and tests support richer mission detail than the current table displays: destination, issuing/target faction, reward/donation/fine, influence/reputation, wing, expiry, trade commodity/count, origin context, `CargoDepot` collected/delivered/total/progress, and massacre kill progress.
- Current frontend DTO exposes a summary `MissionView` with id/state/kind/display name/factions/destination/accepted/expiry/reward/progress. The current `MissionPanel` renders only a compact table-like summary.
- For future frontend redesign brainstorming, include Mission List plus Mission Detail content. Trade/collection/delivery missions especially need detail fields for commodity, collected, delivered, total count, remaining amount, CargoDepot progress, destination, and related recent events.
