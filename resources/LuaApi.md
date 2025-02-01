# LuaApi

Lua api for bureau plugins.

## hook

`hook.onThink(fn: fun())`

`hook.onUserConnect(fn: fun(addr: string):boolean?)`

`hook.onNewUser(fn: fun())`

`hook.onPositionUpdate(fn: fun(user: User, pos: Vector))`

`hook.onTransformUpdate(fn: fun(user: User))`

`hook.onChatSend(fn: fun(user: User, msg: string):string?)`

`hook.onNameChange(fn: fun(user: User, name: string, old: string))`

`hook.onAvatarChange(fn: fun(user: User, avatar: string, old: string))`

`hook.onPrivateChat(fn: fun(sender: User, receiver: User, msg: string):string?)`

`hook.onUserDisconnect(fn: fun(user: User))`

`hook.onPluginsLoaded(fn: fun())`

## User

`User:disconnect()`

Disconnect the user from the bureau.

`User:setPos(pos: Vector)`

Set User's position.

`User:getPos() -> Vector`

Get User's position.

`User:setRot(rot: Basis)`

Set User's rotation.

`User:getRot() -> Basis`

Get User's rotation.

`User:sendMsg(msg: string)`

Send a message to the User's chat.

## user_manager

`user_manager.getAll() -> User[]`

Get all connected users.

`user_manager.get(id: number) -> User`

Get user by their id.

## Vector

`Vector:getLengthSqr() -> number`

Get the length of the vector squared. (Faster than getting the actual length)

`Vector:getLength() -> number`

Get the length of the vector.

`Vector:getNormalized() -> Vector`

Create a new vector with the same direction but a length of 1.

`Vector:normalize()`

Modify the vector so that its length is 1.

`Vector:clone() -> Vector`

Clone vector.

`Vector(x: number, y: number, z: number) -> Vector`

Create a new vector, you can get x, y, and z components by indexing 1, 2, and 3 respectively.

## Basis

`Basis:set(arr: number[])`

Set values of the basis.

`Basis:fromYaw(r)`

Set values based on a given angle. Only use this function on a new basis.

`Basis:scale(n)`

Multiplies every value in the basis by n.

`Basis:getScale() -> Vector`

Gets scale of the basis.

`Basis:setScale(v)`

Sets the scale of the basis.

`Basis:clone() -> Basis`

Clones the basis.

`Basis()`

Create a new basis.

