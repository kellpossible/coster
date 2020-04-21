# Coster

Coster will be a web application designed to be used for the purpose of sharing costs between multiple people.

This project is inspired by [SplittyPie](https://github.com/cowbell/splittypie), but with the following differences:

+ Written in Rust - simpler distribution, more reliable than Javascript with a stronger type system, and also for my own learning purposes.
+ Uses [yew](https://github.com/yewstack/yew) (similar to React and Elm) for the front-end logic.
+ Currency per expense - groups can submit expenses with different currencies
+ Support for a local database using [rusqlite](https://crates.io/crates/rusqlite). Or perhaps use [kvdb-rocksdb](https://crates.io/crates/kvdb-rocksdb) on the server side, along with [kvdb-web](https://crates.io/crates/kvdb-web) on the client side because they both support the same interface which can be used directly in the `costing` library, and the [kvdb-memorydb](https://crates.io/crates/kvdb-memorydb) can be used for testing. Using `kvdb` would allow allow easily leveraging the work done to implement `serde` support for most of the types which was needed anyway for synchronisation.
+ Explore using [web-view](https://github.com/Boscop/web-view) in the future to provide a desktop application.

## Libraries

The following libraries were developed to service this application's needs, but they should also hopefully be useful for other purposes:

+ [Doublecount](https://github.com/kellpossible/doublecount) - A double entry accounting system/library.
+ [Commodity](https://github.com/kellpossible/commodity) - A library for representing commodities/currencies.
+ [cargo-i18n](https://github.com/kellpossible/cargo-i18n) - A tool for extracting localizations and embedding them using `i18n-embed`.
+ [i18n-embed](https://github.com/kellpossible/cargo-i18n/tree/master/i18n-embed) - A library for embedding localizations extracted using `cargo-i18n`.

## TODO

+ [x] Implement `gettext` translation capabilities using [cargo-i18n](https://github.com/kellpossible/cargo-i18n).
+ [x] Build `gui` WASM subcrate automatically using the [build.rs](./build.rs) build script.
+ [ ] Create a JSON rest API
+ [ ] Create GUI with yew
+ [ ] Support cookies to remember user on client
+ [ ] Implement database migrations with [migrant](https://crates.io/crates/migrant) or [refinery](https://github.com/rust-db/refinery).
