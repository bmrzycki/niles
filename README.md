# Niles

A one-page web controller for a Niles ZR-6 multi-zone receiver. Open it from Safari on an iPhone, pick a room, pick an input, set the volume, and later turn it off.

The server is one Rust binary. It runs on the Raspberry Pi that has the USB-to-serial adapter. The phone only talks HTTP. There is no JSON API, no login, and no status polling.

## Tested with

- Alpine Linux 3.24, aarch64
- Raspberry Pi 4, 2 GiB RAM
- Niles ZR-6, chassis firmware v3.26
- FTDI USB-to-serial adapter
- Custom converter cable from that adapter to the ZR-6 3.5 mm serial jack

The Pi GPIO UART is 3.3 V. The ZR-6 jack is RS-232, so use a USB-to-RS-232 adapter. Wiring for the 3.5 mm plug is in [protocol.md](protocol.md). The process user needs read-write access to the device node, usually `/dev/ttyUSB0`.

## Screen

Room tabs are one row across the top, sorted by floor number then name. The room is the first line. The floor number, `0` or greater, is a smaller line under it. Under the tabs is a compact row of inputs. Vol −, Vol +, and Mute fill the middle of the screen. Off and All off stay at the bottom on every tab. All off is red because it turns off every zone on the chassis.

The phone remembers the open tab and the last input tapped there. Off clears that room's highlight. All off clears every room. Mute toggles, and the page does not know whether the zone is muted. Selecting an input is how a zone turns on. Holding Vol − or Vol + repeats.

The page follows `prefers-color-scheme` and is laid out for one phone screen, including the home-indicator inset.

Safari is told this is a standalone web app. The icon is a house with a music note in the middle. It is a square, full-bleed image with hard edges and no baked-in shadow or rounded mask, so Safari can apply its own icon shape.

On iPhone or iPad: Share, then Add to Home Screen. It opens without the Safari toolbar, under the name Niles.

## Run

From the project directory, with `niles.toml` beside the working directory:

```sh
./target/release/niles
```

Then open `http://<pi>:8080` from the phone. The listen address in the sample config is `0.0.0.0:8080`. Anyone on the LAN can reach it.

```sh
./target/release/niles --debug
./target/release/niles --debug /path/to/niles.toml
```

`--debug` does not open the serial port. Known buttons still return success, so the page can be tried with the receiver unplugged. An unknown room or input is still a 404.

A git label is logged at startup and shown under the room tabs. It ends in `-dirty` when `./version.sh` saw uncommitted or untracked files.

## Build

Check out this repo on the machine that has the USB-to-serial adapter, then build there. Alpine 3.24's `rust` package is enough. `apk add cargo` pulls in `rustc` and the system linker. The binary links against that system's musl.

`./version.sh` writes `src/version.rs` from `git describe --tags --always`. The label ends in `-dirty` when the work tree has uncommitted or untracked changes. That file is not committed, so run the script in the checkout before building.

On a 2 GiB Pi the first release build is tight. If it is killed, rerun it with one job.

```sh
apk add cargo
./version.sh
cargo build --release
```

```sh
CARGO_BUILD_JOBS=1 cargo build --release
```

## Config

`niles.toml` maps protocol numbers to the labels on the phone. Each zone names the room and gives its floor as an integer, `0` or greater. That number is the small line under the room, and tabs are sorted by it, then by room name. Zone numbers are only the address sent to the receiver.

```toml
listen = "0.0.0.0:8080"
serial = "/dev/ttyUSB0"

[zone.1]
name = "Master"
floor = 1

[zone.2]
name = "Study"
floor = 1

[zone.3]
name = "Garage"
floor = 1

[zone.4]
name = "Game"
floor = 2

[zone.5]
name = "Balcony"
floor = 2

[zone.6]
name = "Patio"
floor = 1

[input]
3 = "Bluetooth"
4 = "TV"
5 = "AirPlay"
```

Zones must be 1–18. Inputs must be 2–6. Input 1 is the tuner and is omitted. Names used in URLs are the room names, such as `/zones/Master/inputs/AirPlay`.

## What it sends

One task owns the serial port at 38400 8N1, no flow control. Each command ends in a carriage return. The reply is read and discarded. A `FAIL` from the receiver is logged, and the button still returns 204.

| Button | Command |
| --- | --- |
| Input | `zsc,<zone>,<code>` |
| Volume up | `zsc,<zone>,12` |
| Volume down | `zsc,<zone>,13` |
| Mute | `zsc,<zone>,11` |
| Zone off | `zsc,<zone>,10` |
| All off | `znt,10,h` |

Volume up and down are the decimal codes from the control-codes sheet. The server does not send tuner, tone, transport, status-poll, or system commands. Factory reset is not implemented.

## Docs

- [design.md](design.md) is the product design: screen, HTTP paths, and the command allowlist.
- [protocol.md](protocol.md) is the RS-232 reference, including the cable.
- The command list follows Niles’s [ZR-6 control-codes sheet](https://nilesaudio.com/sites/nilesaudio.com/files/_/techsupport/pdf/zr6_control_codes.pdf).
