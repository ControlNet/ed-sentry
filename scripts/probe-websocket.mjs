#!/usr/bin/env node
import crypto from "node:crypto"
import net from "node:net"

const urlArg = process.argv[2]
if (urlArg === undefined) {
  console.error("Usage: scripts/probe-websocket.mjs ws://127.0.0.1:PORT/api/events")
  process.exit(2)
}

const url = new URL(urlArg)
if (url.protocol !== "ws:") {
  console.error("Only ws:// URLs are supported by this local probe")
  process.exit(2)
}

const socket = net.createConnection({
  host: url.hostname,
  port: Number(url.port),
})
socket.setTimeout(5_000)

const key = crypto.randomBytes(16).toString("base64")
let buffer = Buffer.alloc(0)
let upgraded = false
let messages = 0

socket.on("connect", () => {
  socket.write(
    [
      `GET ${url.pathname}${url.search} HTTP/1.1`,
      `Host: ${url.host}`,
      "Upgrade: websocket",
      "Connection: Upgrade",
      `Sec-WebSocket-Key: ${key}`,
      "Sec-WebSocket-Version: 13",
      `Origin: http://${url.host}`,
      "",
      "",
    ].join("\r\n"),
  )
})

socket.on("data", (chunk) => {
  buffer = Buffer.concat([buffer, chunk])
  if (!upgraded) {
    const headerEnd = buffer.indexOf("\r\n\r\n")
    if (headerEnd === -1) {
      return
    }
    const headers = buffer.subarray(0, headerEnd).toString("utf8")
    if (!headers.startsWith("HTTP/1.1 101 Switching Protocols")) {
      console.error(headers)
      process.exit(1)
    }
    upgraded = true
    buffer = buffer.subarray(headerEnd + 4)
  }
  readFrames()
})

socket.on("timeout", () => {
  console.error("timed out waiting for WebSocket message")
  process.exit(1)
})

socket.on("error", (error) => {
  console.error(error.message)
  process.exit(1)
})

function readFrames() {
  while (buffer.length >= 2) {
    const opcode = buffer[0] & 0x0f
    let length = buffer[1] & 0x7f
    let offset = 2
    if (length === 126) {
      if (buffer.length < 4) {
        return
      }
      length = buffer.readUInt16BE(2)
      offset = 4
    } else if (length === 127) {
      if (buffer.length < 10) {
        return
      }
      const bigLength = buffer.readBigUInt64BE(2)
      if (bigLength > BigInt(Number.MAX_SAFE_INTEGER)) {
        console.error("WebSocket frame too large")
        process.exit(1)
      }
      length = Number(bigLength)
      offset = 10
    }
    if (buffer.length < offset + length) {
      return
    }
    const payload = buffer.subarray(offset, offset + length)
    buffer = buffer.subarray(offset + length)
    if (opcode === 0x1) {
      console.log(payload.toString("utf8"))
      messages += 1
      if (messages >= 1) {
        socket.end()
        process.exit(0)
      }
    } else if (opcode === 0x8) {
      process.exit(messages > 0 ? 0 : 1)
    }
  }
}
