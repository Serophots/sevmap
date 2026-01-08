# Sevmap - Single-evmap
Derives from [evmap](https://github.com/jonhoo/evmap), built upon [left-right](https://github.com/jonhoo/left-right), `sevmap` is a lock-free, eventually consistent, concurrent single-valued map.

## Deviations from evmap
- The map is single valued, though a multivalued map could be built ontop of the single valued map
- Values in the map are split into a mutable part and an immutable part:
  - The immutable part is allocated only once, as in `evmap`. Once inserted it cannot be mutated in place. Of course you can always insert and remove values in the map.
  - The mutable part is allocated twice, once in the left side and again in the right side. It can be mutated in place in the map: callers can define their own deterministic operations which can be appended to the underlying `left-right` oplog.

## Usage
See the /tests for now
