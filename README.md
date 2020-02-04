# Coster

Coster will be a web application designed to be used for the purpose of sharing costs between multiple people.

This project is inspired by [SplittyPie](https://github.com/cowbell/splittypie), but with the following differences:

+ Currency per expense - groups can submit expenses with different currencies
+ Written in Rust - simpler distribution, and for my own learning purposes.
+ Support for a local database using [sled](https://github.com/spacejam/sled)
+ For simplicity, initially not a Progressive Web App, but the option remains open to compile the library to Web Assembly for clientside calculations/interactions, hopefully using a rust front-end framework like [yew](https://github.com/yewstack/yew) or similar.

## Libraries

### costing

The business logic for this application.

### accounting

A double entry accounting system.

### currency

Primatives for representing monetary values with associated currencies, and methods for converting them.

### exchange_rate

Primatives for representing exchange rates between currencies, methods for querying online exchange rate apis.