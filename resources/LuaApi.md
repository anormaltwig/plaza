# LuaApi

Lua api for bureau plugins.

## hook

```lua
local hook = require("hook")
```

`hook.think(fn: fun())`

`hook.user_connect(fn: fun(addr: string):boolean?)`

`hook.new_user(fn: fun(user: User, name: string, avatar: string))`

`hook.position_update(fn: fun(user: User, pos: Vector))`

`hook.transform_update(fn: fun(user: User))`

`hook.chat_send(fn: fun(user: User, msg: string):string?)`

`hook.name_change(fn: fun(user: User, name: string, old: string))`

`hook.avatar_change(fn: fun(user: User, avatar: string, old: string))`

`hook.private_chat(fn: fun(sender: User, receiver: User, msg: string):string?)`

`hook.user_disconnect(fn: fun(user: User))`

`hook.plugins_loaded(fn: fun())`

## User

`User:disconnect()`

Disconnect the user from the bureau.

`User:set_pos(pos: Vector)`

Set User's position.

`User:pos() -> Vector`

Get User's position.

`User:set_rot(rot: Basis)`

Set User's rotation.

`User:rot() -> Basis`

Get User's rotation.

`User:send_msg(msg: string)`

Send a message to the User's chat.

## users

```lua
local users = require("users")
```

`users.all() -> User[]`

Get all connected users.

`users.get(id: number) -> User`

Get user by their id.

## Vector

```lua
local Vector = require("vector")
```

`Vector:length_sqr() -> number`

Get the length of the vector squared. (Faster than getting the actual length)

`Vector:length() -> number`

Get the length of the vector.

`Vector:normalized() -> Vector`

Create a new vector with the same direction but a length of 1.

`Vector:normalize()`

Modify the vector so that its length is 1.

`Vector:clone() -> Vector`

Clone vector.

`Vector(x: number, y: number, z: number) -> Vector`

Create a new vector, you can get x, y, and z components by indexing 1, 2, and 3 respectively.

## Basis

```lua
local Basis = require("basis")
```

`Basis:set(arr: number[])`

Set values of the basis.

`Basis:from_yaw(r)`

Set values based on a given angle. Only use this function on a new basis.

`Basis:scale() -> Vector`

Gets scale of the basis.

`Basis:scale_by(n)`

Multiplies every value in the basis by n.

`Basis:set_scale(v)`

Sets the scale of the basis.

`Basis:clone() -> Basis`

Clones the basis.

`Basis()`

Create a new basis.

