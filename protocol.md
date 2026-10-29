# Niles ZR-6 RS-232 Control Protocol

Practical guide to the serial protocol used by the Niles ZR-6 MultiZone Receiver. The receiver accepts ASCII commands and replies in the same form. A controller polls for status. The ZR-6 sends a zone update when a command asks for one.

The command list follows Niles’s ZR-6 control-codes sheet: [zr6_control_codes.pdf](https://nilesaudio.com/sites/nilesaudio.com/files/_/techsupport/pdf/zr6_control_codes.pdf). Where that sheet and the ZR-6 installation manual disagree, the difference is called out below.

Zones are numbered **1–18**. One chassis covers six zones. Zones 1–6 are the master, 7–12 the first slave, and 13–18 the second slave.

## Contents

- [Connection](#connection)
- [Message rules](#message-rules)
- [The active control zone](#the-active-control-zone)
- [Command summary](#command-summary)
- [Select the active zone](#select-the-active-zone)
- [Read zone status](#read-zone-status)
- [Control one zone](#control-one-zone)
- [Control many zones](#control-many-zones)
- [Tune the radio](#tune-the-radio)
- [Ready message](#ready-message)
- [Failures](#failures)
- [Factory reset](#factory-reset)
- [Command codes](#command-codes)
- [Example sessions](#example-sessions)
- [Prefix index](#prefix-index)
- [Notes on the source](#notes-on-the-source)
- [Ambiguities and inconsistencies](#ambiguities-and-inconsistencies)
- [Commands to test](#commands-to-test)

## Connection

The port is a **3.5 mm female** jack on the ZR-6. Serial settings are fixed.

| Setting | Value |
| --- | --- |
| Baud | 38400 |
| Data bits | 8 |
| Parity | None |
| Stop bits | 1 |
| Flow control | None |

Tip and ring are named from the ZR-6 side. A PC serial port, and a typical USB-to-DB9 adapter, is wired the other way: pin 2 listens and pin 3 talks. Cross the two data wires.

| ZR-6 3.5 mm | ZR-6 signal | DB9 pin | Computer signal |
| --- | --- | --- | --- |
| Tip | Transmit | 2 | Receive (RX) |
| Ring | Receive | 3 | Transmit (TX) |
| Sleeve | Ground | 5 | Ground |

Leave DB9 pins 1, 4, 6, 7, 8, and 9 open. Flow control is unused, so RTS, CTS, DTR, and DSR stay unconnected. Read the pin numbers molded on the connector. The solder cups on a female plug are mirrored from the face that mates with the adapter.

The jack expects RS-232 voltage levels, the kind a DB9 serial port already uses. A 3.3 V or 5 V UART wired straight to the tip and ring is a different electrical interface.

## Message rules

Every message is ASCII text, fields separated by commas, ending in a **carriage return** (`CR`, byte `0x0D`). Examples below write that as `<CR>`. Published examples contain neither spaces nor a line feed.

A successful reply repeats the command family with an `r` in front, then usually `OK`. A failed command comes back with `FAIL` and a numeric reason. See [Failures](#failures).

```
→ zsc,3,01<CR>          controller to ZR-6
← rzsc,3,01,OK<CR>      ZR-6 to controller
```

Volume, bass, and treble move one step per command. Read the result with a status poll. An absolute level command is not in this protocol.

## The active control zone

Two different ideas share the word "zone":

| Idea | How you address it | What it affects |
| --- | --- | --- |
| Active control zone | `znc,4,<zone>` | Status polls and direct tuning |
| A specific zone | `zsc,<zone>,<code>` | That zone only. The command carries the zone number, so any zone 1–18 can be addressed while another zone stays active |

`znc,4` chooses which zone later status and tuner commands talk about. Source, volume, and power stay as they were. To read zones 1, 2, and 3, select each one and poll it. See [Poll several zones](#poll-several-zones).

## Command summary

| You want to | Send | Reply |
| --- | --- | --- |
| Make zone *z* the active control zone | `znc,4,<z>` | `rznc,4,<z>` |
| Read the active zone | `znc,5` | `usc,2,<z>,<source>,<power>,<vol>,<mute>,<bass>,<treble>` |
| Run command *c* in zone *z* | `zsc,<z>,<c>` | `rzsc,<z>,<c>,OK` |
| Hold command *c* across zones | `znt,<c>,h` | `rznt,<c>,OK` |
| Tune the active zone's radio | `src,11,<station>` | `rsrc,11,OK` |
| Hear that boot finished | *(unsolicited)* | `usc,1` |
| Erase configuration and presets | `syc,3,` | `rsyc,3,` |

`<z>` is 1–18. `<c>` is a code from [Command codes](#command-codes). Each line ends with `<CR>`.

## Select the active zone

Makes one zone the target for status polls and direct tuning.

```
znc,4,<zone><CR>
```

| Field | Meaning |
| --- | --- |
| `znc` | Zone command |
| `4` | Change the active control zone |
| `<zone>` | Zone number, 1–18 |

```
→ znc,4,2<CR>
← rznc,4,2<CR>
```

`rznc,4,2` confirms that zone 2 is now the active control zone.

```
→ znc,4,18<CR>
← rznc,4,18<CR>
```

An out-of-range zone fails. Code **6** is "invalid zone number" and code **12** is "invalid index". See [Failures](#failures).

## Read zone status

Returns the live settings of the **active control zone**. Select that zone first.

```
znc,5<CR>
```

```
→ znc,5<CR>
← usc,2,2,3,1,14,0,0,0<CR>
```

| Field | Example | Meaning |
| --- | --- | --- |
| 1 | `usc` | Status reply |
| 2 | `2` | This reply is the answer to a status request |
| 3 | `2` | Zone the numbers describe (the active control zone) |
| 4 | `3` | Selected source, 1–6. Source **1** is the built-in tuner |
| 5 | `1` | Power. `1` = on, `0` = off |
| 6 | `14` | Volume, 0–100. The source describes this figure as a percent |
| 7 | `0` | Mute. `1` = muted, `0` = unmuted |
| 8 | `0` | Bass, −7 to +7. `0` is flat |
| 9 | `0` | Treble, −7 to +7. `0` is flat |

Reading that example aloud: zone 2 is playing source 3, the zone is on, volume is 14, mute is off, bass and treble are flat.

The published sample uses `0` for flat tone. The source gives the bass and treble range as −7 to +7 and does not show a negative sample, so the sign character for a cut is not illustrated.

Sources 2–6 are the external inputs. Their names are whatever was configured on the chassis. The status field returns the source number.

## Control one zone

Sends one remote-style command to one zone. The zone number travels in the command, so the active control zone can stay where it is.

```
zsc,<zone>,<code><CR>
```

| Field | Meaning |
| --- | --- |
| `zsc` | Zone-specific command |
| `<zone>` | Zone to act on, 1–18 |
| `<code>` | Command code. Use the decimal **Send** column in [Command codes](#command-codes) |

```
→ zsc,3,01<CR>
← rzsc,3,01,OK<CR>
```

Zone 3 selected the tuner (source 1). `OK` means the ZR-6 accepted the command.

More of the same shape:

| Action | Send | Reply |
| --- | --- | --- |
| Select source 2 in zone 5 | `zsc,5,02` | `rzsc,5,02,OK` |
| Volume up once in zone 2 | `zsc,2,12` | `rzsc,2,12,OK` |
| Volume down once in zone 7 | `zsc,7,13` | `rzsc,7,13,OK` |
| Mute zone 1 | `zsc,1,11` | `rzsc,1,11,OK` |
| Turn zone 4 off | `zsc,4,10` | `rzsc,4,10,OK` |
| Power key in zone 6 | `zsc,6,49` | `rzsc,6,49,OK` |
| Play in zone 3 | `zsc,3,17` | `rzsc,3,17,OK` |
| Enter in zone 8 | `zsc,8,69` | `rzsc,8,69,OK` |

Each volume, bass, or treble command is a single step. Three steps up is command `12` sent three times, each answered with `OK`. Poll afterward to learn the new level.

Mute is a single code (`11`). The status field tells you whether the zone is muted now. The source does not define a separate unmute code.

## Control many zones

The hold form applies one command more widely than a single zone. The command family is `znt` (zone transport).

```
znt,<code>,h<CR>
```

`h` is the hold flag. Hold is valid for **source select** (`01`–`06`) and **Off** (`10`).

| Command | Who it affects | What it does |
| --- | --- | --- |
| Source select + `h` | Zones with party mode enabled | Turns those zones on and selects that source |
| Off + `h` | All zones | Turns every zone off |

Any other code with `h` fails with reason **8** ("hold flag is not allowed").

Select the tuner in every party-enabled zone:

```
→ znt,01,h<CR>
← rznt,01,OK<CR>
```

The reply names the command (`01`) and leaves the zone number out, because more than one zone was addressed.

Turn every zone off:

```
→ znt,10,h<CR>
← rznt,10,OK<CR>
```

Select source 4 in every party-enabled zone:

```
→ znt,04,h<CR>
← rznt,04,OK<CR>
```

Party membership is configured on the ZR-6. The serial command uses that configuration. It does not include a way to edit the party list.

## Tune the radio

Tunes a frequency on the chassis tuner that belongs to the **active control zone**.

```
src,11,<station><CR>
```

Before this will do what you expect:

1. Make the listening zone the active control zone (`znc,4,<zone>`).
2. Select the tuner in that zone (`zsc,<zone>,01`), so source 1 is the current source.
3. Send `src,11,<station>`.

| Zones | Tuner that receives `src,11` |
| --- | --- |
| 1–6 | Master chassis |
| 7–12 | First slave chassis |
| 13–18 | Second slave chassis |

| Band | Form | Example | Meaning |
| --- | --- | --- | --- |
| FM | `XXX.X` with a decimal point | `102.7` | FM 102.7 |
| AM | Four digits | `0560` | AM 560 |

```
→ src,11,102.7<CR>
← rsrc,11,OK<CR>

→ src,11,0560<CR>
← rsrc,11,OK<CR>
```

`11` in the reply means "direct tune", the same code that was sent. A bad frequency string fails with a parameter error (reasons **20**–**25**). Reason **24** is "too many dots" and reason **25** is "must be all numbers", which covers a malformed station field.

The published FM example is `102.7`. The source describes FM as `XXX.X` and does not show a station below 100 MHz.

## Ready message

After power-up the ZR-6 runs its diagnostics and writes the results to the port. When those tests finish cleanly it sends:

```
← usc,1<CR>
```

| Field | Meaning |
| --- | --- |
| `usc` | Unsolicited message |
| `1` | The ZR-6 is ready for serial commands |

`usc,1` arrives on its own. A controller that opens the port mid-boot should ignore diagnostic lines it does not recognize until this one shows up, or until it gets a normal reply to a command it sent.

## Failures

A command the ZR-6 rejects uses this shape:

```
r<type>,<code>,FAIL,<reason><CR>
```

| Field | Meaning |
| --- | --- |
| `r<type>` | Reply family. `type` is the command prefix that failed |
| `<code>` | Command number that failed |
| `FAIL` | The command was rejected |
| `<reason>` | Reason code from the table below |

Prefixes that can appear in `r<type>`:

| Prefix | Family |
| --- | --- |
| `znc` | Zone command |
| `znt` | Zone transport (hold) |
| `src` | Source command |
| `srt` | Source transport |
| `spc` | Special command |
| `syc` | System command |
| `zsc` | Zone-specific command |

`srt` and `spc` are named in the failure text. This protocol listing does not give commands for them.

The source publishes the pattern and does not include a captured failure trace. These two lines are that pattern filled in:

```
← rznc,9,FAIL,2<CR>       command 9 is not a known zone command (reason 2)
← rznt,12,FAIL,8<CR>      volume up does not accept hold (reason 8)
```

A failure line has no zone field. The success form `rzsc,3,01,OK` carries the zone. The failure form `rzsc,01,FAIL,2` carries the command number.

| Code | Meaning |
| --- | --- |
| 1 | Cannot allocate memory to process the message |
| 2 | Invalid command |
| 3 | Incomplete message: missing command |
| 4 | Incomplete message: missing parameter |
| 5 | Cannot allocate memory to process the command |
| 6 | Cannot change the zone: invalid zone number |
| 7 | Invalid key code (command type) |
| 8 | Hold flag is not allowed on this command |
| 9 | Repeat flag is not allowed |
| 10 | Invalid flag |
| 11 | Flags are not allowed |
| 12 | Cannot find the zone: invalid index |
| 13 | Cannot find the source: invalid index |
| 14 | Cannot find the soft key: invalid index |
| 15 | Cannot find the menu item: invalid index |
| 16 | IP address is not defined |
| 17 | Alternative behavior is not supported |
| 18 | ZR-6 is waiting for the previous special-command answer |
| 19 | ZR-6 is waiting for the previous special-command setting answer |
| 20 | Parameter 1 is out of range |
| 21 | Parameter 2 is out of range |
| 22 | Parameter 3 is out of range |
| 23 | Parameter 4 is out of range |
| 24 | Invalid parameter: too many dots |
| 25 | Invalid parameter: must be all numbers |
| 26 | Menu is not available |
| 27 | `#` or the byte count does not match the string that was received |
| 28 | Menu does not support alphanumeric search |

Reason 9 mentions a repeat flag. The command list in this document does not define a repeat syntax. Reasons 14–19 and 26–28 refer to menus, soft keys, and special commands beyond the commands specified here.

## Factory reset

Erases configuration and tuner presets and restores factory defaults. The ZR-6 installation manual adds that **learned IR commands are kept**.

```
→ syc,3,<CR>
← rsyc,3,<CR>
```

Both the command and the reply are published **with the trailing comma** after `3`.

| Field | Meaning |
| --- | --- |
| `syc` | System command |
| `3` | Erase EEPROM |
| `rsyc` | Reply to a system command |

Treat this as destructive. It clears installer configuration and radio presets. Leave it out of control programs. The test procedure does not ask for it.

## Command codes

Send the decimal number in the **Send** column. The worked examples use that form: tuner select is `zsc,3,01`, and volume up is `12` (hex `0C` in the second source table).

Codes under 10 are written with a leading zero, matching the examples (`01`, `02`). Larger codes are sent as their decimal number (`12`, `49`, `126`, `144`).

The source prints two tables. They are one numbering system. The second table is hexadecimal and lists the buttons of the R-6L remote, including bass, treble, and extra tuner keys that the decimal table omits. For those keys, **Send** is the hex value converted to decimal.

**(h)** means the code accepts the hold flag and may be used with `znt,<code>,h`.

### Source and power

| Command | Send | Hex | Hold |
| --- | --- | --- | --- |
| Tuner select (source 1) | `01` | `01` | yes |
| Source 2 select | `02` | `02` | yes |
| Source 3 select | `03` | `03` | yes |
| Source 4 select | `04` | `04` | yes |
| Source 5 select | `05` | | yes |
| Source 6 select | `06` | | yes |
| Off | `10` | `0A` | yes |
| Power | `49` | | |

Source 1 is the built-in tuner. The party-mode example uses `znt,01,h` for that tuner, which is why tuner select is marked hold even though a few printings leave the mark off the `01` row.

### Level

| Command | Send | Hex | Hold |
| --- | --- | --- | --- |
| Mute | `11` | `0B` | |
| Volume up | `12` | `0C` | |
| Volume down | `13` | `0D` | |
| Bass increase | `128` | `80` | |
| Bass decrease | `129` | `81` | |
| Treble increase | `130` | `82` | |
| Treble decrease | `131` | `83` | |

Bass and treble appear only in the hex table. `128`–`131` are `0x80`–`0x83` written as decimal, the same conversion that turns volume up `0C` into send code `12`.

```
→ zsc,2,128<CR>
← rzsc,2,128,OK<CR>
```

That steps zone 2's bass up by one. Poll `znc,5` to read the new bass field.

### Keypad

| Command | Send | Hex |
| --- | --- | --- |
| 1 | `33` | `21` |
| 2 | `34` | `22` |
| 3 | `35` | `23` |
| 4 | `36` | `24` |
| 5 | `37` | `25` |
| 6 | `38` | `26` |
| 7 | `39` | `27` |
| 8 | `40` | `28` |
| 9 | `41` | `29` |
| 0 | `42` | `2A` |
| Enter | `69` | `45` |

Digits are remote keypad keys for the selected source. They are a different path from `src,11`, which tunes a frequency in one command.

### Navigation

| Command | Send | Hex |
| --- | --- | --- |
| Cursor up | `23` | |
| Cursor left | `25` | |
| Select | `26` | |
| Cursor right | `27` | |
| Cursor down | `29` | |
| Previous | `43` | `2B` |
| Next | `44` | `2C` |
| Page up | `126` | |
| Page down | `127` | |

### Transport

These codes match the source-device buttons (play, pause, and so on) for whichever input the zone has selected.

| Command | Send | Hex |
| --- | --- | --- |
| Play | `17` | |
| Stop | `18` | |
| Pause | `19` | |
| Rewind | `20` | |
| Fast forward | `21` | |
| Shuffle | `45` | |
| Group | `46` | |
| Disc | `47` | |
| Favorites | `48` | |
| Record | `144` | |

### Tuner keys

Published in the hex table only. **Send** is the decimal form.

| Command | Send | Hex |
| --- | --- | --- |
| Tuner scan down | `25` | `19` |
| Tuner scan up | `27` | `1B` |
| Tuner next preset | `67` | `43` |
| Tuner previous preset | `68` | `44` |

Two of those numbers are also navigation commands in the decimal table:

| Send | Hex | Decimal-table name | Hex-table name |
| --- | --- | --- | --- |
| `25` | `19` | Cursor left | Tuner scan down |
| `27` | `1B` | Cursor right | Tuner scan up |

Both names are printed in the source for that same code. Preset keys have their own send codes. Hex `43` is tuner next preset, which is send code `67`. Decimal `43` is **Previous**, and decimal `44` is **Next**.

```
→ zsc,2,67<CR>          tuner next preset, from hex 43
← rzsc,2,67,OK<CR>

→ zsc,2,43<CR>          previous track / previous item
← rzsc,2,43,OK<CR>
```

## Example sessions

`<CR>` is omitted in the commentary and present on the wire.

### Play FM 102.7 in zone 2

```
→ znc,4,2<CR>
← rznc,4,2<CR>

→ zsc,2,01<CR>
← rzsc,2,01,OK<CR>

→ src,11,102.7<CR>
← rsrc,11,OK<CR>

→ znc,5<CR>
← usc,2,2,1,<power>,<volume>,<mute>,<bass>,<treble><CR>
```

The status reply's fourth field is the source. After `zsc,2,01` that field is `1` when the tuner selection took effect. The angle-bracket fields come back as the zone's current levels. A fully numeric sample from the source, for a different moment (zone 2, source 3, on, volume 14, unmuted, flat tone), looks like:

```
← usc,2,2,3,1,14,0,0,0<CR>
```

### Play AM 560 in zone 9

Zone 9 uses the tuner in the first slave.

```
→ znc,4,9<CR>
← rznc,4,9<CR>

→ zsc,9,01<CR>
← rzsc,9,01,OK<CR>

→ src,11,0560<CR>
← rsrc,11,OK<CR>
```

### Step the volume, then read it

```
→ zsc,2,12<CR>
← rzsc,2,12,OK<CR>

→ zsc,2,12<CR>
← rzsc,2,12,OK<CR>

→ znc,4,2<CR>
← rznc,4,2<CR>

→ znc,5<CR>
← usc,2,2,<source>,<power>,<volume>,<mute>,<bass>,<treble><CR>
```

Volume is the sixth field. The two `zsc` lines each raise it by one step. The `znc,4,2` line matters when some other zone was the active control zone. Status always describes the active one.

### Mute one zone and leave the others alone

```
→ zsc,6,11<CR>
← rzsc,6,11,OK<CR>
```

Zone 6 mutes even if zone 2 is the active control zone. Confirm by selecting zone 6 and polling. Mute status is the seventh field (`1` = muted).

### Party mode, then all off

```
→ znt,01,h<CR>
← rznt,01,OK<CR>

→ znt,10,h<CR>
← rznt,10,OK<CR>
```

The first line selects the tuner and turns on every zone that is enabled for party mode. The second turns every zone off.

### Poll several zones

```
→ znc,4,1<CR>
← rznc,4,1<CR>
→ znc,5<CR>
← usc,2,1,<source>,<power>,<volume>,<mute>,<bass>,<treble><CR>

→ znc,4,2<CR>
← rznc,4,2<CR>
→ znc,5<CR>
← usc,2,2,<source>,<power>,<volume>,<mute>,<bass>,<treble><CR>

→ znc,4,3<CR>
← rznc,4,3<CR>
→ znc,5<CR>
← usc,2,3,<source>,<power>,<volume>,<mute>,<bass>,<treble><CR>
```

The third field of each `usc,2` line echoes the zone you just activated. A controller can keep a cache per zone and refresh it on a timer. Nothing in the protocol pushes a change when someone uses a keypad.

### A command the ZR-6 rejects

Hold is legal on source select and Off. Volume up is command `12` and has no hold flag:

```
→ znt,12,h<CR>
← rznt,12,FAIL,8<CR>
```

Reason 8 is "hold flag is not allowed". The same command addressed to one zone is legal:

```
→ zsc,3,12<CR>
← rzsc,3,12,OK<CR>
```

## Prefix index

| Prefix | Direction | Meaning |
| --- | --- | --- |
| `znc` | Controller → ZR-6 | Zone command: set the active zone, or ask for status |
| `rznc` | ZR-6 → controller | Reply to `znc` |
| `zsc` | Controller → ZR-6 | One command, one zone |
| `rzsc` | ZR-6 → controller | Reply to `zsc` |
| `znt` | Controller → ZR-6 | Held command across party zones, or all zones off |
| `rznt` | ZR-6 → controller | Reply to `znt` |
| `src` | Controller → ZR-6 | Source command. Documented use is direct tune (`11`) |
| `rsrc` | ZR-6 → controller | Reply to `src` |
| `syc` | Controller → ZR-6 | System command. Documented use is EEPROM erase (`3`) |
| `rsyc` | ZR-6 → controller | Reply to `syc` |
| `usc` | ZR-6 → controller | Unsolicited ready (`usc,1`), and the status reply (`usc,2,...`) |
| `srt` | *(failure name only)* | Source transport. No command syntax is given |
| `spc` | *(failure name only)* | Special command. No command syntax is given |

## Notes on the source

The control-codes sheet disagrees with the printed installation manual in a few places, and a few of its own field notes disagree with the examples on the same page. The tables above follow the examples and, where those are silent, the installation manual.

- **Status prefix.** The example reply is `usc,2,2,3,1,14,0,0,0`. The sentence under it on the control-codes sheet calls the prefix `sc`. The installation manual and the example both say `usc`.
- **Volume range.** The control-codes sheet says 0–100 and calls the sample value "14%". The installation manual's protocol page says 0–99.
- **Failure 27.** The control-codes sheet ends with "bytes that were taped". The installation manual says "bytes that were received".
- **Product name on two replies.** The control-codes sheet labels the ready message and the EEPROM reply as coming from a ZR-4. Both messages are ZR-6 replies: `usc,1` and `rsyc,3,`.
- **EEPROM and IR.** The installation manual notes that learned IR commands survive `syc,3,`. The command clears configuration and tuner presets.
- **Tuner select and hold.** `znt,01,h` is the documented party-mode example, so code `01` accepts hold.
- **Cursor and scan codes.** Decimal `25` / hex `19`, and decimal `27` / hex `1B`, each have two names. See [Tuner keys](#tuner-keys).
- **Trailing comma.** `syc,3,` and `rsyc,3,` are printed with a comma before the carriage return. The status example is printed as `znc,5` with no extra comma. The syntax line for status is sometimes written `znc,5,`.

The list above is the settled errata. Open questions, and the commands that resolve them, are in the next two sections.

## Ambiguities and inconsistencies

These are places where the control-codes sheet disagrees with its own examples, disagrees with the ZR-6 installation manual, or names a behavior and never specifies it. The earlier sections of this guide pick one reading so a controller can be written. That reading is the **working assumption**. It stays provisional until the matching test in [Commands to test](#commands-to-test) comes back from a real chassis.

Settled typos (the ZR-4 label on two replies, "taped" versus "received", learned IR surviving an EEPROM erase) are in [Notes on the source](#notes-on-the-source). They do not need a test.

| ID | What is unclear | Working assumption | Test |
| --- | --- | --- | --- |
| A1 | Status reply prefix `usc` or `sc`, and whether more fields follow treble | Prefix is `usc`, nine fields, as in the printed example | T1 |
| A2 | Status request written `znc,5,` in the syntax line and `znc,5` in the example | Send `znc,5` with no trailing comma | T2 |
| A3 | Which other commands require or accept a trailing comma | Only the EEPROM command is written with one. Other commands omit it | T3 |
| A4 | Volume span 0–100 or 0–99, and the size of one up/down step | Level is 0–100. Step size is unstated, so the guide does not assume one | T4 |
| A5 | How a negative or positive bass/treble level is spelled on the wire | Range is −7…+7. The only published sample is `0` | T5 |
| A6 | Command numbers are decimal (`12`) or hex text (`0C`) | Send decimal, because that is the table the examples use | T6 |
| A7 | Send code `25` and send code `27` each have two names | Both names are listed. The chassis will only do one of them | T7 |
| A8 | Tuner preset keys are hex `43`/`44` (decimal 67/68), while decimal 43/44 are Previous/Next | Send `67` and `68` for the preset keys | T8 |
| A9 | Power (`49`) is on, off, or a toggle, and how it relates to Off (`10`) | The names are sent as documented. The effect is unstated | T9 |
| A10 | Mute (`11`) latches mute or toggles | One code, no separate unmute. Toggle is the behavior to confirm | T10 |
| A11 | A source select with no hold flag powers that zone on | Hold+source powers party zones. A plain `zsc` source select only changes the source | T11 |
| A12 | `znc,4` is serial context only, or it also changes what a room is playing | It changes which zone status and direct-tune talk to. Source, volume, and power stay put | T12 |
| A13 | Status might accept a zone number the syntax leaves out | `znc,5` takes no zone parameter | T13 |
| A14 | Direct-tune grammar, and whether the tuner must already be selected | Tuner is the active zone's source first. FM is `XXX.X`, AM is four digits | T14 |
| A15 | How many zone numbers this chassis actually accepts | The protocol allows 1–18, mapped across master and two slaves | T15 |
| A16 | Leading zeros, letter case, and spaces | Match the examples: lowercase, no spaces, codes under 10 written as two digits | T16 |
| A17 | Reply ending is CR, LF, or CRLF, and whether a command must end in CR | Commands and replies end in CR only | T17 |
| A18 | Exact failure spelling: padding, zone field, `FAIL` case, trailing comma | `r<type>,<code>,FAIL,<reason>` with no zone field | T18 |
| A19 | Exact success echo: padding, `OK` case, whether `znt` echoes `h` | `rzsc,<zone>,<code>,OK`, `rznt,<code>,OK`, `rsrc,11,OK` | T19 |
| A20 | Party hold turns zones on silently, or each zone also emits status | One `rznt,<code>,OK`. No per-zone push | T20 |
| A21 | Error 9 names a repeat flag and never gives its syntax | No repeat suffix is documented, so none is sent | T21 |
| A22 | Keypad and front-panel actions stay off the serial port | Traffic arrives only as a reply, plus boot messages | T22 |
| A23 | Boot diagnostic lines that precede `usc,1` | `usc,1` means ready. Earlier lines are unspecified | T23 |
| A24 | EEPROM erase is shown as `syc,3,` with a trailing comma | The comma is part of the published command. Running either form erases configuration | none |

### A1. Status prefix and field count

The worked example is `usc,2,2,3,1,14,0,0,0`. The sentence under that example on the control-codes sheet says the prefix is `sc`. The installation manual's protocol page agrees with the example and says `usc`.

A live reply can also be longer than the nine fields in the example. Nothing in the source says the treble field is the last one. A frequency, a preset number, or a zone name would not contradict the published sample if it were appended.

**Working assumption.** Parse `usc`, then the status marker `2`, then zone, source, power, volume, mute, bass, treble. Keep any extra fields when logging.

### A2. Trailing comma on the status request

The syntax line is `znc,5,[cr]`. The example on the control-codes sheet is `znc,5[cr]`. The installation manual's worked example is printed `znc,5,[cr]`, comma included.

**Working assumption.** Send `znc,5` and a carriage return.

### A3. Trailing commas everywhere else

`syc,3,[cr]` and `rsyc,3,[cr]` are printed with a comma. Every other worked example (`znc,4,2`, `zsc,3,01`, `znt,01,h`, `src,11,102.7`) ends on its last field. It is unknown whether a trailing comma is harmless, required, or a syntax error on those commands.

**Working assumption.** Send the comma only on the EEPROM command.

### A4. Volume span and step size

The control-codes sheet says volume is 0–100 and describes the sample `14` as 14%. The installation manual's protocol page says 0–99. Neither source says whether one Volume Up moves the level by 1, by 2, or by some other amount, nor what a further step does at the top or the bottom (another `OK` with the level clamped, or `FAIL`).

**Working assumption.** Treat the field as a percent in 0–100. Discover the step from T4 before writing a "set volume to N" loop.

### A5. Bass and treble text

The range is −7 to +7. The only sample is `0`, so the spelling of a cut or a boost is unknown: `-3`, `+3`, a bare `3`, or some other token. The step size and the clamp at ±7 are also unpublished.

The bass and treble *commands* exist only in the hex table (`80`–`83`). The guide sends them as decimal `128`–`131`, which is the same conversion that turns hex `0C` into volume-up `12`. That conversion is an inference. See A6.

**Working assumption.** Levels are signed integers from −7 to +7, spelled with a leading minus when negative. Commands are `128`, `129`, `130`, `131`.

### A6. Decimal versus hex command numbers

This is the ambiguity that changes what every command above 9 should look like.

The two code tables describe one set of buttons. Volume up is `12` in the first table and `0C` in the hex table, and `0x0C` is 12. The same match holds for mute (`11` / `0B`), Off (`10` / `0A`), the keypad (`33` / `21` through `42` / `2A`), Enter (`69` / `45`), Next (`44` / `2C`), and Previous (`43` / `2B`). The worked examples only use tuner select `01`, which is the same text either way.

If the chassis reads the field as **decimal**:

| You send | The chassis does |
| --- | --- |
| `12` | Volume up |
| `0C` | Rejects the letter, likely reason 25 |
| `10` | Off |
| `128` | Bass increase |
| `80` | Command 80, which is not in the decimal table |

If the chassis reads the field as **hex**:

| You send | The chassis does |
| --- | --- |
| `0C` | Volume up |
| `12` | Command `0x12`, which is decimal 18, **Stop** |
| `0A` | Off |
| `10` | Command `0x10` (decimal 16), which is in neither table |
| `80` | Bass increase |
| `128` | Either bass increase (`0x80` written with an extra digit) or a rejection |

The first probe has to be `12`, watched against the volume field. Volume going up means the field is decimal. Playback stopping, with volume unchanged, means the field was read as hex `12` and executed Stop. `10` is the wrong probe: as decimal it turns the zone off, and as hex it fires an undocumented code.

**Working assumption.** The field is decimal. Hex text is a second printing of the same numbers, not a second wire format.

### A7. One code, two names

| Send | Hex printing | Name in the decimal table | Name in the hex table |
| --- | --- | --- | --- |
| `25` | `19` | Cursor left | Tuner scan down |
| `27` | `1B` | Cursor right | Tuner scan up |

Status has no frequency field, so a serial reply of `OK` does not say which function ran. The keypad or tuner display does.

`19` and `1B` as literal command text are a second question, folded into A6. Under a decimal parser, `19` is Pause and `1B` is an illegal letter.

**Working assumption.** List both names. Tell a caller that `25` and `27` are the published numbers, and that the function has to be confirmed on hardware.

### A8. Tuner preset codes

| Intent | Hex table | Decimal equivalent | Collision |
| --- | --- | --- | --- |
| Next preset | `43` | `67` | Decimal `43` is Previous |
| Previous preset | `44` | `68` | Decimal `44` is Next |

Sending the hex digits `43` on a decimal parser steps Previous, not the next radio preset.

**Working assumption.** Send `67` and `68`.

### A9. Power and Off

Off is code `10` and supports hold, which the source defines as "turn all zones off". Power is code `49` and the source never says whether that key turns a zone on, turns it off, or flips it. A zone that is already off may ignore one of them.

**Working assumption.** Use `10` to turn a zone off. Treat `49` as the remote's Power key and read the power field afterward.

### A10. Mute

Code `11` is Mute. There is no Unmute code. The status field reports the current state (`1` muted, `0` unmuted). The command may set mute, or it may toggle.

**Working assumption.** Send `11` and then read status. Send `11` again if the zone is muted and the first result shows a toggle.

### A11. Source select and power

`znt,01,h` is defined as turning party-enabled zones on and selecting the tuner. The single-zone form `zsc,<zone>,01` is defined only as selecting the tuner. It may also power that zone on, which is what the keypad source keys do on many of these receivers, and the serial page does not say so.

**Working assumption.** A `zsc` source select changes the source field. Power changes when the command is Off, Power, or a held source select.

### A12. Active control zone

`znc,4,<zone>` "selects the zone to receive commands." The commands that actually consult that selection are the ones with no zone number in them: `znc,5` and `src,11`. A `zsc` carries its own zone number.

Unknowns:

- Whether `znc,4` changes source, volume, power, or the room you hear.
- Whether a later `zsc` to a different zone moves the active selection.
- Whether the active selection survives commands that fail.

**Working assumption.** `znc,4` changes serial context only. `zsc` does not move it.

### A13. A zone on the status request

The syntax is `znc,5`. A controller that wants zone 5 has to send `znc,4,5` and then `znc,5`. The source may simply have omitted a legal form such as `znc,5,5`.

**Working assumption.** The status request has no parameters. A zone field is a malformed command.

### A14. Direct tune

Documented rules:

- The tuner is the selected source in the active control zone.
- Zones 1–6 tune the master, 7–12 the first slave, 13–18 the second slave.
- FM looks like `XXX.X` and includes a decimal point. The only sample is `102.7`.
- AM looks like `XXXX`. The sample is `0560`.

Unknowns:

- Whether the chassis accepts the command and tunes when the current source is not the tuner.
- Whether the active zone or the zone that last received `zsc,...,01` decides which tuner moves.
- FM under 100 MHz: `98.5` or `098.5`.
- AM without the leading zero: `560` or only `0560`.
- Extra precision (`102.70`), a missing dot (`1027`), and frequencies outside the band.

`OK` means the string was accepted. The station that actually came up is on the tuner display, because status does not report it.

**Working assumption.** Select the tuner, make that zone active, then send `XXX.X` or a zero-padded four-digit AM frequency.

### A15. Zone numbers on this installation

The protocol allows zones 1–18. A single chassis has zones 1–6. Zone 7 on a master-only system may come back as reason 6 or reason 12, or it may succeed and refer to a zone with no keypad.

**Working assumption.** Send 1–18 when the installation has the slaves. Ask the chassis (T15) before assuming a zone exists.

### A16. Padding, case, and spaces

Samples use `01`, not `1`, and `zsc`, not `ZSC`, and they contain no spaces. Codes above 9 are printed as bare decimals (`12`, `49`, `126`, `144`), so three-digit codes are clearly legal. Whether `1` is accepted as `01`, and whether `012` is accepted as `12`, is unstated. Reason 25 ("must be all numbers") implies a letter in a numeric field is rejected, which covers hex text under a decimal parser (A6).

**Working assumption.** Lowercase, no spaces, two digits when the sample uses two digits.

### A17. Line endings

The source says carriage return. It does not mention line feed. A controller that sends CRLF, the way many serial libraries do, may get a clean reply, a failure, or a reply plus a second error caused by the stray `LF`.

The reply's own ending is also unpublished. The examples write `[cr]` and never mention `LF`.

**Working assumption.** Send `CR` only. Treat `CR` as the end of a reply. Record the raw bytes once so this is measured rather than assumed.

### A18. Failure line

The pattern is `r[type],[com #],FAIL,[fcode][cr]`. No sample failure is printed. Open details:

- Is the command number echoed the way it was sent (`01`), as an unpadded integer (`1`), or always in some fixed width?
- Does a failed `zsc` include the zone? The pattern has no zone field. The success line does (`rzsc,3,01,OK`).
- Is the word `FAIL` upper case, and is there a trailing comma?
- Do reason 6 and reason 12 both show up for a bad zone, and on which commands?

**Working assumption.** Follow the pattern literally: no zone field, upper-case `FAIL`, no trailing comma.

### A19. Success line

Same echo questions on the happy path. `znt,01,h` is answered `rznt,01,OK` in the example, so the hold flag is not echoed there. `src,11,102.7` is answered `rsrc,11,OK`, so the frequency is not echoed. A chassis could still echo them.

**Working assumption.** Match the examples. T6, T9, and T14 record the real echo as a side effect, which is what T19 checks.

### A20. Party hold and unsolicited zone updates

The party example returns a single `rznt,01,OK`. The introduction says the receiver answers requests and does not push status. A hold that turns several zones on might still emit one `usc,2,...` per zone, which the introduction would not lead you to expect.

Which zones are "party enabled" is configuration, not a serial query. There is no documented command to read or edit that set.

**Working assumption.** Expect one `rznt` reply. Snapshot every zone yourself if you need to know what changed.

### A21. Repeat flag and the unnamed families

Reason 9 is "repeat flag is not allowed". Reason 8 is the matching sentence for the hold flag, and hold is the letter `h`. No letter is given for repeat. Reasons 10 and 11 ("invalid flag", "flags are not allowed") imply more than one flag character exists.

Failure types `srt` (source transport) and `spc` (special command) have no request syntax anywhere in the sheet. Reasons 14–19 and 26–28 talk about soft keys, menus, IP addresses, and special-command answers. Those command families were left out of this protocol listing.

**Working assumption.** Send `h` only, and only on source select and Off. Do not invent `srt` or `spc` commands. T21 tries one obvious suffix so the failure text can identify the flag, and stops there.

### A22. Traffic from the keypad

"Responds only to requests" means a keypad press should produce no serial line. Installations sometimes still emit zone status when a local key is used. The source gives no such line to parse.

**Working assumption.** Ignore nothing: log unexpected bytes. Parse them only after T22 shows they exist.

### A23. Boot text

During startup the receiver writes the results of its diagnostic tests, then `usc,1` if those tests passed. The diagnostic lines have no grammar. A controller that opens the port during boot has to ignore unrecognized lines until `usc,1` or until a command of its own is answered.

**Working assumption.** `usc,1` is the ready marker. Anything before it on a fresh power-up is diagnostic text.

### A24. EEPROM erase

Published command `syc,3,` with the trailing comma. Published reply `rsyc,3,` with the trailing comma. Whether the comma is required is unknown, and the way to find out is to send the command.

The installation manual says this erases configuration and tuner presets and keeps learned IR. That is a factory reset. There is no test for it below. Control software must not send this command, and it is not part of the procedure in [Commands to test](#commands-to-test).

## Commands to test

Type these on the ZR-6 one line at a time, ending each line with a carriage return, and paste the reply back. Each test is named so a reply can be filed against the ambiguity with the same number.

Use **zone 1** throughout, unless that room should stay untouched. If you substitute a zone, use the same zone for the whole run and say which one. Skip any test marked optional. Skip a test the moment it would interrupt someone who is listening, and say that you skipped it.

Factory reset is excluded. Do not type it, and do not put it in a program.

### What to capture

A useful log is the raw exchange, not a paraphrase. For every command record the exact text you sent and the exact text that came back, including a second line if one arrives. Write a carriage return as `<CR>` and a line feed as `<LF>` when you can see them.

The port is 38400, 8 data bits, no parity, 1 stop bit, no flow control. If a command fails, write down the failure and continue. The failure line is the result.

For the tuner tests, add the frequency the keypad or front panel shows after the reply. Status does not contain it.

### Core batch

Serial context, framing, and harmless parser checks. Zone 1's audio should stay as it is, apart from T12 selecting zone 2 as the *control* zone and then selecting zone 1 again. If A12 is wrong and `znc,4` itself switches the room, stop and say what you heard.

**T15.** Which zone numbers exist.

```
znc,4,1
znc,4,0
znc,4,6
znc,4,7
znc,4,19
znc,4,1
```

**T1.** Status prefix, field count, and the raw line ending.

```
znc,5
```

**T2.** Trailing comma on the status request.

```
znc,5,
```

**T13.** Status request with a zone stuck on the end.

```
znc,5,1
```

**T16.** Letter case, then a space after the comma.

```
ZNC,5
znc, 5
```

**T18.** Failure shape. None of these name a real zone command.

```
znc,9
znc,4,99
zsc,99,01
zsc,1
zsc,1,
```

**T12.** Active zone is context. Compare the three status bodies.

```
znc,4,1
znc,5
znc,4,2
znc,5
znc,4,1
znc,5
```

**T3.** Trailing comma on a zone-select that is otherwise legal.

```
znc,4,1,
znc,4,1
```

What decides the questions:

| Result | Resolves |
| --- | --- |
| Status line starts `usc,2` or `sc,2` | A1 prefix |
| Fields after the ninth | A1 field count. Paste the whole line |
| `znc,5` succeeds and `znc,5,` fails, or the reverse, or both succeed | A2 |
| `znc,5,1` returns status for zone 1, or `FAIL` | A13 |
| `ZNC,5` and `znc, 5` succeed or fail | A16 case and spaces |
| Full text of `rznc,...,FAIL,...` and `rzsc,...,FAIL,...` | A18. Note whether the zone number appears |
| The three `usc` bodies match, and the middle one reports zone 2 | A12. Also note whether the room audibly switched |
| `znc,4,7` is `OK` or `FAIL` with a reason | A15. `FAIL` on zone 7 means this chassis has no slave |
| Hex of the T1 reply contains `0d` alone or `0d 0a` | A17 reply ending |

### Level batch

Changes zone 1's power, source, volume, mute, and tone. Start with zone 1 on and playing. If the baseline power field is `0`, turn the zone on from the keypad and start this batch again. Snapshot that first status line and put the zone back when you finish. The fourth field is the source to restore.

Volume, mute, and tone run first, while the zone is still on. Off and Power run at the end so a zone that is off cannot hide a volume change.

T6 is the decimal-versus-hex test. Watch the volume field after `12`. Stop the batch if `12` stops playback instead of raising volume, and say so. That result means the chassis read `12` as hex and executed Stop (decimal 18). Do not continue with `10` or `0A` until that is sorted out.

Baseline. The power field should be `1`.

```
znc,4,1
znc,5
```

**T6.** Decimal `12` should be volume up. `0C` is the hex spelling, and `0c` is that spelling in lower case.

```
zsc,1,12
znc,5
zsc,1,0C
znc,5
zsc,1,0c
znc,5
```

**T4.** Four more steps, to measure the step, then one step down.

```
zsc,1,12
znc,5
zsc,1,12
znc,5
zsc,1,12
znc,5
zsc,1,12
znc,5
zsc,1,13
znc,5
```

**T10.** Mute twice.

```
zsc,1,11
znc,5
zsc,1,11
znc,5
```

**T5.** Bass. `129` is hex `81` in decimal. `81` is the hex spelling.

```
zsc,1,129
znc,5
zsc,1,129
znc,5
zsc,1,81
znc,5
```

**T5.** Treble. `131` is hex `83` in decimal. `130` is hex `82`.

```
zsc,1,131
znc,5
zsc,1,130
znc,5
```

Sources 5 and 6 are in the decimal table and missing from the hex table.

```
zsc,1,05
znc,5
zsc,1,06
znc,5
```

**T16.** Padding. `1` and `01` should be the same command if padding is optional.

```
zsc,1,1
znc,5
zsc,1,01
znc,5
```

**T9 and T11.** Off, then a source select, then Off again, then Power twice. The source select sits between the two Offs so it is tried on a zone that was just turned off.

```
zsc,1,10
znc,5
zsc,1,01
znc,5
zsc,1,10
znc,5
zsc,1,49
znc,5
zsc,1,49
znc,5
```

Restore, using the source number from the baseline line (the fourth field). Repeat the volume command that moves toward the original level, and poll between repeats. If mute ended on, send `zsc,1,11` once more when T10 showed that the second press cleared it. Step bass with `128` or `129`, whichever T5 showed actually moves bass, until the bass field matches the baseline.

Optional ceiling check, after T6 has shown that `12` is volume up: send `zsc,1,12` and `znc,5` until two polls show the same volume, and keep a count. Then send `zsc,1,13` until the volume stops falling. That pair answers the 99-versus-100 question and the clamp-versus-error question. It walks the level through the whole range, so skip it if the room is in use. Return the volume afterward.

What decides the questions:

| Result | Resolves |
| --- | --- |
| Power field after Off, after one Power, and after a second Power | A9 |
| Power field after `zsc,1,01` when the zone was off | A11 |
| Volume after `12`, after `0C`, and after `0c` | A6. Also record the exact `rzsc` echo for A19 |
| Difference between consecutive volume numbers | A4 step size |
| Highest and lowest volume seen, and the reply at the end of the range | A4 span and clamp |
| Mute field after the first `11` and after the second | A10 |
| Exact bass and treble tokens, including the minus sign if one is present | A5 spelling |
| Which of `129` and `81` moves bass | A5 command spelling, and a second vote on A6 |
| Status source field after `05` and after `06` | Source codes the hex table left out |
| Echo of `zsc,1,1` versus `zsc,1,01` | A16 padding and A19 echo |

### Tuner batch

Needs the tuner display or a zone keypad, because the serial status line has no frequency. Write the station you see next to each reply. Skip the batch if no preset stations are stored, and say so. T7 and T8 can still be attempted. T14 can be attempted with no presets.

Pick the station the zone is on now and tune back to it at the end with whichever `src,11,...` form actually worked.

Tuner selected, and this zone is the active one.

```
znc,4,1
zsc,1,01
znc,5
```

**T14.** Documented frequency forms.

```
src,11,102.7
src,11,0560
```

**T14.** Forms the source never shows.

```
src,11,98.5
src,11,098.5
src,11,102.70
src,11,1027
src,11,560
src,11,1000
```

**T14.** These should fail. Reason 24 is "too many dots". Reason 25 is "must be all numbers".

```
src,11,102.7.1
src,11,10a.7
```

**T14.** Tune while the source is not the tuner.

```
zsc,1,02
znc,5
src,11,102.7
znc,5
```

Back to the tuner for the key tests.

```
zsc,1,01
znc,5
```

**T7.** Cursor-left / scan-down, then the hex spelling. Then cursor-right / scan-up.

```
zsc,1,25
zsc,1,19
zsc,1,1B
zsc,1,27
```

**T8.** Preset keys. `67` and `68` are the hex codes converted to decimal. `43` and `44` are Previous and Next in the decimal table.

```
zsc,1,67
zsc,1,43
zsc,1,68
zsc,1,44
```

**T14.** Active zone is 2. The tuner was last selected in zone 1.

```
znc,4,2
src,11,102.7
znc,4,1
```

On `zsc,1,25`, `19`, `27`, and `1B`, write down the frequency before and after, and whether audio paused or stopped. On `67`, `68`, `43`, and `44`, write down the preset or the frequency. `OK` by itself does not separate Previous from Next Preset.

What decides the questions:

| Result | Resolves |
| --- | --- |
| Which frequency strings return `OK`, and what the display shows for each | A14 grammar |
| `src,11,102.7` while source 2 is selected: `OK` or `FAIL`, and whether the source field changes | A14 precondition |
| `src,11,102.7` while zone 2 is the active zone: which tuner moves, if you can see both | A14, which zone owns the tune |
| Display after `25` versus after `19` | A7, and whether `19` was read as Pause |
| Display after `27` versus after `1B` | A7 and A6 |
| Display after `67`, `43`, `68`, `44` | A8 |

### Line endings

**T17.** Every command above already ends in a carriage return. That is the CR case. Paste one status reply exactly as the terminal shows it, including whether the receiver ends the line with a carriage return, a line feed, or both. A typed session cannot cleanly send a bare line feed, so the LF-only and CRLF cases are left out.

### Flag batch

Non-destructive probes for the repeat flag named in reason 9. Volume up is used because hold on volume up is already specified to fail, so the zone should not change when the flag is rejected. If a reply comes back `OK`, poll status and undo the volume step with `13`.

**T21.** `h` is hold. `r` is the guess for repeat. `x` is a nonsense flag.

```
znc,4,1
znt,12,h
zsc,1,12,r
znt,12,r
zsc,1,12,x
```

| Reply | Meaning |
| --- | --- |
| `FAIL,8` on `znt,12,h` | Matches the manual: volume up has no hold |
| `FAIL,9` on a suffix | That suffix is the repeat flag, and this command rejects it |
| `FAIL,10` or `FAIL,11` | The suffix is a flag, and not the repeat flag |
| `OK` | The suffix is legal here. Poll volume and step it back |

### Optional party batch

`znt,01,h` turns on every zone that is enabled for party mode and selects the tuner in those zones. Skip it unless every one of those rooms can come on.

Snapshot first. The core batch already knows whether zone 7 exists. Poll each zone you have, then fire the hold, then poll again. Extra `usc` lines between the command and the next prompt are the interesting part of A20.

```
znc,4,1
znc,5
znc,4,2
znc,5
znc,4,3
znc,5
znc,4,4
znc,5
znc,4,5
znc,5
znc,4,6
znc,5
znt,01,h
znc,4,1
znc,5
znc,4,2
znc,5
znc,4,3
znc,5
znc,4,4
znc,5
znc,4,5
znc,5
znc,4,6
znc,5
```

To put the rooms back, send `zsc,<zone>,10` to each zone that was off in the snapshot, and `zsc,<zone>,0<source>` to restore a source that changed. `znt,10,h` turns **every** zone off, including zones that were on before the test. Use it only when that is what you want.

### Optional listen

Confirms whether the port stays quiet (A22). With the terminal open, type nothing. Press one keypad key in zone 1 and wait a few seconds.

Silence supports A22. Any line is a message the protocol sheet does not describe. Paste it.

### Optional boot capture

Power-cycle the ZR-6 with the terminal already open and idle (A23). Leave it connected until at least ten seconds after the unit seems ready. `usc,1` is the documented ready marker. Everything before it is the undocumented diagnostic text. This interrupts audio in every zone on that chassis.

### What to paste back

1. How many ZR-6 chassis are linked, if you know.
2. The zone you used, if it was not zone 1.
3. The unedited transcript of each batch you ran.
4. For the tuner batch, the on-screen frequency after each command.
5. Anything you skipped, and why.

Those replies are enough to update the working assumptions in this guide: the status grammar, the command-number base, the tone-field spelling, power, mute, tune syntax, and the failure line. A24 stays untested.
