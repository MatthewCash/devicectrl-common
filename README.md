# devicectrl-common

Common library for `devicectrl`, providing core data structures for devices and the network API.

Uses `no-std` to support embedded devices but may require `alloc` in the future.

## TODO

- Extract networking logic from clients and devices into this crate.
- Represent integer states better by allowing percentages and explicitly specifying allowed ranges.
