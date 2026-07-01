# ABOUT page implementation

- The ABOUT workspace is an in-shell dashboard tab, not a separate router path.
- `ui/src/components/dashboard/tactical-about-view.tsx` owns the page content and reuses `TacticalPanel` and `DataRow` from the existing tactical dashboard UI primitives.
- `ui/src/components/dashboard/dashboard-shell.tsx` registers the `about` workspace tab with the `Info` icon and renders `TacticalAboutView`.
- The page uses project metadata requested by the project owner: build version in `<Cargo.toml package version>-<latest git commit date YYYYMMDD>` format, author `CMDR ControlNet` linked to `https://inara.cz/elite/cmdr/78197/`, source repository `https://github.com/ControlNet/ed-sentry`, and `GNU Affero General Public License v3.0`.
