The `fispact` module contains useful utilities for quickly processing FISPACT-II
outputs.

Currently the JSON output is fully deserialised to useful structures. The TAB
and legacy output files will be supported as needed.

## Quickstart example

To quickly load a JSON inventory into structures for further analysis, this is
a simple read.

```rust, ignore
// Read the JSON contents of the file as an instance of `Inventory`.
let inventory: Inventory = read_json("path/to/results.json").unwrap();
```

## Core concepts

The library is structured much like the output files for simplicity.

An `Inventory` contains metadata about the run in `RunData`, and a list of
`Interval`s for every step of the inventory calculation. An `Interval` describes
overall totals for the sample, a list of every `Nuclide` present, and a basic
histogram of gamma lines for FISPACT-II v5.

### Important naming changes

Several tweaks were made to the deserialiser and data structures are not
entirely a 1:1 match for the JSON dictionary.

#### Sample mass

The masses are inconsistent between nuclides and the interval, often leading to
people forgetting to change the units to/from grams/kilograms.

This is converted to grams in the background such that `Nuclide.mass` and
`Interval.mass` are consistent.

#### Sample dose

The sample dose rate for the interval is converted into something more useful
that is concise and works better with the type system.

```json
"dose_rate": {
    "type": "Point source",
    "distance": 1.0,
    "mass": 1.0,
    "dose": 1.0
}
```

This is tuned into a `Dose` of type `DoseKind`. The mass is redundant as either
the mass of the sample or irrelevant for a contact dose, so it is dropped.

```rust, no_run
struct Dose {
    /// Dose rate (Sv/hr)
    rate: f64,
    /// Type of dose
    kind: DoseKind,
}

enum DoseKind {
    /// Semi-infinite slab approximation
    Contact,
    /// Point source approximation at contained distance (m)
    Point(f64),
}
```

#### Sample totals

Several of the main sample totals have been renamed for brevity. For example,
at the `Interval` level:

```json
"inventory_data": [
    {
        "total_atoms": 1.0,
        "total_activity": 1.0,
        "total_mass": 1.0,
        "total_heat": 1.0,
    }
]
```

```rust, no_run
struct Interval {
    atoms: f64,
    activity: f64,
    mass: f64,
    heat: f64,
}
```
