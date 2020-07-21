# Changelog

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
