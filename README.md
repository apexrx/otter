# Welcome to Otter

## What's Otter?

Otter is a JSON enforcement library for LLM output. It takes raw text from a language model and a JSON Schema, and returns valid, schema-conformant JSON — or a precise correction prompt when it cannot.

LLM output is usually almost correct. Missing quotes, wrong types, slight key drift — mechanical errors with deterministic fixes. Otter resolves these automatically. When errors exceed safe repair, it does not guess. It generates a structured correction prompt so retries are targeted.

Otter never mutates beyond schema constraints. No probabilistic fixes. Every transformation is explicit and auditable.

---

## API

### enforce(input, schema)

Primary entrypoint.

Accepts raw LLM output + schema. Runs repair → validate → enforce.

```python id="k1q8zs"
result = otter.enforce(input, schema)
```

Returns:

* `status`: Valid | Repaired | NeedsCorrection | InvalidSchema
* `json`: final output (if enforceable)
* `prompt`: correction prompt (if needed)

---

### validate(json, schema)

Parse and validate. No mutation.

Returns:

* `status`: Valid | ParseError | SchemaErrors | InvalidSchema

---

### repair(input, schema)

Apply deterministic heuristic fixes.

Returns:

* `repaired`: fixed JSON
* `rules`: applied fixes
* `confidence_level`: 0.0–1.0

---

### generate_prompt(invalid_json, schema)

Build correction prompt from validation errors.

Example:

```id="y6f2ra"
Fix 1 violation:
- age: expected number, got string

Return valid JSON only.
```

Full format → `/docs/prompts.md`

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
