# Changelog

## Unreleased

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
