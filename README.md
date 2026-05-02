# halo2-practice

[![Rust tests](https://github.com/mdvillagra/halo2-practice/actions/workflows/rust.yml/badge.svg)](https://github.com/mdvillagra/halo2-practice/actions/workflows/rust.yml)

Small Halo2 practice repository that implements a custom PLONK-style gate and verifies it with `MockProver`.

The circuit uses private witnesses `x` and `y` and public inputs `c` and `z` to check:

```text
z = x^2 * y^2 + c
```

The code is intentionally compact and kept in `src/main.rs` so the full flow is easy to inspect: column configuration, custom gate construction, witness assignment, equality constraints, public input exposure, and mock proving.

## What to look at

- `CustomConfig`: advice, fixed, and instance columns for the mini composer.
- `meta.create_gate("mini plonk", ...)`: constraint expression for addition and multiplication rows.
- `CustomChip`: helper methods for raw addition, raw multiplication, copying cells, and exposing public inputs.
- `SampleCircuit`: witness synthesis for the arithmetic relation.
- `main`: concrete example using `x = 5`, `y = 9`, `c = 7`, and `z = 25 * 81 + 7`.

## Running locally

```bash
cargo test
cargo run
```

`cargo run` builds the sample circuit and asserts that `MockProver::verify()` succeeds.

## Status

This is a learning and experimentation repository, not a reusable proving library. It is useful as a readable Halo2 example for custom gates, cell copying, and public input constraints.
