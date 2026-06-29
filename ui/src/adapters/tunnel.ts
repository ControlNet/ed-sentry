import { z } from "zod"

export const tunnelStatusKinds = [
  "disabled",
  "start",
  "starting",
  "running",
  "error",
  "unsupported",
] as const
export const tunnelProviders = ["cloudflare_quick", "ssh"] as const

export const tunnelStatusViewSchema = z.object({
  kind: z.enum(tunnelStatusKinds),
  status_label: z.string(),
  provider: z.enum(tunnelProviders),
  provider_label: z.string(),
  session_id: z.string().nullable().optional(),
  message: z.string().nullable().optional(),
  public_url: z.string().nullable().optional(),
  checked_at: z.string().nullable().optional(),
  checked_at_display: z.string().nullable().optional(),
  retryable_error: z.boolean(),
})

export type TunnelStatusKind = (typeof tunnelStatusKinds)[number]
export type TunnelProvider = (typeof tunnelProviders)[number]
export type TunnelStatusView = Readonly<z.infer<typeof tunnelStatusViewSchema>>
export type TunnelLoginResult = Readonly<
  | {
      readonly ok: true
    }
  | {
      readonly ok: false
      readonly code: string
      readonly message: string
    }
>

export function parseTunnelStatusView(payload: unknown): TunnelStatusView {
  return tunnelStatusViewSchema.parse(payload)
}
