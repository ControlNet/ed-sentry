Systems config must not expose event routing levels as raw numeric inputs.

User-facing meaning:

- `0` = Off: do not route this event.
- `1` = Notify: send a standard notification.
- `2` = Mention: send a notification and mention the configured Matrix user.

The persisted config remains numeric because the Rust runtime uses those values directly. The WebUI should present them as a three-position labeled control so users do not need to memorize what `0`, `1`, and `2` mean. The segment buttons should show only `Off`, `Notify`, and `Mention`; do not add secondary `L0`/`L1`/`L2` labels because they reintroduce numeric jargon and make the control taller.

Use distinct active colors so the selected mode is recognizable at a glance:

- `Off`: muted slate.
- `Notify`: cyan/info.
- `Mention`: orange/tactical accent.

If a persisted legacy value is higher than `2`, the UI should display it as the `Mention` tier because runtime routing treats level `2+` as mention-capable.
