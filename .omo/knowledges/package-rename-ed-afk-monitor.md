# Package rename to ed-afk-monitor

- Package and binary name are now `ed-afk-monitor`, matching the repository folder name.
- Rust crate imports use `ed_afk_monitor` because Cargo maps hyphens to underscores.
- Functional source/docs/workflows should have no previous package-name or crate-name references; ignored historical `.omo/evidence`, `.omo/plans`, `.omo/notepads`, and `.omo/boulder.json` may retain old task history.
- Ignored local release artifacts were renamed from previous artifact prefix to `ed-afk-monitor*` for directory-level consistency.
