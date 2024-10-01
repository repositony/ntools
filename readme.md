# Neutronics toolbox

[![Build Status][test-img]][test-url]
[![Documentation][doc-img]][doc-url]

[test-img]: https://github.com/repositony/ntools/actions/workflows/tests.yml/badge.svg
[test-url]: https://github.com/repositony/ntools/actions/workflows/tests.yml

[doc-img]: https://github.com/repositony/ntools/actions/workflows/documentation.yml/badge.svg
[doc-url]: https://repositony.github.io/ntools/index.html

**A modular toolkit of fast and reliable libraries for neutronics analysis**

The full library documentation is published [here](https://repositony.github.io/ntools/index.html)

*This is a pre-release version for testing and development, with the API subject to change until a stable 1.0 release.*

## Library overview

The `ntools` toolkit contains a collection of mostly modular libraries for
common fusion neutronics tasks and analysis.

| Crate | Description |
| ----- | ----------- |
| [fispact](https://repositony.github.io/ntools/ntools/fispact/index.html) | Analysis tools for FISPACT-II inventory calculations  |
| [iaea](https://repositony.github.io/ntools/ntools/iaea/index.html)       | Module for interacting with the IAEA decay data API   |
| [mesh](https://repositony.github.io/ntools/ntools/mesh/index.html)       | MCNP mesh tally operations and file parsing           |
| [posvol](https://repositony.github.io/ntools/ntools/posvol/index.html)   | Se/deserialiser for UKAEA CuV posvol binaries         |
| [utils](https://repositony.github.io/ntools/ntools/utils/index.html)     | Common utilities and extension traits                 |
| [weights](https://repositony.github.io/ntools/ntools/weights/index.html) | Tools for MCNP weight window operations               |
| [wwgen](https://repositony.github.io/ntools/ntools/wwgen/index.html)     | Weight window generation methods for MCNP             |

[Command line tools](https://github.com/repositony?tab=repositories&q=&type=&language=rust&sort=) 
built on these core libraries are maintained in their own repositories.

### Modular crates

The `ntools` crates are included as dependencies through feature flags. Specify
`"full"` to include everything.

```toml
[dependencies]
ntools = { git = "https://github.com/repositony/ntools.git", features = ["full"] }
```

It is recommended that users are more selective to avoid compiling unnecessary
dependencies.

For example, if only the `fispact` and `iaea` crates are needed:

```toml
[dependencies]
ntools = { git = "https://github.com/repositony/ntools.git", features = ["fispact", "iaea"] }
```

### Documentation and Tests

To reproduce the full library documentation seen
[here](https://repositony.github.io/ntools/index.html):

```shell
cargo doc --workspace --no-deps --features full
```

To run all tests for all modules, use the `--workspace` flag.

```shell
cargo test --workspace
```
