# Conformance tests made simple

Example test file:

```

=========
test name
optional test description
=========
test input
=========
test output
=========

============
another test
the fences must all be the same length
============
this input is what is passed to your function (including trailing newline)
the single newline after the leading fence is stripped
============
test output is any serde data format
it is normalized before comparison
============
```

Tests are typically dispatched through the standard test harness:

```rust
use { conformance, serde, serde_yaml as yaml };

#[conformance::test(exact, serde=yaml, file="tests/example.test")]
fn example_test(s: &str) -> impl serde::Serialize {
    // Do the transformation that you want to test.
    // The output type can be any type that implements Serialize.
    // The function remains and can be used by other code!
    // You can also attach multiple `conformance::test` attributes to one fn. 
}
```

The arguments to `conformance::tests`:

- `exact`: the mode to match in. Only `exact` is currently supported.
  The generated and expected output must match exactly (after normalization).
- `serde`: the data format to target. Stands in for three arguments, `ser`, `de`, and `value`.
  Instead of `serde`, you can provide just `ser`, `de`, and optionally `value`,
  or you can override any of `ser`/`de`/`value` after providing `serde`.
    - `ser`: path to a function of shape `fn<T>(&T) -> Result<String, impl Error>`.
      Default: `#serde::to_string`.
    - `de`: path to a function of shape `fn<T>(&str) -> Result<T, impl Error>`.
      Default: `#serde::from_str`.
    - `value`: a type that can be deserialized from the expected output.
      Default: `#serde::Value`, or the annotated function's return type if `serde` is not provided.
- `file`: A filesystem path relative to `Cargo.toml` to the test file.
  Can also be `glob` to enable [glob paths](https://docs.rs/glob/).

Setting the environment variable `CONFORMANCE_BLESS` will cause tests to
overwrite the expected output if it does not match with the produced output.
(They will still fail on the first run, however.)<sup>†</sup>

By default, all test file reading and parsing is done at compile time.
If this is not desired, you can enable the `runtime` feature<sup>†</sup>.
This adds two new attributes, `conformance::test_case`, which has all the same
arguments as `conformance::test`, and `conformance::main`, which is used to
collect all of your test cases and generate a test runner for them:

```rust
#[conformance::test_case(exact, serde=yaml, file="tests/example.test")]
fn example_test_case(s: &str) -> impl serde::Serialize {
    // Do the transformation that you want to test.
    // The output type can be any type that implements Serialize.
    // Only one `conformance::test_case` attribute can be attached to a fn;
    // the function item is replaced with a static `conformance::TestCase`.
}

#[conformance::main]
static TESTS: &[conformance::TestCase] = &[example_test_case];
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

If you are a highly paid worker at any company that prioritises profit over
people, you can still use this crate. I simply wish you will unionise and push
back against the obsession for growth, control, and power that is rampant in
your workplace. Please take a stand against the horrible working conditions
they inflict on your lesser paid colleagues, and more generally their gross
disrespect for the very human rights they claim to fight for.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

---

<sup>†</sup> Not yet implemented
