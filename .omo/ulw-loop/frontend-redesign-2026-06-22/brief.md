# Brief: Frontend redesign from reference design

User request: Based on `reference-design/design1.tsx`, redo the current frontend. Do not modify `reference-design/design1.tsx`.

Objective: Rebuild the shared WebUI/Tauri React frontend so the actual application follows the user-approved reference design while preserving live dashboard/config functionality, adapter behavior, and existing privacy boundaries.

Tier: HEAVY. This is a full frontend redesign across shared React/Tauri/WebUI surfaces, multiple TSX/CSS modules, route/view structure, browser QA, and visual fidelity against a reference design.

Hard constraints:
- Never modify `reference-design/design1.tsx`.
- Use the reference design as the visual/content interaction basis, not as a file to edit.
- Preserve the shared frontend architecture for `mock`, `web`, and `tauri` adapters.
- Preserve config editing and write-only Matrix token behavior.
- Preserve mission detail/list requirements from `.omo/knowledges/frontend-redesign-requirements-2026-06-22.md`.
- Do not introduce GUI replay.
- Do not expose raw Matrix tokens, raw Journal lines, or private Journal/chat content.
- Keep code/comments/identifiers in English.
- Reply to the user in Chinese.

Success criteria:

1. `reference-preserved`
   - Scenario: CLI/data surface. Run `git diff -- reference-design/design1.tsx` after implementation.
   - Pass observable: command exits 0 and prints no diff.
   - Expected evidence: `.omo/ulw-loop/evidence/frontend-redesign-reference-preserved.txt`.

2. `browser-redesign`
   - Scenario: Browser use. Run production Vite build/preview, open the real app with Playwright at `http://127.0.0.1:<port>/?adapter=mock`, click tabs `Telemetry`, `Missions`, `Comms Feed`, `Systems`, and capture desktop/tablet/mobile screenshots.
   - Pass observable: each tab renders reference-design-derived UI content, and screenshots are non-empty with no browser console page crash.
   - Expected evidence: `.omo/ulw-loop/evidence/frontend-redesign-browser-qa.txt` plus PNG screenshots under `.omo/ulw-loop/evidence/`.

3. `functional-regression`
   - Scenario: CLI/test surface. Run `pnpm --dir ui typecheck`, `pnpm --dir ui build`, and focused Playwright tests covering dashboard/config/adapters.
   - Pass observable: commands exit 0 without weakening tests.
   - Expected evidence: `.omo/ulw-loop/evidence/frontend-redesign-functional-regression.txt`.

4. `visual-review`
   - Scenario: Browser/visual QA surface. Compare actual screenshots against the rendered reference design intent and run visual QA review for desktop/tablet/mobile.
   - Pass observable: visual QA reports PASS/GOOD or all blocking findings are fixed and rerun.
   - Expected evidence: `.omo/ulw-loop/evidence/frontend-redesign-visual-qa.md`.

5. `final-review`
   - Scenario: Review gate. Run final code/goal/security/QA review against the diff and evidence.
   - Pass observable: reviewer approval has no blocking findings.
   - Expected evidence: `.omo/ulw-loop/evidence/frontend-redesign-final-review.md`.

Manual QA cleanup requirements:
- Stop any preview/dev server started for QA.
- Close any Playwright/browser context started for QA.
- Confirm no QA tmux sessions, bound ports, or temp dirs remain.
