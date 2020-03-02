# Coster

Coster will be a web application designed to be used for the purpose of sharing costs between multiple people.

This project is inspired by [SplittyPie](https://github.com/cowbell/splittypie), but with the following differences:

+ Currency per expense - groups can submit expenses with different currencies
+ Written in Rust - simpler distribution, and for my own learning purposes.
+ Support for a local database using [sled](https://github.com/spacejam/sled)
+ For simplicity, initially not a Progressive Web App, but the option remains open to compile the library to Web Assembly for client-side calculations/interactions, hopefully using a rust front-end framework like [yew](https://github.com/yewstack/yew) or similar.

## Libraries

The following libraries were developed to service this application's needs, but they should also hopefully be useful for other purposes:

+ [Doublecount](https://github.com/kellpossible/doublecount) - A double entry accounting system/library.
+ [Commodity](https://github.com/kellpossible/commodity) - A library for representing commodities/currencies.
