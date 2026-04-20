# Welcome to Otter

## What's Otter?

Otter is a JSON enforcement library for LLM output. It takes raw text from a language model and a JSON Schema, and returns valid, schema-conformant JSON — or a precise correction prompt when it cannot.

LLM output is usually almost correct. Missing quotes, wrong types, slight key drift — mechanical errors with deterministic fixes. Otter resolves these automatically. When errors exceed safe repair, it does not guess. It generates a structured correction prompt so retries are targeted.

Otter never mutates beyond schema constraints. No probabilistic fixes. Every transformation is explicit and auditable.

---

## API

### `enforce(input, schema)`

The primary entrypoint — and for most use cases, the only one you need. Accepts raw LLM output and a JSON Schema, attempts deterministic repair, validates the result, and returns one of four outcomes: conformant JSON, repaired JSON with the fixes applied, a correction prompt for the model, or an InvalidSchema error if the schema itself is malformed.

```python id="k1q8zs"
result = otter.enforce(input, schema)
```

The result always has a `status` field, which is one of `Valid`, `Repaired`, `NeedsCorrection`, or `InvalidSchema`. Depending on status, it also carries `json` (the final conformant output) or `prompt` (a correction prompt for the model). See [Result States](#result-states) below.

### `validate(json, schema)`

Parse and validate without mutation. Useful when you want to check output you've already cleaned, or when you want validation separated from repair. Returns a status of `Valid`, `ParseError`, `SchemaErrors`, or `InvalidSchema`.

### `repair(input, schema)`

Apply heuristic fixes and return the repaired JSON alongside the list of rules applied and a `confidence_level` between `0.0` and `1.0`. Use this directly when you want visibility into what was changed before committing to the result.

### `generate_prompt(invalid_json, schema)`

Build a correction prompt from validation errors. This is what `enforce` calls internally when repair isn't sufficient, but you can call it directly to integrate correction prompts into your own retry loop.

```
Fix 1 violation:
- age: expected number, got string

Return valid JSON only.
```

Full prompt format is documented in [`/docs/prompts.md`](/docs/prompt.md).

---

## Result States

| State             | Meaning            | Output       |
| ----------------- | ------------------ | ------------ |
| `Valid`           | Already conformant | JSON         |
| `Repaired`        | Auto-fixed         | JSON + rules |
| `NeedsCorrection` | Cannot repair      | Prompt       |
| `InvalidSchema`   | Schema error       | Error        |

---

## Design Principles

Otter is built around a few firm commitments:

**Enforcement over validation.** Returning an error is the last resort, not the first response. The goal is always conformant JSON, not a report of why conformance failed.

**Repair before reject.** Heuristic fixes handle the common case. Correction prompts handle the rest. A raw error is only returned when the schema itself is broken.

**The schema defines truth.** Otter does not infer intent. It does not guess at what the model probably meant. The schema is the contract, and everything Otter does is in service of that contract.

**Deterministic transformations.** Given the same input and schema, Otter always produces the same output. There is no randomness, no sampling, no model call inside the repair path.

---

## Non-Goals

Otter is not a general ETL pipeline. It does not generate schemas from examples. It does not apply probabilistic or model-based fixes. It does not handle streaming output. If you need those things, Otter is not the right tool for that layer.

---

## Install
> **Note:** Otter is not yet published to crates.io, PyPI, or npm. The install commands below reflect the intended distribution — see the local build instructions to run from source in the meantime.

### Rust (Core Library)

```bash id="z0r5pf"
cargo add otter
```

Or in `Cargo.toml`:

```toml id="j5u7kd"
[dependencies]
otter = "0.6.1"
```

---

### Python (Native Bindings)

Prebuilt wheels via PyPI. No Rust toolchain required.

```bash id="z2o8nc"
pip install otter
```

Requires Python 3.8+

Build locally (contributors):

```bash id="k4x8tn"
make build-python
pip install target/wheels/otter-*.whl --force-reinstall
```

---

### WebAssembly (Browser & Edge)

**Option 1: npm / bundlers**

```bash id="d3s9qa"
npm install otter-wasm
```

**Option 2: ES Modules**

```bash id="l2p7fd"
make build-wasm
```

```html id="q8v1mw"
<script type="module">
  import init, { enforce_wasm } from './pkg/otter.js';
  await init();
</script>
```

---

## License

Otter is released under the [MIT License](https://opensource.org/licenses/MIT).
