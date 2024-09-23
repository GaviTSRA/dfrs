# dfrs
dfrs is a text-based programming language for the DiamondFire Minecraft server. It allows you to create code templates from text based code.
For more information, see [the syntax doc](syntax.md).

## Usage
> [!WARNING]  
> dfrs is currently in beta and may not function correctly.  
> Make sure to back up any important code before replacing it with dfrs-generated code.

To use dfrs, download the latest release from [the releases](https://github.com/GaviTSRA/dfrs/releases). Make sure the executable is in your path.   
Now, create a new project using `dfrs init <path>`   
Start writing your code in a `.dfrs` file.   
To send the code to minecraft, you will need to have CodeClient or Recode installed. Select the API you want to use in your `dfrs.toml` [configuration file](#Configuration).   
To compile the code and send it, run `dfrs compile <file>`.

If you already have code on DiamondFire but want to switch to using dfrs, you can quickly do so by using `dfrs decompile <codestring>`.
Currently, you need to use a mod like NBT Editor to extract the codestring from a template.

## Configuration
A projects configuration is stored in its dfrs.toml.  
Available configs:
- sending
    - api: Which API to use when sending templates. Either "recode" or "codeclient"

## Current limitations
- Documentation is lacking
- The extension is not ready for use
- Variables are not type checked
- There is no way to represent items
- Processes cannot be created or called
- Actions with multiple possible arguments are not correctly type checked
- Function calls are not validated
- Potion names and sound names are not validated
- Particles cannot be represented
- Error handling is lacking
- Some argument types are not implemented
- Multiline errors are not properly printed