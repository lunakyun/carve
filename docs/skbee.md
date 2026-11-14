# SkBee parity

Status of SkBee features in Carve. Syntax details live in `docs/syntax/`.

Legend: ✅ done, 🟡 partial, ⛔ blocked.

## Utilities

Details: [syntax/skbee-utilities.md](syntax/skbee-utilities.md).

- ✅ Random UUID — `random uuid`
- ✅ Number with commas — `n with commas [and N decimal places]`
- ✅ Round to decimals — `n rounded to N decimal places`
- ✅ Abbreviated numbers — `abbreviated n` (1.5K, 2M)
- ✅ Roman numerals — `roman numeral of n`, `number from roman numeral "XIV"`
- ✅ Hex numbers — `hex of n`, `number from hex "ff"`
- 🟡 Text to JSON — `json of x` quotes and escapes one value; no JSON objects or parsing
- ✅ Client brand — `client brand of player`
- ✅ Client locale — `locale of player`
- ✅ View distance — `view distance of player`
- ✅ Pose — `pose of player`
- ✅ Main hand — `main hand of player`
- ✅ World of, UUID of, ping — already in the base (`world of`, `uuid of`, `ping of`)
- ⛔ Virtual furnace — Pumpkin's plugin API has no way to run a furnace without a block
