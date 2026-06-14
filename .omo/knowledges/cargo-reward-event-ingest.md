# Cargo and reward event ingest

- Promoted on 2026-06-14: `Cargo`, `CollectCargo`, `MarketBuy`, `MarketSell`, and `RedeemVoucher`.
- These events are ingested as typed Journal variants so cargo, trade, and redeemed reward fields are available for future AFK monitor decisions.
- Promotion is intentionally ingest-only: `SessionState` ignores the new variants and replay low-noise regression coverage asserts they do not appear in default stdout.
- `Cargo` stores optional `Vessel`, optional total `Count`, and an `Inventory` list with item name/localised name/count/stolen/mission id when present.
- `MarketBuy`, `MarketSell`, and `RedeemVoucher` keep only the currently useful external Journal fields and avoid introducing state/store assumptions.
- `RedeemVoucher` preserves both singular `Faction` and bounty-style `Factions` breakdown entries with per-faction amounts.
