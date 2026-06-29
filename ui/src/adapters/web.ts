import ky, { HTTPError } from "ky"
import { z } from "zod"
import type { ConfigApiView, EditableConfigUpdate } from "@/adapters/config"
import {
  type AppSnapshot,
  type DashboardAdapter,
  DashboardAdapterError,
  formatAdapterError,
  parseAppSnapshot,
  parseConfigApiView,
  parseTunnelStatusView,
  type TunnelLoginResult,
  type TunnelStatusView,
} from "@/adapters/types"
import { createWebSocketSubscription } from "@/adapters/web-events"
import {
  clearTunnelAuthToken,
  configAuthorizationHeader,
  formatWebHttpError,
  formatWebResponseError,
  isAuthFailureStatus,
  parseWebErrorResponse,
  reconcileTunnelAuth,
  writeTunnelAuthFromStatus,
} from "@/adapters/web-tunnel-auth"

const tunnelLoginResponseSchema = z.object({
  token: z.string().min(1),
})

export type WebDashboardAdapterOptions = {
  readonly baseUrl?: string
  readonly snapshotPath?: string
  readonly configPath?: string
  readonly eventsPath?: string
  readonly tunnelStatusPath?: string
  readonly tunnelStartPath?: string
  readonly tunnelLoginPath?: string
  readonly timeoutMs?: number
}

export function createWebDashboardAdapter(
  options: WebDashboardAdapterOptions = {},
): DashboardAdapter {
  const baseUrl = options.baseUrl ?? globalThis.location.origin
  const snapshotPath = options.snapshotPath ?? "api/snapshot"
  const configPath = options.configPath ?? "api/config"
  const eventsPath = options.eventsPath ?? "/api/events"
  const tunnelStatusPath = options.tunnelStatusPath ?? "api/tunnel/status"
  const tunnelStartPath = options.tunnelStartPath ?? "api/tunnel/start"
  const tunnelLoginPath = options.tunnelLoginPath ?? "api/tunnel/login"
  const timeout = options.timeoutMs ?? 5_000
  const api = ky.create({
    baseUrl,
    timeout,
    retry: 0,
  })

  return {
    mode: "web",
    label: "Web API",
    async loadSnapshot(): Promise<AppSnapshot> {
      try {
        const snapshotPayload = await api.get(snapshotPath).json<unknown>()
        return parseAppSnapshot(snapshotPayload)
      } catch (error) {
        if (error instanceof Error) {
          throw formatAdapterError("web", error)
        }
        throw new DashboardAdapterError("web", "Web adapter failed with a non-Error value")
      }
    },
    async loadConfig(): Promise<ConfigApiView> {
      try {
        const response = await api.get(configPath, {
          ...(await protectedConfigRequestOptions()),
          throwHttpErrors: false,
        })
        if (isAuthFailureStatus(response.status)) {
          clearTunnelAuthToken()
        }
        if (!response.ok) {
          throw await formatWebResponseError(response, "Web config load failed")
        }
        return parseConfigApiView(await response.json<unknown>())
      } catch (error) {
        if (error instanceof Error) {
          throw formatAdapterError("web", error)
        }
        throw new DashboardAdapterError("web", "Web config load failed with a non-Error value")
      }
    },
    async saveConfig(update: EditableConfigUpdate): Promise<ConfigApiView> {
      try {
        const response = await api.put(configPath, {
          json: update,
          ...(await protectedConfigRequestOptions()),
          throwHttpErrors: false,
        })
        if (isAuthFailureStatus(response.status)) {
          clearTunnelAuthToken()
        }
        if (!response.ok) {
          throw await formatWebResponseError(response, "Web config save failed")
        }
        return parseConfigApiView(await response.json<unknown>())
      } catch (error) {
        if (error instanceof Error) {
          throw formatAdapterError("web", error)
        }
        throw new DashboardAdapterError("web", "Web config save failed with a non-Error value")
      }
    },
    async loadTunnelStatus(): Promise<TunnelStatusView> {
      try {
        const status = await loadTunnelStatusFromApi()
        reconcileTunnelAuth(status)
        return status
      } catch (error) {
        if (error instanceof HTTPError) {
          throw await formatWebHttpError(error, "Web tunnel status failed")
        }
        if (error instanceof Error) {
          throw formatAdapterError("web", error)
        }
        throw new DashboardAdapterError("web", "Web tunnel status failed with a non-Error value")
      }
    },
    async startTunnel(): Promise<TunnelStatusView> {
      try {
        const status = parseTunnelStatusView(await api.post(tunnelStartPath).json<unknown>())
        reconcileTunnelAuth(status)
        return status
      } catch (error) {
        if (error instanceof HTTPError) {
          throw await formatWebHttpError(error, "Web tunnel start failed")
        }
        if (error instanceof Error) {
          throw formatAdapterError("web", error)
        }
        throw new DashboardAdapterError("web", "Web tunnel start failed with a non-Error value")
      }
    },
    async loginTunnel(password: string): Promise<TunnelLoginResult> {
      try {
        const loginResponse = await api.post(tunnelLoginPath, {
          json: { password },
          throwHttpErrors: false,
        })
        if (isAuthFailureStatus(loginResponse.status)) {
          clearTunnelAuthToken()
          const webError = await parseWebErrorResponse(loginResponse)
          return {
            ok: false,
            code: webError?.code ?? "tunnel_login_rejected",
            message: webError?.message ?? "Tunnel login credentials were rejected",
          }
        }
        if (!loginResponse.ok) {
          clearTunnelAuthToken()
          throw await formatWebResponseError(loginResponse, "Web tunnel login failed")
        }
        const response = tunnelLoginResponseSchema.parse(await loginResponse.json<unknown>())
        const status = await loadTunnelStatusFromApi()
        return writeTunnelAuthFromStatus(response.token, status)
      } catch (error) {
        clearTunnelAuthToken()
        if (error instanceof HTTPError) {
          throw await formatWebHttpError(error, "Web tunnel login failed")
        }
        if (error instanceof Error) {
          throw formatAdapterError("web", error)
        }
        throw new DashboardAdapterError("web", "Web tunnel login failed with a non-Error value")
      }
    },
    subscribe(onEvent) {
      return createWebSocketSubscription(baseUrl, eventsPath, onEvent)
    },
  }

  async function loadTunnelStatusFromApi(): Promise<TunnelStatusView> {
    return parseTunnelStatusView(await api.get(tunnelStatusPath).json<unknown>())
  }

  async function protectedConfigRequestOptions(): Promise<
    { readonly headers: { readonly Authorization: string } } | Record<string, never>
  > {
    const authorization = await configAuthorizationHeader(loadTunnelStatusFromApi)
    if (authorization === null) {
      return {}
    }
    return { headers: { Authorization: authorization } }
  }
}
