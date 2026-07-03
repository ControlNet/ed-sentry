# Mission Details Lost After Transient Empty Snapshot - 2026-07-03

## Context

User reported that the latest Elite Dangerous journal loads 20 active missions in OD Elite Tracker, but ed-sentry shows only 12 missions with full details and 8 sparse `0/0` entries without useful details.

Latest inspected journal:

- `~/Elite Dangerous/Journal.2026-07-03T033630.01.log`

## Finding

The 8 sparse mission IDs all have complete `MissionAccepted` rows in the latest journal before a transient empty `Missions` snapshot:

- `1059564055` at line 3807
- `1059564058` at line 3808
- `1059564064` at line 3809
- `1059564072` at line 3810
- `1059564074` at line 3811
- `1059564170` at line 3820
- `1059564209` at line 3821
- `1059564219` at line 3822

Then line 3912 contains:

```json
{ "timestamp":"2026-07-03T02:34:47Z", "event":"Missions", "Active":[  ], "Failed":[  ], "Complete":[  ] }
```

The current tracker treats every `Missions.Active` snapshot as fully authoritative, so this empty snapshot removes detailed active missions from memory. A later `Missions` snapshot at line 3945 reintroduces those 8 mission IDs only as sparse active rows with `MissionID`, generic `Name`, `PassengerMission`, and `Expires`, so target faction, localized name, reward, and kill count are no longer available.

The 12 missions that still have details are `MissionAccepted` events that occur after line 3945, so they are not affected by the destructive empty snapshot.

## Implication

This is not a parser field-shape issue for these 8 missions, and it is not a bounded-history issue: the complete `MissionAccepted` rows are present in the selected latest file itself. The root behavior is destructive handling of transient or partial `Missions.Active` snapshots.

## Fix Direction

Prefer preserving an archive of recently removed active/redirected missions keyed by `MissionID`, then restoring details if a later active snapshot reintroduces the same ID. A regression should replay:

1. Complete `MissionAccepted` rows for active massacre missions.
2. A transient `Missions` event with `Active: []`.
3. A later sparse `Missions.Active` list with the same IDs.
4. Assert mission details such as localized name, target faction, reward, and kill count remain available.
