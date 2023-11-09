# Book
Source to build the documentation for [`sansan`](https://sansan.cat).

## Build
You need [`mdbook`](https://github.com/rust-lang/mdBook).

Build:
```bash
mdbook build
```

or if you have [`cargo`](https://doc.rust-lang.org/cargo/getting-started/installation.html):
```bash
cargo install mdbook
cargo mdbook build
```

The output files should be in `docs/` after building, e.g:
```
firefox docs/index.html
```
should open the book.
