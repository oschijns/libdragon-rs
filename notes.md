# Migration notes

This file keeps track of the changes that were applied to the
project to update the toolchain and the version of libdragon used.


## Initial state

State of the examples using the old version of `libdragon-rs` without any changes.

| Example     | Compile | Run     | Remark |
|-------------|---------|---------|--------|
| a440        | Ok      | Ok      |        |
| audioplayer | Ok      | Ok      | Unhandled out of bound when selecting music -1. |
| customfont  | Ok      | Ok      |        |
| dfstest     | Ok      | Ok      | Out of bound error when pressing `L` then `A`. I suspect the demo tries to open a file when none is selected. |
| fontdemo    | Ok      | Ok      |        |
| gldemo      | Ok      | Ok      | unused imports: `Surface`, `TexFormat` |
| importglb   | Ok      | Ok      |        |
| joypadtest  | Ok      | Ok      |        |
| mixertest   | Ok      | Ok      |        |
| rdpqtest    | Ok      | Ok      |        |
| rspqdemo    | Ok      | Ok      |        |
| spritemap   | Ok      | Ok      |        |
| test        | Ok      | Ok      |        |
| vtest       | Ok      | Ok      |        |


## Code reformatting

_rustfmt_ is applied to all sources in the project to standardized the formatting before any other changes are applied to the project.
Then we run _clippy_ to identify and rewrite code portions to follow _Rust_'s conventions.
If changes requested by _clippy_ cannot be trivially fixed, we add an `allow` attribute.

After applying _clippy_'s changes, we can rerun all the examples to guarantee no behavior has been broken.

| Example     | Compile | Run     | Remark |
|-------------|---------|---------|--------|
| a440        | Ok      | Ok      |        |
| audioplayer | Ok      | Ok      |        |
| customfont  | Ok      | Ok      |        |
| dfstest     | Ok      | Ok      |        |
| fontdemo    | Ok      | Ok      |        |
| gldemo      | Ok      | Ok      |        |
| importglb   | Ok      | Ok      |        |
| joypadtest  | Ok      | Ok      |        |
| mixertest   | Ok      | Ok      |        |
| rdpqtest    | Ok      | Ok      |        |
| rspqdemo    | Ok      | Ok      |        |
| spritemap   | Ok      | Ok      |        |
| test        | Ok      | Ok      |        |
| vtest       | Ok      | Ok      |        |

### Notes:
One of _clippy_ suggestion caused an error on music playback.

```
// replacing
if let Some(_) = core::mem::replace(&mut self.backing_instance, None) { ... }

// by
if self.backing_instance.take().is_some() { ... }

// caused
// on file Arcade_S900.xm64
// error: AI reading from RDRAM address 0xa9490 which is modified in the cache (missing cache writeback?)
```


## Upgrading the Rust's toolchain

Bump the _Rust_'s compiler nightly channel and we fix `rust-toolchain.toml`
and `mips-nintendo64-none.json` accordingly.

We rerun the unit tests and the examples.

| Example     | Compile | Run     | Remark |
|-------------|---------|---------|--------|
| a440        | Ok      | Ok      |        |
| audioplayer | Ok      | Ok      |        |
| customfont  | Ok      | Ok      |        |
| dfstest     | Ok      | Ok      |        |
| fontdemo    | Ok      | Ok      |        |
| gldemo      | Ok      | Ok      |        |
| importglb   | Ok      | Ok      |        |
| joypadtest  | Ok      | Ok      |        |
| mixertest   | Ok      | Ok      |        |
| rdpqtest    | Ok      | Ok      |        |
| rspqdemo    | Ok      | Ok      |        |
| spritemap   | Ok      | Ok      |        |
| test        | Ok      | Ok      |        |
| vtest       | Ok      | Ok      |        |

### Notes:
`mips-nintendo64-none.json` is now renamed `mips64-nintendo64-none.json`
to reflect the change for `mips` architecture to `mips64` architecture.


## Upgrade the dependencies

We run `cargo update` to generate a new `Cargo.lock` file.

We rerun the unit tests and the examples.

| Example     | Compile | Run     | Remark |
|-------------|---------|---------|--------|
| a440        | Ok      | Ok      |        |
| audioplayer | Ok      | Ok      |        |
| customfont  | Ok      | Ok      |        |
| dfstest     | Ok      | Ok      |        |
| fontdemo    | Ok      | Ok      |        |
| gldemo      | Ok      | Ok      |        |
| importglb   | Ok      | Ok      |        |
| joypadtest  | Ok      | Ok      |        |
| mixertest   | Ok      | Ok      |        |
| rdpqtest    | Ok      | Ok      |        |
| rspqdemo    | Ok      | Ok      |        |
| spritemap   | Ok      | Ok      |        |
| test        | Ok      | Ok      |        |
| vtest       | Ok      | Ok      |        |

Then we check the `Cargo.toml` file of each project to pick a newer version of each dependency if available.

We rerun the unit tests and the examples.

| Example     | Compile | Run     | Remark |
|-------------|---------|---------|--------|
| a440        |         |         |        |
| audioplayer |         |         |        |
| customfont  |         |         |        |
| dfstest     |         |         |        |
| fontdemo    |         |         |        |
| gldemo      |         |         |        |
| importglb   |         |         |        |
| joypadtest  |         |         |        |
| mixertest   |         |         |        |
| rdpqtest    |         |         |        |
| rspqdemo    |         |         |        |
| spritemap   |         |         |        |
| test        |         |         |        |
| vtest       |         |         |        |


## Upgrading Libdragon

We modify the `build.rs` file to pick the latest version of _Libdragon_.
We fix any interface issue we encounter.
