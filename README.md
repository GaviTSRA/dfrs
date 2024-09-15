# dfrs
dfrs is a text-based programming language for the DiamondFire Minecraft server. It allows you to create code templates from text based code.
For more information, see [the syntax doc](syntax.md).

## Usage
> **Warning**  
> dfrs is currently in beta and may not function correctly.  
> Make sure to backup any important code before replacing it with dfrs-generated code.

To use dfrs you need to build the source code of the cli and put the executable in your path. Afterwards you can use the dfrs command to compile .dfrs files. You also need either recode or codeclient to receive the compiled templates. Which API is used can be configured in the dfrs.toml configuration file. This file can be created using `dfrs init <path>`. Files can be compiled using `dfrs compile <path>`.

## Configuration
A projects configuration is stored in its dfrs.toml.  
Available configs:
- sending
    - api: Which API to use when sending templates. Either "recode" or "codeclient"

## Current limitations
- No proper release is available
- Documentation is lacking
- Variables are not type checked
- There is no way to represent items
- Processes cannot be created or called
- Actions with multiple possible arguemnts are not correctly type checked
- Function calls are not validated
- Potion names, sound names and particles names are not validated
- Events are not loaded from action dump
- Error handling is lacking
- Some argument types are not implemented
- Multiline errors are not properly printed