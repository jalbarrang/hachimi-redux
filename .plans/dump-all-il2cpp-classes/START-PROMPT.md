# Dump All IL2CPP Classes — Implementation Prompt

## Goal

Add a feature to the Training Tracker plugin that enumerates **every** IL2CPP class in the game and writes full introspection (namespace, name, fields with types, methods with signatures) to `il2cpp_classes.txt` next to the game executable. Triggered by a menu button.

## Codebase Layout

```
plugins/training-tracker/
  Cargo.toml                    # Plugin crate (cdylib)
  src/
    lib.rs                      # Plugin entry, mod declarations
    ui.rs                       # Menu + overlay rendering
    diagnostics.rs              # Existing class probing (DO NOT modify heavily)
    skill_shop.rs, hooks.rs, memory_reader.rs, ...
crates/hachimi-plugin-sdk/src/
    il2cpp.rs                   # SDK: resolve_symbol, get_assembly_image, class_get_methods, etc.
    gui.rs                      # SDK: gui_button, gui_heading, gui_small, etc.
    sdk.rs, lib.rs              # SDK core
```

## Key APIs Available

The plugin SDK (`Sdk::get()`) provides:
- `sdk.resolve_symbol("il2cpp_xxx")` → `Option<*mut c_void>` — resolves any IL2CPP export
- `sdk.class_get_methods(klass, &mut iter)` → `*const MethodInfo` — iterate methods
- `sdk.gui_button(ui, text)` → `bool` — menu button
- `sdk.show_notification(msg)` — toast

### IL2CPP symbols to resolve via `resolve_symbol`:

| Symbol | Signature | Purpose |
|--------|-----------|---------|
| `il2cpp_domain_get` | `() → *mut Domain` | Get runtime domain |
| `il2cpp_domain_get_assemblies` | `(domain, &mut usize) → *mut *const Assembly` | List all assemblies |
| `il2cpp_assembly_get_image` | `(assembly) → *const Image` | Assembly → image |
| `il2cpp_image_get_name` | `(image) → *const c_char` | Image/DLL name |
| `il2cpp_image_get_class_count` | `(image) → usize` | Total classes in image |
| `il2cpp_image_get_class` | `(image, index) → *const Class` | Get class by index |
| `il2cpp_class_get_name` | `(klass) → *const c_char` | Class name |
| `il2cpp_class_get_namespace` | `(klass) → *const c_char` | Namespace |
| `il2cpp_class_get_fields` | `(klass, &mut iter) → *mut FieldInfo` | Iterate fields |
| `il2cpp_type_get_name` | `(type) → *mut c_char` | Type name (ALLOCATED — must free) |
| `il2cpp_free` | `(ptr)` | Free allocated strings |

`il2cpp_class_get_methods` is already wrapped in the SDK as `sdk.class_get_methods()`.

### Existing patterns in `diagnostics.rs`

- `TypeIntrospection` struct resolves `type_get_name`, `class_get_name`, `class_get_fields`, `il2cpp_free`
- `dump_all_fields(label, klass, &introspect)` iterates fields via `class_get_fields` with `FieldInfoCompat` struct
- `dump_methods(label, klass, introspect)` iterates methods via `sdk.class_get_methods` with `MethodInfoCompat` struct
- Both use `FieldInfoCompat` and `MethodInfoCompat` repr(C) structs for reading name/type pointers

**FieldInfoCompat layout** (from diagnostics.rs):
```rust
#[repr(C)]
struct FieldInfoCompat {
    name: *const c_char,
    type_: *const c_void, // Il2CppType*
}
```

**MethodInfoCompat layout** (from diagnostics.rs):
```rust
#[repr(C)]
struct MethodInfoCompat {
    method_pointer: usize,
    virtual_method_pointer: usize,
    invoker_method: usize,
    name: *const c_char,
    klass: *mut c_void,
    return_type: *const c_void,
    parameters: *mut c_void,
    _union1: usize,
    _union2: usize,
    token: u32,
    flags: u16,
    iflags: u16,
    slot: u16,
    parameters_count: u8,
}
```

### Menu section in `ui.rs`

The menu is drawn in `draw_menu_section_inner(ui)`. Add the button there, after the existing "Show Training Overlay" button block (~line 68).

### Logging macro

Use `hlog_info!("message")` / `hlog_error!("message")` — available via `#[macro_use] extern crate hachimi_plugin_abi` in lib.rs.

## Steps

### Step 1: Create `plugins/training-tracker/src/class_dump.rs` [DONE:1]

New module with:

