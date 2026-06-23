Systems config should autosave ordinary field changes instead of relying on the old manual `Commit Modifications` action.

Autosave behavior:

- Debounce ordinary config edits before calling `adapter.saveConfig`.
- Show pending/saving/saved/error status in the config panel so users know persistence state.
- If a save resolves after the user made newer edits, preserve the newer form values and only update the saved baseline; the next autosave should handle the latest form.

Sensitive token exception:

- Matrix access token replacement and clear-token requests must not autosave.
- The replacement field is write-only and may contain a partial token while the user is typing.
- When token replacement or clear-token is pending, suspend autosave and require the explicit protected save button.
