# Rhai Language Reference

Rhai is an embedded scripting language for Rust. In Undone, Rhai scripts define scene logic and event callbacks. The Rust engine registers custom functions, types, and globals that scripts can call.

## Type System

Rhai is dynamically typed. All variables are `Dynamic` at runtime.

Common types: `INT` (i64), `FLOAT` (f64), `bool`, `String`, `Array`, `Map`, and custom types registered by the engine.

```rhai
let x = 42;
let name = "Alice";
let arr = [1, 2, 3];
let map = #{ key: "value" };
```

## Functions

```rhai
fn greet(name) {
    `Hello, ${name}!`   // backtick string interpolation
}

let double = |x| x * 2;  // closure
```

## Control Flow

```rhai
if condition { ... } else if other { ... } else { ... }

for item in array { ... }

while condition { ... }

loop { if done { break; } }
```

## Module System

```rhai
import "module_name" as m;
m::function();
```

## Checking What's Available

Before calling a function in a script, use the `rhai_list_registered_api` MCP tool to confirm it exists in the engine. Use `rhai_validate_script` (file path) or `rhai_check_syntax` (source string) to validate before saving.

## Anti-Patterns

**Unbounded loops** — Rhai has a configurable operation limit. Always include a break condition.

**Variable shadowing** — Rhai allows shadowing within blocks; the outer variable is unchanged after the block exits:
```rhai
let x = 1;
{ let x = 2; }  // x is still 1 here
```

**Type coercion** — Rhai does not coerce. `1 + "2"` is a runtime error, not `"12"`.

**String mutation** — `s.push('!')` mutates in-place. `s = s + "!"` creates a new string.