```rust
//! Full IL2CPP class enumeration — writes all classes, fields, and methods to a file.

use std::ffi::{c_char, c_void, CStr};
use std::io::{BufWriter, Write};
```

**`DumpContext` struct** — resolves all needed IL2CPP symbols in one shot:
- `domain_get: fn() → *mut c_void`
- `domain_get_assemblies: fn(*const c_void, *mut usize) → *mut *const c_void`
- `assembly_get_image: fn(*const c_void) → *const c_void`
- `image_get_name: fn(*const c_void) → *const c_char`
- `image_get_class_count: fn(*const c_void) → usize`
- `image_get_class: fn(*const c_void, usize) → *mut c_void`
- `class_get_name: fn(*mut c_void) → *const c_char`
- `class_get_namespace: fn(*mut c_void) → *const c_char`
- `class_get_fields: fn(*mut c_void, *mut *mut c_void) → *mut c_void`
- `type_get_name: fn(*const c_void) → *mut c_char`
- `il2cpp_free: fn(*mut c_void)`

All resolved from `Sdk::get().resolve_symbol(...)`. Constructor returns `Option<Self>` — None if any symbol fails.

**`pub fn dump_all_classes()`** — main entry point:
1. Resolve `DumpContext`. If fails, `hlog_error!` and return.
2. Get domain via `domain_get()`.
3. Get assemblies via `domain_get_assemblies(domain, &mut count)`.
4. Determine output path: `std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.join("il2cpp_classes.txt")))`. Fallback to `"il2cpp_classes.txt"`.
5. Open file with `File::create`, wrap in `BufWriter`.
6. For each assembly `i` in `0..count`:
   - Read assembly pointer: `*assemblies.add(i)`
   - Get image, image name
   - Get class count
   - Write header line
   - For each class `j` in `0..class_count`:
     - Get klass, read namespace + name (CStr, null-check, empty-string fallback)
     - Write `\n[{namespace}] {name}\n`
     - Iterate fields: loop `class_get_fields(klass, &mut iter)` until null. Read `FieldInfoCompat`, get type name via `type_get_name` (remember to free!), write `  field: {type} {name}\n`. Cap at 500 fields per class.
     - Iterate methods: loop `sdk.class_get_methods(klass.cast(), &mut iter)` until null. Read `MethodInfoCompat`, get return type name, write `  method: {ret_type} {name}({param_count} args)\n`. Cap at 500 methods per class.
   - Track total class count.
7. Log summary: `hlog_info!("Dumped {total} classes from {asm_count} assemblies → {path}")`.

**Important safety notes:**
- Every pointer from IL2CPP must be null-checked before dereferencing
- `type_get_name` returns an allocated `*mut c_char` — must call `il2cpp_free` after converting to String
- `class_get_name` and `class_get_namespace` return static pointers — do NOT free
- `image_get_name` returns a static pointer — do NOT free
- All function pointer transmutes need `unsafe` blocks with SAFETY comments

### Step 2: Register module in `lib.rs` [DONE:2]

Add `mod class_dump;` in the module list (after `mod diagnostics;`).

### Step 3: Add menu button in `ui.rs` [DONE:3]

In `draw_menu_section_inner`, after the "Show Training Overlay" button block, add:

```rust
if sdk.gui_button(ui, "\u{1f4cb} Dump All IL2CPP Classes") {
    class_dump::dump_all_classes();
    sdk.show_notification("Class dump complete — see il2cpp_classes.txt");
}
```

### Step 4: Run clippy, fmt, tests [DONE:4]

```bash
cargo fmt --manifest-path plugins/training-tracker/Cargo.toml
cargo clippy --manifest-path plugins/training-tracker/Cargo.toml -- -D warnings
cargo test --manifest-path plugins/training-tracker/Cargo.toml
```

Fix any issues. The new module won't have unit tests (it's all unsafe IL2CPP FFI), but existing tests must still pass.

## Constraints

- **AGENTS.md**: Never launch the game. Never kill game processes. Never modify `cri_mana_vpx.dll.backup`.
- **AGENTS.md**: Run clippy and rust-fmt after writing code.
- **AGENTS.md**: Less code = better. Keep the new file focused; don't add features beyond what's specified.
- **diagnostics.rs** already has `FieldInfoCompat` and `MethodInfoCompat` structs. Either make them `pub(crate)` and import, or duplicate them in the new file (duplicating is acceptable since they're small repr(C) structs and avoids coupling). Prefer making them pub(crate) if it's a 1-line change in diagnostics.rs.
- Field iteration cap: 500 fields per class, 500 methods per class (safety valve).
- `il2cpp_type_get_name` result MUST be freed with `il2cpp_free`. Other name getters return static pointers.
