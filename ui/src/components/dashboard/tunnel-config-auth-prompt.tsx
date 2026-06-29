import { Loader2, ShieldCheck } from "lucide-react"
import { type FormEvent, useState } from "react"
import type { DashboardAdapter } from "@/adapters/dashboard"
import { FieldMessage } from "./config-form-fields"

type TunnelConfigAuthPromptProps = {
  readonly adapter: DashboardAdapter
  readonly onAuthenticated: () => void
}

type LoginState =
  | { readonly status: "idle" }
  | { readonly status: "submitting" }
  | { readonly status: "error"; readonly message: string }

export function TunnelConfigAuthPrompt({
  adapter,
  onAuthenticated,
}: TunnelConfigAuthPromptProps): React.JSX.Element {
  const [password, setPassword] = useState("")
  const [loginState, setLoginState] = useState<LoginState>({ status: "idle" })

  const submit = async (event: FormEvent<HTMLFormElement>): Promise<void> => {
    event.preventDefault()
    const loginTunnel = adapter.loginTunnel
    if (loginTunnel === undefined) {
      setLoginState({ status: "error", message: "Tunnel login is unavailable for this runtime" })
      return
    }
    setLoginState({ status: "submitting" })
    const result = await loginTunnel(password)
    if (result.ok) {
      setPassword("")
      setLoginState({ status: "idle" })
      onAuthenticated()
      return
    }
    setLoginState({ status: "error", message: result.message })
  }

  return (
    <section aria-label="Tunnel config authentication" className="tactical-config-section">
      <h3 className="absolute -top-3 left-4 flex items-center gap-2 bg-surface-panel px-2 text-[10px] font-bold uppercase tracking-widest text-tactical">
        <ShieldCheck aria-hidden="true" className="size-3" />
        Tunnel access
      </h3>
      <form className="mt-2 grid gap-3" onSubmit={(event) => void submit(event)}>
        <p className="text-sm text-slate-400">
          Enter the tunnel config password to unlock SYSTEMS for this browser session.
        </p>
        <label className="text-sm">
          <span className="mb-2 block font-mono text-[10px] uppercase text-slate-500">
            Tunnel config password
          </span>
          <input
            aria-label="Tunnel config password"
            autoComplete="current-password"
            className="tactical-input"
            type="password"
            value={password}
            onChange={(event) => setPassword(event.currentTarget.value)}
          />
        </label>
        {loginState.status === "error" ? (
          <FieldMessage tone="error">{loginState.message}</FieldMessage>
        ) : null}
        <button
          type="submit"
          className="flex w-fit cursor-pointer items-center gap-2 rounded-sm bg-tactical-accent px-5 py-2 text-[10px] font-bold uppercase tracking-widest text-primary-foreground transition-colors hover:bg-primary disabled:cursor-not-allowed disabled:opacity-50"
          disabled={loginState.status === "submitting" || password.trim().length === 0}
        >
          {loginState.status === "submitting" ? (
            <Loader2 aria-hidden="true" className="size-3.5 animate-spin" />
          ) : null}
          Unlock Systems
        </button>
      </form>
    </section>
  )
}
