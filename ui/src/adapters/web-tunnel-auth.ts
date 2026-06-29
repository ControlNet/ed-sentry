import type { HTTPError } from "ky"
import { z } from "zod"
import { DashboardAdapterError, type TunnelLoginResult, type TunnelStatusView } from "./types"

export const tunnelAuthStorageKey = "ed-sentry:tunnel-auth-token"

const tunnelAuthRecordSchema = z
  .object({
    token: z.string().min(1),
    publicHost: z.string(),
    sessionId: z.string(),
  })
  .readonly()

const webErrorResponseSchema = z.object({
  error: z.object({
    code: z.string(),
    message: z.string(),
  }),
})

type TunnelAuthRecord = Readonly<z.infer<typeof tunnelAuthRecordSchema>>

type TunnelAuthBinding = {
  readonly publicHost: string
  readonly sessionId: string
}

type TunnelStatusLoader = () => Promise<TunnelStatusView>

export function clearTunnelAuthToken(): void {
  if (typeof globalThis.sessionStorage === "undefined") {
    return
  }
  globalThis.sessionStorage.removeItem(tunnelAuthStorageKey)
}

export function reconcileTunnelAuth(status: TunnelStatusView): void {
  const record = readTunnelAuthToken()
  if (record === null) {
    return
  }
  const binding = tunnelAuthBinding(status)
  if (
    binding === null ||
    binding.publicHost !== record.publicHost ||
    binding.sessionId !== record.sessionId
  ) {
    clearTunnelAuthToken()
  }
}

export function writeTunnelAuthFromStatus(
  token: string,
  status: TunnelStatusView,
): TunnelLoginResult {
  const binding = tunnelAuthBinding(status)
  if (binding === null) {
    clearTunnelAuthToken()
    return {
      ok: false,
      code: "tunnel_session_unavailable",
      message: "Tunnel login completed, but no active tunnel session is available",
    }
  }
  globalThis.sessionStorage.setItem(tunnelAuthStorageKey, JSON.stringify({ token, ...binding }))
  return { ok: true }
}

export async function configAuthorizationHeader(
  loadTunnelStatus: TunnelStatusLoader,
): Promise<string | null> {
  const record = readTunnelAuthToken()
  if (record === null) {
    return null
  }
  const status = await loadTunnelStatus()
  const binding = tunnelAuthBinding(status)
  if (
    binding === null ||
    binding.publicHost !== record.publicHost ||
    binding.sessionId !== record.sessionId
  ) {
    clearTunnelAuthToken()
    return null
  }
  return ["Bearer", record.token].join(" ")
}

export function isAuthFailureStatus(status: number): boolean {
  return status === 401 || status === 403
}

export async function formatWebHttpError(
  error: HTTPError,
  fallbackMessage: string,
): Promise<DashboardAdapterError> {
  const webError = await parseWebErrorResponse(error.response)
  return new DashboardAdapterError("web", webError?.message ?? fallbackMessage)
}

export async function formatWebResponseError(
  response: Response,
  fallbackMessage: string,
): Promise<DashboardAdapterError> {
  const webError = await parseWebErrorResponse(response)
  return new DashboardAdapterError("web", webError?.message ?? fallbackMessage)
}

export async function parseWebErrorResponse(
  response: Response,
): Promise<{ readonly code: string; readonly message: string } | null> {
  if (response.bodyUsed) {
    return null
  }
  const text = await response.text()
  if (text.trim().length === 0) {
    return null
  }
  const parsedJson = parseJsonText(text)
  const parsed = webErrorResponseSchema.safeParse(parsedJson)
  if (!parsed.success) {
    return null
  }
  return parsed.data.error
}

function readTunnelAuthToken(): TunnelAuthRecord | null {
  if (typeof globalThis.sessionStorage === "undefined") {
    return null
  }
  const stored = globalThis.sessionStorage.getItem(tunnelAuthStorageKey)
  if (stored === null) {
    return null
  }
  const parsedJson = parseJsonText(stored)
  const parsed = tunnelAuthRecordSchema.safeParse(parsedJson)
  if (!parsed.success) {
    clearTunnelAuthToken()
    return null
  }
  return parsed.data
}

function tunnelAuthBinding(status: TunnelStatusView): TunnelAuthBinding | null {
  const publicUrl = status.public_url
  const sessionId = status.session_id
  if (status.kind !== "running" || publicUrl === null || publicUrl === undefined) {
    return null
  }
  if (sessionId === null || sessionId === undefined) {
    return null
  }
  try {
    return {
      publicHost: new URL(publicUrl).host,
      sessionId,
    }
  } catch (error) {
    if (error instanceof TypeError) {
      return null
    }
    throw error
  }
}

function parseJsonText(text: string): unknown {
  try {
    return JSON.parse(text)
  } catch (error) {
    if (error instanceof SyntaxError) {
      return null
    }
    throw error
  }
}
