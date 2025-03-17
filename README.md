# svg2pbc-rs

A rust reimplementation of the `svg2pbc` tool from the [cards-example](https://github.com/pebble-examples/cards-example/blob/master/tools/svg2pdc.py) project.

Tested with the files in the `cards-example` project. (Currently the tests fail for the example [files found in the official Pebble SDK](https://developer.rebble.io/developer.pebble.com/guides/app-resources/converting-svg-to-pdc/index.html#example-output))

## Installation

```bash
cargo install --git github.com/flyinpancake/svg2pbc-rs
```

## Usage

```bash
svg2pbc-rs <input> -o <output>
```

> [!NOTE]
>
> Check the help message for more options.

## Features

- [x] Parse SVG files
- [x] Output PDC files
- [x] Support the original `svg2pdc` element types
- [ ] Support batch processing
- [ ] Support frames and animations
- [ ] Parity with the original `svg2pdc` tool
- [ ] Binary release
- [ ] Library usage and documentation
- [ ] CI/CD
- [x] Tests
- [ ] Benchmarks
- [ ] Examples
- [ ] Documentation
