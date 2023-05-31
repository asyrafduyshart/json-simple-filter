# Rust JSON Simple Filter Library

This library provides functionality to apply a series of filters to JSON data structures in Rust. The filters are defined by a string and parsed into a series of `Filter` structs, which can then be applied to any JSON data structure.

## Features

- Define filters using simple string syntax.
- Apply multiple filters to JSON data.
- Filters can use string comparison, numeric comparison, or both.
- Optionally use multipliers for numeric comparisons.

## Usage

### Defining Filters

Filters are defined as a string with a simple syntax:

```
.field = 'value' AND .other_field >= 10
```

Each part of the string is separated by " AND " to define multiple filters.

In each filter:

- The field to be filtered is prefixed with a dot (`.`).
- The operator can be one of: `=`, `!=`, `>`, `<`, `>=`, `<=`.
- The value to be compared can be a string (surrounded by `'`) or a number.

### Parsing Filters

Use the `parse` function to parse a filter string into a list of `Filter` structs:

```rust
let filters = parse(filter_string).unwrap();
```

### Applying Filters

Use the `apply` function to apply a list of `Filter` structs to a JSON data structure:

```rust
let v = json!({ "field": "hello", "value": 20 });
let result = apply(&v, &filters);
```

This returns `true` if the data passes all filters, and `false` otherwise.

## Example

```rust
use serde_json::json;
use json_filter::{parse, apply};

let filter_string = ".field = 'hello' AND .value >= 20";
let filters = parse(filter_string).unwrap();

let v = json!({ "field": "hello", "value": 30 });
assert!(apply(&v, &filters));

let v = json!({ "field": "world", "value": 30 });
assert!(!apply(&v, &filters));
```

## Testing

The library includes a test suite to validate the functionality. Run the tests with `cargo test`.
