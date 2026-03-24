# rufdb vs lowdb: Behavioral Differences

rufdb is a faithful Rust port of [lowdb](https://github.com/typicode/lowdb). This
document records the three intentional behavioral divergences where rufdb
improves on lowdb's JavaScript quirks.

## 1. `read()` uses `Option`, not JavaScript truthiness

**lowdb:**
```typescript
async read(): Promise<void> {
    const data = await this.adapter.read()
    if (data) this.data = data   // ← truthiness check
}
```
If the adapter returns a falsy value (`0`, `""`, `false`), the data is
silently discarded and `this.data` retains its previous value. This is a
JavaScript truthiness quirk — only `null` should be discarded (meaning
"no data stored yet"), but `0` and `""` are also falsy.

**rufdb:**
```rust
pub async fn read(&mut self) -> Result<(), RufDbError> {
    let data = self.adapter.read().await?;
    if let Some(d) = data {
        self.data = d;       // ← only None is discarded
    }
    Ok(())
}
```
rufdb uses `Option<T>` — only `None` is discarded. All valid values
(including `0`, empty strings, and `false`) are preserved correctly.

**Impact:** If your lowdb database stored a top-level falsy value (rare
in practice since most databases are objects/arrays), lowdb would silently
fail to load it. rufdb handles all types correctly.

---

## 2. `write()` always writes

**lowdb:**
```typescript
async write(): Promise<void> {
    if (this.data) this.adapter.write(this.data)  // ← truthiness check
}
```
If `this.data` is falsy (`0`, `""`, `false`), the write is silently
skipped. This is almost never intentional.

**rufdb:**
```rust
pub async fn write(&self) -> Result<(), RufDbError> {
    self.adapter.write(&self.data).await  // ← always writes
}
```
rufdb unconditionally writes `self.data` to the adapter.

**Impact:** If you set `db.data = 0` (or any falsy value) in lowdb and
call `write()`, nothing happens. In rufdb, the value is persisted as
expected.

---

## 3. `MemorySync.read()` is consistent with `Memory.read()`

**lowdb:**
```typescript
// Memory (async) — correct
read(): Promise<T | null> {
    return Promise.resolve(this.#data)   // ← returns #data directly
}

// MemorySync — inconsistent
read(): T | null {
    return this.#data || null             // ← || converts falsy to null
}
```
lowdb's `MemorySync.read()` uses `||` which converts any falsy stored
value to `null`. The async `Memory.read()` does not have this bug.

**rufdb:**
```rust
// Both Memory and MemorySync use the same correct logic:
fn read(&self) -> Result<Option<T>, RufDbError> {
    let guard = self.data.read().map_err(|e| RufDbError::Lock(e.to_string()))?;
    Ok(guard.clone())   // ← returns the stored value directly
}
```
Both async and sync memory adapters behave identically — they return the
stored value without any truthiness coercion.

---

## Summary

| Behavior | lowdb | rufdb | Who's correct? |
|---|---|---|---|
| `read()` with falsy data | Discards it | Preserves it | rufdb |
| `write()` with falsy data | Silently skips | Always writes | rufdb |
| `MemorySync.read()` with falsy data | Returns `null` | Returns stored value | rufdb |

These differences only affect edge cases where the top-level `data` value
is a JavaScript "falsy" value. For the common case where `data` is an
object or array, lowdb and rufdb behave identically.
