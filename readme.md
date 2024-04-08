# Neutronics toolbox

[![Build Status][test-img]][test-url]
[![Documentation][doc-img]][doc-url]

[test-img]: https://github.com/repositony/ntools/actions/workflows/tests.yml/badge.svg
[test-url]: https://github.com/repositony/ntools/actions/workflows/tests.yml

[doc-img]: https://github.com/repositony/ntools/actions/workflows/documentation.yml/badge.svg
[doc-url]: https://repositony.github.io/ntools/index.html

**A modular toolkit of fast and reliable libraries for neutronics analysis**

The full library documentation published [here](https://repositony.github.io/ntools/index.html)

*This is a pre-release version for testing and development, with the API subject to change until a stable 1.0 release. The core structure and libraries are written but it is a matter of finding time to extend all the high-level features and functionality*

## Command line tools

Several simple and efficient command line tools are written using this core
library.

| Command line   | Description                                             |
| -------------- | ------------------------------------------------------- |
| `mesh2vtk`     | Convert any meshtal tally to various VTK formats        |
| `mesh2ww`      | Convert any meshtal tally to a mesh-based weight window |
| `pointextract` | Extract voxel results for any point(s) in a mesh        |
| [splitmesh](https://github.com/repositony/splitmesh) | Split meshtal tallies into individual files             |
| [posvol](https://github.com/repositony/posvol)       | Inspect and convert binary UKAEA CuV posvol files       |

All tools are fully documented with detailed `--help` messages, including
examples for common use cases.

## Library overview

The `ntools` toolkit contains a collection of mostly modular libraries for
common fusion neutronics tasks and analysis.

| Crate | Description |
| ----- | ----------- |
| [fispact](https://repositony.github.io/ntools/ntools/fispact/index.html) | Analysis tools for FISPACT-II inventory calculations  |
| [format](https://repositony.github.io/ntools/ntools/format/index.html)   | Common utility for extended `std` type formatting     |
| [iaea](https://repositony.github.io/ntools/ntools/iaea/index.html)       | Module for interacting with the IAEA decay data API   |
| [mesh](https://repositony.github.io/ntools/ntools/mesh/index.html)       | MCNP mesh tally operations and file parsing           |
| [posvol](https://repositony.github.io/ntools/ntools/posvol/index.html)   | Se/deserialiser for UKAEA CuV posvol binaries         |
| [weights](https://repositony.github.io/ntools/ntools/weights/index.html) | Tools for MCNP weight window operations               |
| [wwgen](https://repositony.github.io/ntools/ntools/wwgen/index.html)     | Weight window generation methods for MCNP             |

The decision was made to split the command line tools and core libraries into
separate repositories for better maintainability, scalability, and shorter
compile times.

### Features

`ntools` is a collection of utility crates that can be used individually or in
combination to easily build more advanced anaysis tools.

The structure is heavily inspired by the [gloo](https://github.com/rustwasm/gloo)
approach. These modules are often used in combination, so I find it an excellent
compromise between the conveniece of a large single library and the modularity
of many individual repositories.

#### Crate selection

The `ntools` crates are included as dependencies through feature flags. Specify
`"full"` to include everything.

```toml
[dependencies]
ntools = {
    git      = "https://github.com/repositony/ntools.git",
    features = ["full"]
}
```

However, it is strongly recommended that users are selective to avoid compiling
unnecessary dependencies.

For example, perhaps there is a need to generate an SDEF source of gamma
emissions from a FISPACT-II JSON.

```toml
[dependencies]
ntools = {
    git      = "https://github.com/repositony/ntools.git",
    features = ["fispact", "iaea"]
}
```

This will compile only the `fispact` and `iaea` crates. The first has various
tools for interpreting and manipulating FISPACT-II output data, while the latter
can use the IAEA chart of nuclides decay data to define a source.

### Documentation and Tests

To produce the full library documentation seen
[here](https://repositony.github.io/ntools/index.html), specify the `"full"`
feature flag.

```shell
cargo doc --no-deps --features full
```

To run all tests for all modules, use the `--workspace` flag.

```shell
cargo test --workspace
```
