import { describe, expect, it } from "vitest"
import { feedSignature, freshEventIds, nextFlash } from "./live-updates"

const feed = (...ids: string[]) => ids.map((id) => ({ id }))

describe("nextFlash", () => {
  it("starts with the first flash class and alternates so each change restarts the animation", () => {
    expect(nextFlash("")).toBe("fx-a")
    expect(nextFlash("fx-a")).toBe("fx-b")
    expect(nextFlash("fx-b")).toBe("fx-a")
  })
})

describe("freshEventIds", () => {
  it("returns events missing from the previous feed", () => {
    const fresh = freshEventIds(new Set(["e1", "e2"]), feed("e4", "e3", "e2", "e1"))
    expect([...fresh]).toEqual(["e4", "e3"])
  })

  it("returns nothing when the feed only dropped old events", () => {
    expect(freshEventIds(new Set(["e3", "e2", "e1"]), feed("e3", "e2")).size).toBe(0)
  })

  it("returns nothing for an unchanged feed", () => {
    expect(freshEventIds(new Set(["e2", "e1"]), feed("e2", "e1")).size).toBe(0)
  })
})

describe("feedSignature", () => {
  it("changes when the event ids or their order change", () => {
    expect(feedSignature(feed("e2", "e1"))).toBe(feedSignature(feed("e2", "e1")))
    expect(feedSignature(feed("e3", "e2"))).not.toBe(feedSignature(feed("e2", "e1")))
    expect(feedSignature(feed("e1", "e2"))).not.toBe(feedSignature(feed("e2", "e1")))
  })
})
