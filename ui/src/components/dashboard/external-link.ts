import { invoke } from "@tauri-apps/api/core"
import type { MouseEvent } from "react"

export function handleExternalLinkClick(event: MouseEvent<HTMLAnchorElement>): void {
  if (!Reflect.has(globalThis, "__TAURI_INTERNALS__")) {
    return
  }

  event.preventDefault()
  void invoke("open_external_url", { url: event.currentTarget.href })
}
