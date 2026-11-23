import { type FormEvent, useState } from "react"
import type { DashboardAdapter, TunnelLoginResult } from "@/adapters/dashboard"
import { FieldMessage } from "./config-form-fields"

type TunnelConfigAuthPromptProps = {
  readonly adapter: DashboardAdapter
  readonly onAuthenticated: () => void
}

export type TunnelUnlockState =
  | { readonly status: "idle" }
  | { readonly status: "submitting" }
  | { readonly status: "error"; readonly message: string }

export async function runTunnelUnlock(
  loginTunnel: ((password: string) => Promise<TunnelLoginResult>) | undefined,
  password: string,
  callbacks: {
    readonly onStateChange: (state: TunnelUnlockState) => void
    readonly onSuccess: () => void
  },
): Promise<void> {
  if (loginTunnel === undefined) {
    callbacks.onStateChange({
      status: "error",
      message: "Tunnel login is unavailable for this runtime",
    })
    return
  }
  callbacks.onStateChange({ status: "submitting" })
  try {
    const result = await loginTunnel(password)
    if (result.ok) {
      callbacks.onStateChange({ status: "idle" })
      callbacks.onSuccess()
      return
    }
    callbacks.onStateChange({ status: "error", message: result.message })
  } catch (error) {
    const message =
      error instanceof Error && error.message.trim().length > 0
        ? error.message
        : "Tunnel login failed"
    callbacks.onStateChange({ status: "error", message })
  }
}

export function isTunnelUnlockDisabled(state: TunnelUnlockState, password: string): boolean {
  return state.status === "submitting" || password.trim().length === 0
}

type TunnelConfigAuthPromptViewProps = {
  readonly password: string
  readonly loginState: TunnelUnlockState
  readonly onPasswordChange: (password: string) => void
  readonly onSubmit: (event: FormEvent<HTMLFormElement>) => void
}

export function TunnelConfigAuthPromptView({
  password,
  loginState,
  onPasswordChange,
  onSubmit,
}: TunnelConfigAuthPromptViewProps): React.JSX.Element {
  return (
    <section aria-label="Tunnel config authentication" className="pnl" style={{ maxWidth: 640 }}>
      <h2 className="pt">
        <span>Tunnel access</span>
      </h2>
      <form onSubmit={onSubmit}>
        <p className="sec2" style={{ margin: "0 0 4px" }}>
          Enter the tunnel config password to unlock Systems for this browser session.
        </p>
        <label className="frow">
          <span>Tunnel config password</span>
          <input
            aria-label="Tunnel config password"
            autoComplete="current-password"
            className="inp"
            type="password"
            value={password}
            onChange={(event) => onPasswordChange(event.currentTarget.value)}
          />
        </label>
        {loginState.status === "error" ? (
          <FieldMessage tone="error">{loginState.message}</FieldMessage>
        ) : null}
        <button
          type="submit"
          className="btn"
          style={{ marginTop: 14 }}
          aria-busy={loginState.status === "submitting"}
          disabled={isTunnelUnlockDisabled(loginState, password)}
        >
          Unlock Systems
        </button>
      </form>
    </section>
  )
}

export function TunnelConfigAuthPrompt({
  adapter,
  onAuthenticated,
}: TunnelConfigAuthPromptProps): React.JSX.Element {
  const [password, setPassword] = useState("")
  const [loginState, setLoginState] = useState<TunnelUnlockState>({ status: "idle" })

  const submit = async (event: FormEvent<HTMLFormElement>): Promise<void> => {
    event.preventDefault()
    await runTunnelUnlock(adapter.loginTunnel, password, {
      onStateChange: setLoginState,
      onSuccess: () => {
        setPassword("")
        onAuthenticated()
      },
    })
  }

  return (
    <TunnelConfigAuthPromptView
      password={password}
      loginState={loginState}
      onPasswordChange={setPassword}
      onSubmit={(event) => void submit(event)}
    />
  )
}
