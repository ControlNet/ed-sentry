# OMO Gitignore Runtime vs Knowledge

Do not ignore `.omo/ulw-loop/` wholesale. The directory can contain durable planning and knowledge artifacts such as `brief-*.md`, `notepad-*.md`, `notepads/*.md`, and per-loop `brief.md`, `goals.json`, or `ledger.jsonl`.

Ignore runtime evidence under `.omo/ulw-loop/evidence/` instead. Keep durable knowledge/docs under `.omo/knowledges/`, `.omo/plans/`, `.omo/notepads/`, and non-evidence `.omo/ulw-loop/` paths trackable.
