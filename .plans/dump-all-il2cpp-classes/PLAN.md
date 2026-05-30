# Dump All IL2CPP Classes

Add a "Dump All Classes" button to the Training Tracker menu that enumerates every class across all loaded IL2CPP assemblies and writes full introspection (namespace, name, fields with types, methods with signatures) to a dedicated file.

## Context

Currently `diagnostics.rs` only probes hardcoded class lists (`PROBE_CLASSES`, `SKILL_CLASSES`, `DEEP_DIVE_CLASSES`). The IL2CPP runtime exposes enumeration APIs that the plugin can access via `Sdk::resolve_symbol()`:

- `il2cpp_domain_get()` → `*mut Il2CppDomain`
- `il2cpp_domain_get_assemblies(domain, &mut size)` → `*mut *const Il2CppAssembly` + count
- `il2cpp_assembly_get_image(assembly)` → `*const Il2CppImage`
- `il2cpp_image_get_name(image)` → `*const c_char`
- `il2cpp_image_get_class_count(image)` → `usize`
- `il2cpp_image_get_class(image, index)` → `*const Il2CppClass`
- `il2cpp_class_get_name(klass)` → `*const c_char`
- `il2cpp_class_get_namespace(klass)` → `*const c_char`

Plus existing resolved symbols: `il2cpp_class_get_fields`, `il2cpp_type_get_name`, `il2cpp_class_get_methods` (already used in `TypeIntrospection` and `dump_methods`).

All symbols are in `src/windows/symbols_impl.rs` SYMBOL_LIST, so `resolve_symbol` will find them.

**Output destination**: Dedicated file (`il2cpp_classes.txt`) next to the game executable (same dir as `hachimi.log`). Only a summary line logged to `hachimi.log`.

**Trigger**: "Dump All Classes" button in the Training Tracker menu section.

**diagnostics.rs is 627 lines** — this feature adds significant code (assembly enumeration + file writer). Per AGENTS.md, decompose into a new file.

## Plan:

1. **Create `plugins/training-tracker/src/class_dump.rs`** — new module for full IL2CPP enumeration.

   Contains:
   - `AssemblyEnumerator` struct that resolves the 8 IL2CPP symbols listed above (domain_get, domain_get_assemblies, assembly_get_image, image_get_name, image_get_class_count, image_get_class, class_get_name, class_get_namespace).
   - Reuse existing `TypeIntrospection` from diagnostics (or duplicate the 4 symbols since diagnostics keeps it private). **Decision**: Extract `TypeIntrospection` to a shared location OR just re-resolve the same 4 symbols in the new struct. Since diagnostics.rs owns it privately and AGENTS.md says decompose big files → re-resolve in the new struct to avoid coupling. Bundle all 12 symbols into one `DumpContext` struct.
   - `pub fn dump_all_classes()` — the main entry point:
     1. Resolve all symbols into `DumpContext`. Bail with `hlog_error!` if any fail.
     2. Call `il2cpp_domain_get()` → domain.
     3. Call `il2cpp_domain_get_assemblies(domain, &mut count)` → assembly array.
     4. Determine output path: use `std::env::current_exe()` parent dir (game dir) + `il2cpp_classes.txt`. Fallback to current dir.
     5. Open file with `std::fs::File::create`.
     6. For each assembly (0..count):
        - `il2cpp_assembly_get_image` → image
        - `il2cpp_image_get_name` → image name string
        - `il2cpp_image_get_class_count` → class_count
        - Write section header: `=== Assembly: {image_name} ({class_count} classes) ===`
        - For each class (0..class_count):
          - `il2cpp_image_get_class(image, idx)` → klass
          - `il2cpp_class_get_namespace(klass)` → namespace
          - `il2cpp_class_get_name(klass)` → name
          - Write: `[{namespace}] {name}`
          - Iterate fields via `il2cpp_class_get_fields` (same pattern as `dump_all_fields`), write each as `  field: {type} {name}`
          - Iterate methods via `class_get_methods` (SDK method), write each as `  method: {return_type} {name}({param_count} args)`
     7. Log summary to hachimi.log: `"Dumped {total_classes} classes from {assembly_count} assemblies to {path}"`.
   - Use `std::io::BufWriter` for performance (thousands of classes × dozens of fields each).

2. **Register module in `plugins/training-tracker/src/lib.rs`** — add `mod class_dump;`.

3. **Add menu button in `plugins/training-tracker/src/ui.rs`** — in `draw_menu_section_inner`, after the existing "Show Training Overlay" button, add:
   ```rust
   if sdk.gui_button(ui, "📋 Dump All IL2CPP Classes") {
       class_dump::dump_all_classes();
       sdk.show_notification("Class dump written — see il2cpp_classes.txt");
   }
   ```
   
   Add a status indicator: use an `AtomicBool` in `class_dump` to track if a dump is in progress (it could take a few seconds), show "Dumping..." label while active. Or keep it simple and synchronous since it runs on the UI thread and the game will just freeze briefly.

4. **Run clippy + fmt + tests** to validate.

## File output format

```
=== Assembly: umamusume.dll (4523 classes) ===

[Gallop] SingleModeMainViewController
  field: System.Int32 _commandId
  field: Gallop.TrainingController <TrainingController>k__BackingField
  method: System.Void .ctor(0 args)
  method: Gallop.TrainingController get_TrainingController(0 args)
  method: System.Void OnClickTrainingMenu(1 args)
  ...

[Gallop] WorkDataManager
  field: ...
  ...

=== Assembly: mscorlib.dll (1200 classes) ===
...
```

## Risks / Open Questions

- **Performance**: The game has ~10k+ classes. Full introspection with field types (calling `il2cpp_type_get_name` per field) may take 1-5 seconds. BufWriter + synchronous execution on UI thread should be fine — the game freezes briefly but it's a dev tool triggered by explicit button press.
- **File path**: `std::env::current_exe().parent()` should give the game dir where `hachimi.log` lives. If that fails, fall back to writing next to the DLL or current dir.
- **Thread safety**: All IL2CPP introspection APIs are safe to call from any thread after init. The menu callback runs on the render thread which already calls IL2CPP.
- **`il2cpp_type_get_name` returns allocated string** — must call `il2cpp_free` after reading. Already handled in existing `TypeIntrospection`.
- **Null checks everywhere** — IL2CPP can return null for any pointer. Every step must guard.
