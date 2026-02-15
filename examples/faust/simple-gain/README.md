## Prerequisites

1. Clone Faust to your chosen location:
   ```bash
   git clone --recursive https://codeberg.org/crop/faust.git && cd faust && git checkout b935e324c5174d73d010815a9f426047618ed457 && make
   ```
2. At this project root, set Rust to use nightly:
   ```bash
   rustup override set nightly
   ```
3. Create the DSP source file:
   ```bash
   touch ./src/dsp.rs
   ```
4. Set the `FAUST_PATH` environment variable.

Once you run `cargo run --release`, the files `./src/dsp.rs` and `./src/faust.dsp.json` will be automatically generated. These are auto-generated files produced by `faust-build` and `faust-ui-build`, based on `./src/faust.dsp`.

## Troubleshooting

- **`error: no matching package named 'faust-traits' found`**
  - Ensure you completed step 2 above (setting Rust to nightly).

- **`thread 'main' panicked at ... No such file or directory (os error 2): .../src/dsp.rs`**
  - Ensure you completed step 3 above (creating the `dsp.rs` file).

- **`FAUST_PATH must be set: NotPresent`**
  - Set the `FAUST_PATH` environment variable before building. This should point to your Faust installation directory.