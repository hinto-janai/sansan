# `sansan` internals
This is an overview of `sansan`'s internals.

The code itself has `grep`-able comments with keywords:

| Word        | Meaning |
|-------------|---------|
| `INVARIANT` | This code makes an _assumption_ that must be upheld for correctness
| `SAFETY`    | This `unsafe` code is okay, for `x,y,z` reasons
| `FIXME`     | This code works but isn't ideal
| `HACK`      | This code is a brittle workaround
| `PERF`      | This code is weird for performance reasons
| `TODO`      | This has to be implemented
| `SOMEDAY`   | This should be implemented... someday

---

* [Code Structure](#Code-Structure)
* [Overview](#Overview)

---

# Code Structure
The structure of the folders & files located in `src/`.

## Data
These folders represent data:

| Folder         | Purpose |
|----------------|---------|

## Threads
These folders represent OS threads with a distinct purpose:

| Folder           | Purpose |
|------------------|---------|

## Misc
These are top-level `src/` files for miscellaneous stuff:

| File           | Purpose |
|----------------|---------|

# Overview
