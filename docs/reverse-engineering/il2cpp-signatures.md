# IL2CPP method-signature reference

Before hooking (detouring) any IL2CPP method, **verify its real signature** — the
return type and argument count — against a dump of the game's metadata. Getting the
return type wrong is not a soft failure: it crashes the game (see the pitfall
below).

## The reference file

[`il2cpp/umamusume-methods.txt`](il2cpp/umamusume-methods.txt) is a trimmed dump of
the `umamusume.dll` assembly — **method signatures only** (fields and
compiler-generated `<...>` types removed), which is the assembly that holds
essentially all of our game-logic hooks. It is ~4 MB and greppable.

Format (one class block per type):

```
[Gallop] SingleModeMainViewController
  method: System.Void OnClickTrainingMenu(1 args)
  method: System.Collections.IEnumerator SendCommandAsync(6 args)
  ...
```

### Looking up a method

```bash
# all methods on a class
awk '/^\[[^]]*\] SingleModeMainViewController$/{f=1;next} f&&/^\[/{exit} f' \
  docs/reverse-engineering/il2cpp/umamusume-methods.txt

# a specific method's return type + arg count (may span classes)
grep -n ' SendCommandAsync(' docs/reverse-engineering/il2cpp/umamusume-methods.txt
```

For non-`umamusume.dll` assemblies (UnityEngine.\*, DOTween, mscorlib, Cute.\*),
consult a full dump — they are not included here to keep the file small. The hooks
we have on those are few and target stable, well-known APIs.

## The pitfall: return type **must** match (esp. coroutines)

A hook's `extern "C" fn` must declare the **same return type** as the real method.
The compiled `methodPointer` puts the return value in `RAX`; if your hook returns
`()` (void) but the method returns a value, the caller reads garbage from `RAX`.

This bit us with `SingleModeMainViewController.SendCommandAsync`:

```
method: System.Collections.IEnumerator SendCommandAsync(6 args)
```

It is a **coroutine kickoff** — it returns an `IEnumerator` that the game passes to
`StartCoroutine`. The first hook declared no return type, so the game started a
coroutine on a garbage pointer and crashed deep in `GameAssembly` (no Hachimi frame
on the stack — the trampoline had already returned into the original). Declaring the
hook `-> *mut Il2CppObject` and forwarding the original's return value fixed it.

Rules of thumb when the dump shows these return types:

| Return type in dump | Hook must return | Notes |
|---|---|---|
| `System.Void` | `()` (omit `-> ...`) | the common case |
| `System.Collections.IEnumerator` | `*mut Il2CppObject` (or the `IEnumerator` alias) | **coroutine** — forward the return; optionally hook the enumerator's `MoveNext` (see `GameSystem::InitializeGame`) |
| `System.Threading.Tasks.Task[...]` | the task pointer (`*mut Il2CppObject`) | async — forward the return |
| value/struct/object | the matching type | forward the original's return unless intentionally overriding |

Reference implementations of the coroutine pattern already in the tree:
`GameSystem::InitializeGame`, `CharacterNoteTopViewController::InitializeView`, and
`SingleModeMainViewController::SendCommandAsync`.

### Other ABI notes

- **Arg count**: resolve with the exact count from the dump (`get_method_addr(class,
  c"Name", N)`); a wrong `N` resolves to null (hook silently skipped) or the wrong
  overload.
- **`MethodInfo*`**: IL2CPP appends a trailing `const MethodInfo*` to every
  `methodPointer`. Our hooks generally omit it and forwarding still works because
  most methods don't dereference it; it was not the cause of the `SendCommandAsync`
  crash (the return type was). When invoking a method via its `methodPointer`
  yourself (e.g. the career watcher's getters), pass the `MethodInfo*` as the last
  arg — `fn(this, ..., method_info)`.

## Regenerating / updating

The trimmed file goes stale when the game updates. To regenerate from a fresh full
dump (e.g. `il2cpp_classes.txt` produced in the game directory):

```bash
F=/path/to/il2cpp_classes.txt
OUT=docs/reverse-engineering/il2cpp/umamusume-methods.txt
{
  printf '# IL2CPP method-signature reference — umamusume.dll\n'
  printf '# methods only; fields and <...> generated types dropped; umamusume.dll only.\n'
  printf '# Source build: <describe>; dumped <date>.\n#\n'
  awk '
    /^=== Assembly: umamusume.dll/{inuma=1;next}
    /^=== Assembly: /{inuma=0}
    inuma && /^\[/{ gen=($0 ~ /[<>]/); hdr=$0; ph=0 }
    inuma && !gen && /method:/{ if(!ph){print ""; print hdr; ph=1}; print }
  ' "$F"
} > "$OUT"
```

Keep the **full** dump outside the repo (it is ~22 MB); only the trimmed method list
is committed.
