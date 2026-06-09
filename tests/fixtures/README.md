Sanitized Journal Fixture Policy
================================

These fixtures are synthetic, minimized Elite Dangerous Journal samples for parser,
replay, and state tests. They are not copies or excerpts from a real commander log.

Policy:

- Raw Journals must never be committed to this repository.
- `/home/ubuntu/Elite Dangerous` is read-only local input for manual reference only.
- Fixture content must use fake commander, system, faction, ship, mission, and message
  values.
- Do not include personal chat text, carrier names, local machine paths, tokens,
  credentials, or any other sensitive local data.
- Keep each fixture line-delimited JSON, except for the one documented malformed
  line in `journal_malformed_unknown.log`.

Fixture coverage:

- `journal_minimal_start.log`: basic session start, location, drop, inbound text,
  music, and shutdown.
- `journal_combat_bounty.log`: targeting, combat reward, kill bond, cargo ejection,
  and reservoir replenishment.
- `journal_missions.log`: mission list plus accepted, redirected, completed,
  failed, and abandoned mission transitions.
- `journal_damage_fighter.log`: shield, hull, fighter launch/destruction, and death
  events.
- `journal_malformed_unknown.log`: one unknown valid event and exactly one malformed
  line for negative fixture checks.
- `journal_warning_clock.log`: warning-like text and clock edge cases that remain
  valid JSON lines.
