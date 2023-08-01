# Changelog

## Unreleased

### Various fixes and improvements

- feat: Implement sourcemap composition(#67) by @loewenheim

## 6.3.0

### Various fixes & improvements

- feat: Sourcemaps now support debug ids (#66) by @loewenheim

## 6.2.3

### Various fixes & improvements

- fix: Correctly handle protocol-only sourceRoot values (#61) by @kamilogorek

## 6.2.2

### Various fixes & improvements

- Ensure greatest_lower_bound returns lowest match index (#60) by @jridgewell
- feat: Switch to data-encoding for base64 (#59) by @mitsuhiko

## 6.2.1

### Various fixes & improvements

- ref: Update CI definitions (#58) by @Swatinem
- fix: Correctly rewrite SourceMapHermes (#56) by @Swatinem
- Remove regex dependency for faster runtime, and compile (#55) by @willstott101
- Jridgewell index (#54) by @Swatinem

## 6.2.0

**Features**:

- Add `source_root` support for `SourceMap` and `SourceMapBuilder`, with respective getters/setters and de/serialization. ([#51](https://github.com/getsentry/rust-sourcemap/pull/51))

## 6.1.0

**Features**:

- Add a new `get_scope_for_token` method to `SourceMapHermes` as a more flexible alternative to `get_original_function_name`. ([#48](https://github.com/getsentry/rust-sourcemap/pull/48))

## 6.0.2

**Fixes**:

- Improve parsing performance by reusing a temporary allocation. [#40](https://github.com/getsentry/rust-sourcemap/pull/40)

## 6.0.1

**Fixes**:

- Fix compilation errors when targetting wasm.

## 6.0.0

**Breaking Changes**:

- The `SourceMapRef::Missing` variant was removed in favor of explicit `Option`s.
- The `locate_sourcemap_reference_slice` and `locate_sourcemap_reference` functions now return a `Option`.
- `SourceMapRef::get_url` does _not_ return an `Option` anymore.

**Features**:

- Added missing `Clone` and `Debug` impls to a lot of types.
- A lot of new convenience API, including a `DecodedMap::get_original_function_name` method that works across all supported SourceMap types.
