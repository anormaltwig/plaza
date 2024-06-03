# VSCP Packet Reference

This document serves as a corrected/improved version of [another VSCP Protocol doc](https://github.com/LeadRDRK/OpenBureau/blob/9e43ba72e22bc997808502a316eaa73d5203a5f4/docs/Protocol.md), some things may be missing.

## Packet Layout

### Format

Packets will be displayed in the following format

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| A | 4 | uint32 | This does A |
| B | 2 | int16 | This is for B |
| C | ~ | string | |
| D | 1 | uint8 | |

### Types

| Type | Description |
| --- | --- |
| uint32 | 32 bit unsigned integer |
| uint16 | 16 bit unsigned integer |
| uint8 | 8 bit unsigned integer |
| int32 | 32 bit signed integer |
| int16 | 16 bit signed integer |
| int8 | 8 bit signed integer |
| int32float | 32 bit signed integer representing a float that has been multiplied by 0xFFFF (65535) and rounded |
| string | Sequence of characters termitated by null |
| data | Arbitrary or unknown data |

- All numbers are in Big Endian order.

## Initial Connection

### Hello (Initial)

The first packet sent by a connected client contains "hello" as well as two `uint8`s, assumed to represent the browser version.

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| Hello | 5 | data | Always `hello` |
| VscpVerMajor | 1 | int8 | |
| VscpVerMinor | 1 | int8 | |

### Hello (Server Response)

After the initial hello packet is sent by a client the server will respond with its own hello packet.

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| Hello | 5 | data | Always `hello` |
| ??? | 4 | int32 | Unknown uint32, seems to always be 0 |
| Connection ID | 4 | int32 | Client's unique ID |

## Packets

Every packet starts with a uint8 defining what type of packet it is followed immediately by that data.

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| Section Type | 1 | uint8 | Defines what packet is being sent. |

### General Messages

`Section Type` = `0`

`Byte Size` = `17` + `Content Size`

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| Section Type | 1 | uint8 | Defines what packet is being sent. |
| ID 1 | 4 | int32 | Set to Connection ID when sent from client. Depends on the opcode when sent from the server. |
| ID 2 | 4 | int32 | Set to Client ID when sent from client. Depends on the opcode when sent from the server. |
| Opcode | 4 | uint32 |  |
| Content Size | 4 | uint32 | Byte count of the rest of the packet. |
| Content | `Content Size` | data | Packet content |

### Sys1 Message

`Section Type` = `1`

`Byte Size` = `14`

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| ??? | 14 | data | |

- General Message and Position Update are also internally called 'Sys0 Message' and 'Sys2 Message' respectively by the original bureau, but were renamed since they are both nicer names, and because they were named that way before I started looking through decompiled code.

- Unknown purpose, decompiled community place browser code has debug logs referencing "kinemation" bodies and frames. Maybe for syncing animations?

- Byte Size of 14 is a guess based on numbers used in the decompiled code.

- Safe to just ignore entirely.

### Position Update

`Section Type` = `2`

`Byte Size` = `27`

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| Section Type | 1 | uint8 | Defines what packet is being sent. |
| Connection Id | 4 | int32 | |
| Client Id | 4 | int32 | |
| Broadcast Id | 4 | int32 | |
| X | 4 | int32float | |
| Y | 4 | int32float | |
| Z | 4 | int32float | |
| ??? | 2 | data | Usually just `0x0100` |

## General Message Opcodes

| Name | Value |
| --- | --- |
| CMsgNewUser | 0 |
| SMsgClientId | 1 |
| SMsgUserJoined | 2 |
| SMsgUserLeft | 3 |
| SMsgBroadcastId | 4 |
| MsgCommon | 6 |
| CMsgStateChange | 7 |
| SMsgSetMaster | 8 |
| SMsgUserCount | 11 |

### CMsgNewUser

| Name | Bytes | Type |
| --- | --- | --- |
| Username | ~ | string |
| Avatar | ~ | string |

### SMsgClientId 

| Name | Bytes | Type |
| --- | --- | --- |
| ClientId | 4 | int32 |

### SMsgUserJoined

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| BroadcastId | 4 | int32 | |
| ??? | 4 | int32 | Differs between users, fine to just set to the broadcast id |
| Avatar | ~ | string | |
| Username | ~ | string | |

- Every time a user receives the SMsgUserJoined for when they joined, a new timer for networking their position and rotation starts. You can technically get them to send their position as fast as you want with this for smother movement and rotation if you can time it right.

### SMsgUserLeft

| Name | Bytes | Type |
| --- | --- | --- |
| BroadcastId | 4 | int32 |

### SMsgBroadcastId

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| BroadcastId | 4 | int32 | |

### MsgCommon

| Name | Bytes | Type |
| --- | --- | --- |
| BroadcastId | 4 | int32 |
| MsgType | 4 | uint32 |
| Strategy | 1 | uint8 |
| Content | ~ | data |

### CMsgStateChange

| Name | Bytes | Type |
| --- | --- | --- |
| State | 1 | u8 |

| State | Value |
| --- | --- |
| NotConnected | 0 |
| Connecting | 1 |
| Connected | 2 |
| Disconnected | 3 |
| Active | 4 |
| Sleep | 5 |

### SMsgSetMaster

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| IsMaster | 1 | uint8 | Set to 1 when they are the master, 0 when they arn't. |

- Pretty much determintes whether the amIMaster vscp api function returns true or false.

### SMsgUserCount

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| ??? | 1 | uint8 | Always set to `1` |
| Count | 4 | int32 | Current user count |

## Message Common Types

| Name | Value | Possible Strategies |
| --- | --- | --- |
| TransformUpdate | 2 | 0, 1, 2, 3, 4 |
| ChatSend | 9 | 0 |
| CharacterUpdate | 12 | 1 |
| NameChange | 13 | 1 |
| AvatarChange | 14 | 1 |
| PrivateChat | 15 | 2 |
| VcRegister | 16 | ? |
| VoiceState | 18 | 1 |
| ??? | 19 | ? |
| ApplSpecific | 10000 | 0, 1, 2, 3, 4, 5, 6 |

### Message Common Strategies

| Value | Description |
| --- | --- |
| 0 | All clients in aura. (Including sender) |
| 1 | All clients in aura. (Excluding sender) |
| 2 | Responder only, send to ID specified in MsgCommon header. |
| 3 | All clients. (Including sender) |
| 4 | All clients. (Excluding sender) |
| 5 | Unused/Unknown |
| 6 | Unused/Unknown |

- In the original bureau, most message common types have a strategy value they expect, but only log a warning and keep going if its wrong.

- ApplSpecific seems to get called for any unused type value in the original bureau.

- ApplSpecific has different behaviour if the id sent in MsgCommon is `-9999`

### TransformUpdate

| Name | Bytes | Type |
| --- | --- | --- |
| 1 | 4 | int32float |
| ... | | |
| 9 | 4 | int32float |
| X | 4 | int32float |
| Y | 4 | int32float |
| Z | 4 | int32float |

- TransformUpdate is a 4x3 matrix where the first 9 values make up the rotation matrix and the last three are a position offset.

### ChatSend

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| Message | ~ | string | The user's name + ": " followed by their actual message. |

### CharacterUpdate

| Name | Bytes | Type |
| --- | --- | --- |
| Data | ~ | string |

- The data will look something like `sleep:0 1:000000000000:58:0:`.

- The first part (sleep:0) tells whether the character is sleeping or not (1 for true, 0 for false)
- The second part contains the character data, each part separated by a colon:
    - The first value (1) is the number of the avatar.
    - Next is the body parts's color and scale:
        - Each body part has two values, the first value is the color, the second value is the scale.
        - The amount of values depends on how much body parts the user's avatar has.
    - After that is the minutes spent using that particular avatar (58)
    - The final value is user's medal, which is awarded if they have spent enough time using that avatar (none = 0, happy = 1, lucky = 2, lovely = 3).

### NameChange

| Name | Bytes | Type |
| --- | --- | --- |
| Name | ~ | string |

### AvatarChange

| Name | Bytes | Type |
| --- | --- | --- |
| Avatar | ~ | string |

### PrivateChat

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| BroadcastId | 4 | int32 | The BroadcastId of the user who sent the message. |
| Message | ~ | string | |

#### Special Messages

| Name | Description |
| --- | --- |
| %%REQ | First message sent by a user to request a private chat, also used as a keep-alive |
| %%RINGING | Tell requesting user that they're being prompted for the request |
| %%REJECT | Reject request |
| %%ACCEPT | Accept request |
| %%OK | Reply to `%%REQ` keep-alive |
| %%BUSY | Tell requesting user that they are already in a private chat. |
| %%SILENT | ??? Exists in code but doesn't seem to be sent anywhere. Any user receiving it results in 'Cannot Connect' being printed to their private chat window. |

- The id in the MsgCommon header is the id of the user the message is to be sent to.

### ApplSpecific

| Name | Bytes | Type | Description |
| --- | --- | --- | --- |
| Unknown | 1 | uint8 | Seems to always be set to `2` |
| Method | ~ | string | |
| StrArg | ~ | string | |
| IntArg | 4 | int32 | Usually set to the sender's Broadcast Id |

- This is the worst thing ever to decode.

- If the id in MsgCommon is `-9999`, then there is seperate behaviour for handling the packet. Most notibly strategy 2 is now for requesting to a master client.

- The master client is a (randomly assigned?) client that is responsible for responding to certain ApplSpecific messages (startAreaRequest and broadcastRequest are the only ones i've seen so far). I assume it uses these to sync vrml events. If unhandled things like the intro camera animation in coast will not play.

# Credits

- [LeadRDRK](https://github.com/LeadRDRK), for the original packet structure.

- [barra](https://barrarchiverio.7m.pl/), for character update data.

