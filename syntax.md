file       : [event, function, process]
event      : [expression]
function   : [expression]
process    : [expression]
expression : action | varAction
action     : 
varAction  : targetVar operation values

# Tokens
## Values
- `NUMBER` 1
- `STRING` 'string'
- `TEXT` "\<blue>Text\</blue>"
- `VECTOR` |2, 3, 5|
- `LOCATION` <5, 20, 30, 2, 3.2>
- `VARIABLE` g\`my_var\`
- `SOUND` Sound("sound_name", 1, 2)
- `EFFECT` Effect("effect_name", 1, 2)
- `PARTICLE` TODO
- `GAME_VALUE` $s:value_name
- `ENTITY` Entity("bat")

## Keywords / Symbols
- `AT` @
- `PLUS` +
- `MINUS` -
- `MULTIPLY` *
- `DIVIDE` /

## Other
- `OPEN_PAREN` (
- `CLOSE_PAREN` )
- `OPEN_BLOCK` {
- `CLOSE_BLOCK` }


# Events
```
@(p|e):EVENT {
    EXPR...
}
```

# Functions
```
fn NAME(ARGS...) {
    EXPR...
}
```

# Processes
```
proc NAME {
    EXPR...
}
```