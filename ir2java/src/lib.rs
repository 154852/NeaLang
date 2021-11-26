mod ins;
mod unit;

pub use unit::*;
pub use ins::*;


/*
Paths:
- Local / globals are simple, just push as expected
- Accessing slice elements is less simple:
    - Currently slices are set up with: push I1, push I2, push ref - [I1, I2, ref]
    - Java's aaload wants to, pop I, pop ref - [ref, I]
    - java does however have swap, but this would be a problem as it can only be used on category 1 values, but an array index has to be an int anyway,
    which is category 1, so therefore:
    - push I1, push I2, push ref - [I1, I2, ref]
    - swap - [I1, ref, I2]
    - aaload - [I1, ref]
    - swap - [ref, I1]
    - aaload - [ref]


*/