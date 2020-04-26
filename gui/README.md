# WASM GUI

This crate contains the code for the web GUI which compiles to WASM. See [build.rs](../build.rs) for how this crate is built and included in the overall `coster` website.

For debugging/development purposes, you can build the library using this command:

```bash
wasm-pack build --target web --out-dir ../public/js/gui
```

To build the css files (not required) run:

```bash
npm run css-build
```
