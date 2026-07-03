# Loading screen progress indicators

- `ui/src/components/dashboard/loading-screen.tsx` should use the circular SVG progress ring as the only startup progress indicator.
- The former `.loading-linear-progress` bar duplicated the same `progress` state below the status label and added no separate information.
- Keep `Dashboard startup`, the circular progress value/ring, status text, and detail text; do not reintroduce a second horizontal progress bar unless it conveys different state.
- Regression coverage lives in `ui/e2e/dashboard-smoke.spec.ts` under `@loading-screen`, which asserts `.loading-linear-progress` has count `0`.
