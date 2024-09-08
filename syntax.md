# Events
Events can be created using an @ and the name of the event like this:
```
@EVENTNAME {
    <expressions...>
}
```
Events can be automatically LS-Cancelled with an !:
```
@swapHands! {
    <expressions...>
}
```

# Functions
Functions can be created using the fn keyword:
```
fn functionName() {
    <expressions...>
}
```
They can take in arguments, which require a type:
```
fn functionName(arg1: any) {
    <expressions...>
}
```
Function arguments can be optional:
```
fn functionName(arg1?: number) {
    <expressions...>
}
```
Or allow for multiple input values:
```
fn functionName(arg1*: location) {
    <expressions...>
}
```

# Processes
TODO

---
# Expressions
Every event, function or process contains a list of expressions.
## Actions
Actions can either target players, entities, the game or variables.
The general syntax is the character for one of the categories above, followed by the actions name and arguments:
```
p.sendMessage("Test");
e.teleport(Location(0,0,0));
g.cancelEvent();
v.add(var, 1, 2);
```
Game and entity actions can target different selectors:
```
p:default.sendMessage("Hi");
p:selection.sendMessage("Hi 1");
e:all.remove();
```
## Conditionals
Conditional statements function the same, but they have a slightly different syntax.
The character denoting their target is prefixed by an if and followed by whitespace instead of a dot.
```
ifp isNear(Location(0,0,0), 10) {
    <expressions...>
}
```
They can be inverted using an exclamation mark:
```
ifp !isNear(Location(0,0,0), 10) {
    <expressions...>
}
```
Conditionals can also target different selectors:
```
ifp selection:isNear(Location(0,0,0), 10) {
    <expressions...>
}
```

# Values
## Text
```
"This is <yellow>text"
```
## Strings
```
'This is a string'
```
## Numbers
```
5
5.2
```
## Locations
Location(x, y, z, pitch?, yaw?)
```
Location(1, 1, 1, 0, 0)
Location(1, 1, 1)
```
## Vectors
Vector(x, y, z)
```
Vector(1, 2, 3)
```
## Sounds
Sound(name as string or text, volume, pitch)
```
Sound("Cow Ambient", 1, 2) 
```

## Potions
Sound(type as string or text, amplifier, duration)
```
Potion("strength", 2, 10)
```

## Game values
```
$name
$selection:name
```
## Variables
Variables need to be declared before they are used.
Line and local variables are declared inside the function or event they are used.
```
line var;
local var2;
p.sendMessage(var, var2);
```
Game and saved variables are declared at the top of the file:
```
game players;
save levels;

@join {
    p.sendMessage(levels);
}
```
The way variables are named on DF can be overridden:
```
line var = `%default data`;
p.sendMessage(var);
```

# Current limitations
- Variables are currently not type checked
- There is currently no way to represent items
- Codeblocks are missing