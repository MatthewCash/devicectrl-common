# devicectrl-common

This is the common library for `devicectrl`, providing core data structures for devices and the network API.

devicectrl is a local-first smart home and IoT stack focusing on simplicity, reliability, and speed. I designed it for my own use, it lacks any features that I do not find useful. This project is not complete and will evolve as I add new capabilities.

## Architecture

-   Devices are individual IoT components that control hardware like a switch or a light. They are responsible for receiving attribute update commands from the server and applying them to the hardware they control. Most devices run on an esp32c6.
-   Clients are user-interfacing machines that send attribute update requests to the server. This could be anything from a web interface to a headless keyboard controller.
-   The [server](https://github.com/MatthewCash/devicectrl-server) is the central communication hub for all devices and clients, aggregating update requests and notifications. The server is also responsible for handling scenes and running [automations](https://github.com/MatthewCash/devicectrl-server#Automations).

See [Dependends](#dependents) for implementations of these components.

## Protocols

This crate contains the specification and message enums source for the following protocols:

### simple

A simple JSON-serialized protocol based on TCP means for devices where implementing TLS is impractical.

Once the TCP connection has been established, both sides must send a randomly generated u32 nonce. Nonces are connection-specific and **must** be regenerated for each connection attempt.

```
Client -> Server:
[ u32 nonce ]

Server -> Client:
[ u32 nonce ]
```

After nonces are exchanged, the device will identify itself by sending the [`ServerBoundSimpleMessage::Identify(DeviceId)`](src/protocol/simple.rs) message. The identify message is sent so that the server can select the correct verifying key for the device, ensuring the integrity of future messages sent.

```
Client -> Server:
[ u32 len | data (len bytes) ]
```

Now that the server can verify the device's messages, future messages will be send with an incrementing nonce and signature. The sent must be derived from the nonce received from the other side, incremented _before_ each message. Example: if the server sends initial nonce `22` to the client, the client's next message will contain nonce `23`, `24`, `25`, `...`.

```
Client -> Server / Server -> Client
[ u32 nonce | u32 len | data (len bytes) | 64 byte signature ]
```

The signature should be computed with ECDSA curve P-256 on all of the preceding data, including nonce and data length.

See [`DeviceBoundSimpleMessage`](src/protocol/simple.rs) and [`ServerBoundSimpleMessage`](src/protocol/simple.rs) for valid messages.

### krypton

A TLS-based device protocol.

_WIP_

See [`DeviceBoundKryptonMessage`](src/protocol/krypton.rs) and [`ServerBoundKryptonMessage`](src/protocol/krypton.rs) for valid messages.

### socket

The following three protocols send and receive the same messages, provided by the `socket` protocol enums.

See [`ClientBoundSocketMessage`](src/protocol/socket.rs) and [`ServerBoundSocketMessage`](src/protocol/socket.rs) for valid messages.

#### TCP

A standard tcp stream, protected with TLS 1.3 using client authentication to provide authentication, requiring the client to also provide a certificate trusted by the server.

The TCP server and clients must serialize messages with JSON, delineating messages with a newline (`\n`).

_Note: I plan to switch the message framing from newline-delineated to length-delineated eventually._

#### WebSocket

Nearly identical to the TCP server but using websocket semantics.

A standard tcp stream, protected with TLS 1.3 using client authentication to provide authentication, requiring the client to also provide a certificate trusted by the server.

The WebSocket server and clients must serialize messages with JSON, but since the WebSocket protocol provides framing, each JSON message is sent as a single WebSocket message without appending newlines.

#### HTTP

Messages are sent as POST requests to `/` with the message contents serialized with JSON and placed in the body.

Like the other socket-based servers, the security of this protocol is underpinned by TLS 1.3 using client authentication to provide authentication, requiring the client to also provide a certificate trusted by the server

## Dependents

### [devicectrl-server](https://github.com/MatthewCash/devicectrl-server)

Cental server for processing update requests from clients by sending update commands to devices and relaying state update notifications.

### [devicectrl-input](https://github.com/MatthewCash/devicectrl-input)

Client for sending update requests generated from input events from physical devices like keyboards.

### [devicectrl-fan-controller](https://github.com/MatthewCash/devicectrl-fan-controller)

Device implementation to communicate with FanLamp Pro V2 ceiling fans.

### [devicectrl-esp32-switch](https://github.com/MatthewCash/devicectrl-esp32-switch)

Device implementation for a simple switch running on an esp32c6.

### [devicectrl-esp32-acpi-switch](https://github.com/MatthewCash/devicectrl-esp32-acpi-switch)

Device implementation for a switch managing ACPI power control running on an esp32c6.

### [devicectrl-esp32-dimmable-light](https://github.com/MatthewCash/devicectrl-esp32-dimmable-light)

Device implementation for a simple dimmable light running on an esp32c6.
