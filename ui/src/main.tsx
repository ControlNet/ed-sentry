import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import { App } from "@/App"
import "@/styles.css"

const enableReactDevTools =
  import.meta.env.DEV && import.meta.env.VITE_DISABLE_REACT_DEVTOOLS !== "1"

if (enableReactDevTools) {
  void import("react-grab")
  void import("react-scan")
}

const rootElement = document.getElementById("root")

if (rootElement === null) {
  throw new Error("Root element is missing")
}

createRoot(rootElement).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
