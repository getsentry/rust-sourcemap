The sourcemap here was generated like this:

```sh
    $ hermes -O -emit-binary -output-source-map -out=output input.js
```

When running the bytecode, we get the following stacktrace:

```
Error: lets throw!
    at foo (address at unknown:1:57)
    at global (address at unknown:1:27)
```