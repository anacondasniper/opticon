# opticon-firmware

C (ESP-IDF), runs on each ESP32-P4 doorbell. Adapted from Espressif's
`esp-webrtc-solution` doorbell demo — not written from scratch.

**Does:** camera capture + H.264 encode, on-board person/motion detection,
ring-button GPIO, IR-cut + IR-LED day/night control, two-way audio, WebRTC peer,
signaling to the server, ring/motion events, optional SD-card clips.

**What's actually yours to edit** (the rest comes from the demo):
`main/settings.h` (pins, sensor, server URL, device id), `ir_control.c`,
`button.c`, `events.c`.

**Gotchas:** use the OV5647 sensor config; keep the ring button off GPIO35
(conflicts with RMII_TXD1 when Ethernet is enabled).

> Needs the physical board to test. Buildable/readable now, testable once the
> hardware arrives — so this layer comes last.
