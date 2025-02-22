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
fn functionName(arg1: number?) {
    <expressions...>
}
```

Or allow for multiple input values:

```
fn functionName(arg1: location*) {
    <expressions...>
}
```

# Processes

Process can be created using the proc keyword:

```
proc functionName {
    <expressions...>
}
```

# Expressions

Every event, function or process contains a list of expressions.

## Actions

Actions have multiple types, determined by their block: player, entity, game, variable, control and select.
The general syntax is the character for one of the categories, followed by the actions name and arguments:

```
p.sendMessage("Test");
e.teleport(Location(0,0,0));
g.cancelEvent();
v.add(var, 1, 2);
c.wait(1);
s.eventTarget();
```

Game and entity actions can target different selectors:

```
p:default.sendMessage("Hi");
p:selection.sendMessage("Hi 1");
e:all.remove();
```

Tags can also be used:

```
p.sendMessage("Hi", alignmentMode="Centered");
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

The else keyword can be used on conditionals:

```
ifp selection:isNear(Location(0,0,0), 10) {
    <expressions...>
} else {
    <expressions...>
}
```

## Repeats

Repeats can be used similar to conditionals:

```
repeat forever() {
    <expressions...>
}
```

Repeat while can also be used:

```
repeat while(ifp isNear(Location(0, 0, 0), 1)) {
   <expressions...> 
}
```

## Function calls

Functions can also be called:

```
functionName(arg1, arg2, ...);
```
If the function is defined in the same file, its parameters are validated.

## Starting processes

Processes can be started as follows:

```
start("processName", localVariables="Copy", targetMode="With no targets");
```

---

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
123_456.7
```

More complex numbers can also be represented:

```
Number("%math(%var(test)+1)")
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

Potion(type as string or text, amplifier, duration)

```
Potion("strength", 2, 10)
```

## Particles

Particle(type as text, amount, horizontal_spread, verticle_spread, [tags])

```
Particle("Cloud", 1, 1, 0, motionVariation=50, motion=Vector(0, 1, 0))
```

## Items

Item(NBT)

```
Item("{Count:1b,DF_NBT:3700,id:\"minecraft:stone\",tag:{display:{Name:'{\"italic\":false,\"extra\":[{\"color\":\"green\",\"text\":\"A\"}],\"text\":\"\"}'}}}")
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
line var: `%default data`;
p.sendMessage(var);
```

Variables can be directly assigned a value:

```
line test = v.add(2, 2);
local example: `example variable` = v.equal(Location(0, 0, 0));
```