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
    <section aria-label="Tunnel config authentication" className="pnl" style={{ maxWidth: 640 }}>
      <h2 className="pt">
        <span>Tunnel access</span>
      </h2>
      <form onSubmit={(event) => void submit(event)}>
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
            onChange={(event) => setPassword(event.currentTarget.value)}
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
          disabled={loginState.status === "submitting" || password.trim().length === 0}
        >
          Unlock Systems
        </button>
      </form>
    </section>
  )
}
