# Code Style

### Line Length Limit
Whatever seems reasonable, but probably not more than the width of a reasonably sized monitor at a reasonably sized font

### Unnecessary callbacks
Avoid callbacks where possible for handling `Result`s and `Option`s, instead use a `match` in order to reduce overhead at lower levels of optimisation. For an example, see [this](https://godbolt.org/z/Eb4PK44rT).

### Match Hell
With nested matches and if statements, each match / if statement should start on it's own indented line. This is top help somewhat with the readability of deeply nested matches, which are common in Rust.

### Single-line if statements
If statements can reduced be on a single, but not if they have an else or else if part.

### Getters for vectors
While it can make sense to return a pure reference to a vector, a shortcut the vector length should always be supplied in addition, in the form of `<name>_count` 

### Unwrap / Expect
Should only be used when it is provable that that it won't fail.

### Abreviation
Abreviations should be obvious in context, e.g. vt is usually an acceptable abreviation for value_type.
Note: ins is always singular, insns is always plural.