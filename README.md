# devicectrl-common

Common library for `devicectrl`, providing core data structures for devices and the network API.

Uses `no-std` to support embedded devices but may require `alloc` in the future.

## TODO

-   Extract networking logic from clients and devices into this crate.
-   Represent integer states better by allowing percentages and explicitly specifying allowed ranges.

## Dependents

### [devicectrl-server](https://github.com/MatthewCash/devicectrl-server)

Cental server for processing update requests from clients by sending update commands to devices and relaying state update notifications.

### [devicectrl-input](https://github.com/MatthewCash/devicectrl-input)

Client for sending update requests generated from input events from physical devices like keyboards.

### [devicectrl-fan-controller](https://github.com/MatthewCash/devicectrl-fan-controller)

Device implementation to communicate with FanLamp Pro V2 ceiling fans.

### [devicectrl-esp32-switch](https://github.com/MatthewCash/devicectrl-esp32-switch)

Device implementation for a simple switch running on an esp32c6.

### [devicectrl-esp32-acpi-switch](https://github.com/MatthewCash/devicectrl-esp32-switch)

Device implementation for a switch managing ACPI power control running on an esp32c6.
