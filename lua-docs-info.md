# LDoc Lua Documentation Syntax Cheat Sheet

A concise overview of how to write LDoc‐style comments in Lua (and C) source files.  

---

## 1. Doc-Comment Styles

### 1.1. Single-line comments  
```lua
--- Summary sentence.
-- Optional extended description...
-- More description lines.
```

### 1.2. Block comments  
```lua
--[[--
Summary sentence.
Extended description...
--]]
```

### 1.3. Separator comments  
(Common “—––– text —–––” are ignored.)
```lua
--------------------
-- This also works
```

---

## 2. Comment Structure

1. **Summary sentence** ending in `.` or `?`  
2. *Blank line* (optional)  
3. **Detailed description** (one or more `--` lines)  
4. *Blank line* (optional)  
5. **Tags** (`@tag` lines), in any order  

---

## 3. Common Tags

### 3.1. Module-level Tags

| Tag         | Argument    | Description                              |
|-------------|-------------|------------------------------------------|
| `@module`   | _name_      | Declare a Lua module                     |
| `@classmod` | _name_      | Module exporting a single “class”        |
| `@submodule`| _name_      | Merge into a master module’s docs        |
| `@script`   | _name_      | Document a standalone script             |
| `@author`   | _text_      | Project author(s)                        |
| `@license`  | _text_      | License information                      |
| `@copyright`| _text_      | Copyright notice                         |
| `@release`  | _text_      | Release or version                       |

### 3.2. Function-level Tags

| Tag          | Arguments                  | Description                             |
|--------------|----------------------------|-----------------------------------------|
| `@function`  | _name_                     | Explicitly name a function (C exports)  |
| `@lfunction` | _name_                     | Local (non-exported) function           |
| `@param`     | _name_ _desc_              | Document a parameter                    |
| `@tparam`    | _type_ _name_ _desc_       | Typed parameter                         |
| `@return`    | _desc_                     | Document a return value                 |
| `@treturn`   | _type_ _desc_              | Typed return                            |
| `@raise`     | _desc_                     | Errors thrown                           |
| `@usage`     | _code_                     | Usage example                           |
| `@see`       | _ref_                      | See–also / cross-reference              |
| `@local`     | —                          | Mark as non-exported (unless `--all`)   |
| `@alias`     | _identifier_               | Alias a local table to the module       |

### 3.3. Table-level Tags

| Tag        | Arguments         | Description              |
|------------|-------------------|--------------------------|
| `@table`   | _name_            | Document a table         |
| `@field`   | _name_ _desc_     | Document a table field   |

### 3.4. Sections & Classes

| Tag           | Argument    | Description                              |
|---------------|-------------|------------------------------------------|
| `@section`    | _name_      | Start a new logical section              |
| `@type`       | _name_      | Document a “class” / type section        |
| `@within`     | _section_   | Place item into an existing section      |

### 3.5. Annotations (for internal notes)

- `@todo`  
- `@fixme`  
- `@warning`  

---

## 4. Inline Extraction Patterns

### 4.1. Inline parameter docs  
```lua
--- Compute something.
function foo(
  a,   -- description of a
  b    -- description of b
)
  ...
end
```

### 4.2. Table-constructor field docs  
```lua
--- Constant values.
M.consts = {
  alpha = 0.23, -- first correction
  beta  = 0.44, -- second correction
}
```

---

## 5. References & Linking

- **`@see target`**  
- **Inline**: `@{target}` or `@{target|link text}`  
- **Backticks**: `` `target` ``  
  _(if `backtick_references = true` or using Markdown)_

Unresolved references warn you and render as `???`.

---

## 6. Colon-style Tags

Enable with `--colon` or `colon = true` in `config.ld`:

```lua
--- Check a person.
-- string:name    Person’s name
-- int:age        Age of person
-- !Person:ret    Resulting object
-- function:check(name, age)
```

---

## 7. Basic Examples

### 7.1. Documenting a Module

```lua
--- My awesome module.
-- Provides utilities for XYZ.
-- @module awesome
-- @author Jane Doe
local M = {}

--- Split a string.
-- @param s   The input string.
-- @param sep The separator (default `" "`).
-- @return     Table of substrings.
function M.split(s, sep) … end

return M
```

### 7.2. Documenting a Class

```lua
--- File handle class.
-- @classmod File

function File:read(n)
  -- @param n Number of bytes.
  -- @return  String or `nil, err`.
end
```

---

## 8. Configuration (`config.ld`)

```lua
return {
  format    = 'markdown',    -- 'plain' | 'discount'
  package   = 'my.pkg',      -- base package for resolving @{refs}
  examples  = {'examples'},  -- pretty‐print examples/
  readme    = 'README.md',   -- narrative topics
  new_type('macro','Macros'),-- add @macro tag
  alias('p','param'),        -- allow @p as @param
  colon     = true,          -- enable colon‐style tags
  all       = true,          -- document local funcs
  sort      = true,          -- sort items alphabetically
}
```

---

## 9. Generating Documentation

```bash
# Default: reads config.ld (if present), docs ./ → ./doc
ldoc .

# Custom output directory, include locals, sort
ldoc --dir=doc --all --sort src/

# Single‐file output
ldoc --output foo --dir=html foo.lua
```

---

> _For a full reference, consult the LDoc manual or `ldoc --help`._
