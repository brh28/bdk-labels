# Design Questions

## 1. SQLite vs JSONL as Source of Truth

### Current state

SQLite is the source of truth. BIP329 JSONL is a transport format — data comes in via `import` and goes out via `export`. The DB is the working state; JSONL files are not kept in sync after import.

### The problem

SQLite has no built-in undo. A mistaken `import --override` or `rm --force` has no recovery path. The user cannot diff changes, revert to a previous state, or see what changed when.

### Options considered

**Export + git as a manual workflow**
After each mutating command, the user exports to JSONL and commits to git. Possible, but error-prone — small changes like a single `describe` update are easy to skip.

**Automatic export + optional git**
After every mutation, automatically write a per-wallet `.jsonl` snapshot to a configurable directory. If that directory is a git repo, run `git add` + `git commit` automatically. Git is optional — users who don't configure it still get snapshot files as a manual fallback. Per-wallet files make diffs readable.

**History table**
Log every mutation (old value, new value, timestamp, operation) in a `history` table. Enables an `undo` command and a `log` command. Self-contained, no git dependency. Non-trivial to implement correctly across all write paths.

**Decision deferred.** The current SQLite implementation works. Revisit when `export` is implemented and the automatic snapshot workflow is designed — at that point the overlap between the DB and the JSONL files will be clearer.


## 2. Wallet as required db field?

Makes sense as it belongs to any of the user's personal wallet types (tx, addr, output, etc), but not public types