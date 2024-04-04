# LuaApi

Lua api for bureau plugins.

## hook

`hook.add(name: string, id: string, func: function)`

Adds a new callback for the given hook name.

`hook.remove(name: string, id: string)`

Remove callback for the given hook name and id pair.

`hook.call(name: string, ...any)`

Run a hook. Useful for letting other plugins know when something happens.

### Hooks

| Name | Args | Desc |
| --- | --- | --- |
| Think | | Called every time the bureau polls users. |
| UserConnecting | addr: string | Called when a client first connects, passes socket address. |
| NewUser | user: User, name: string, avatar: string | Called when a user fully connects to the bureau. |
| PositionUpdate | user: User, pos: Vector | Called when a user sends their position |
| TransformUpdate | user: User | Called when a user sends their transform |
| NameChange | user: User | Called every time someone changes their name |
| AvatarChange | user: User | Called every time someone changes their avatar |
| PrivateChat | user1: User, user2: User, msg: string | Called when user1 send a private chat message to user2 |
| AuraEnter | a: User, b: User | Called when user a enters user b's aura |
| AuraLeave | a: User, b: User | Called when user a leaves user b's aura |
| UserDisconnect | user: User | Called when a user disconnects |

## User

`User:disconnect()`

Disconnect the user from the bureau.

`User:getPeerAddr() -> string`

Get socket address of the user.

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

`Basis()`

Create a new basis.

