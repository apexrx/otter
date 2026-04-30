# Repair Engine

Otter's repair engine is the second stage of the `enforce` pipeline. It takes raw LLM output — which may not be valid JSON at all — and applies a sequence of deterministic transformations to recover something parseable and schema-conformant. Every transformation is a named rule. Every rule has a cost. Nothing is guessed.

## How Repair Works

`repair` runs the input through two phases in order.

**String-level repairs** operate on the raw text before any JSON parsing is attempted. These handle the structural damage that makes input unparseable: markdown fences wrapping the output, stray characters before or after the JSON payload, truncation, trailing commas, single quotes used instead of double quotes, unquoted object keys, and Python boolean literals (`True`, `False`, `None`). Each rule is applied independently and in a fixed sequence — if a rule's regex or heuristic produces no change, it is skipped and not recorded.

**Schema-level repairs** operate on the parsed JSON value after the string-level pass succeeds. These handle type mismatches and null fields that the schema can resolve: strings that should be numbers are coerced, null fields with a schema `default` are replaced with that default, array items of the wrong type are dropped, and repairs are applied recursively through object properties and array contents.

If the string-level pass leaves something that still can't be parsed, schema-level repairs are skipped and the unmodified string is returned with whatever rules were applied up to that point.

## Repair Rules

Each rule is a variant of `RepairRule` and carries a cost between `0.0` and `1.0`. Cost represents how structurally significant the repair was — low-cost rules fix superficial formatting, high-cost rules indicate deeper damage. The aggregate cost feeds into the `confidence_level` on the result.

| Rule | What it fixes | Cost |
|---|---|---|
| `StripMarkdownFences` | ` ```json ``` ` wrappers around the output | 0.02 |
| `ExtractJsonPayload` | Leading/trailing non-JSON text around `{...}` or `[...]` | 0.05 |
| `FixTrailingCommas` | Commas before `}` or `]` | 0.05 |
| `FixWrongNumericTypes` | String values that should be numbers per schema | 0.05 |
| `FixPythonBooleans` | `True` / `False` / `None` → `true` / `false` / `null` | 0.08 |
| `FixNullValues` | `null` fields replaced with schema `default` | 0.10 |
| `FixSingleQuotes` | Single-quoted strings → double-quoted | 0.12 |
| `FixUnquotedKeys` | Bare object keys → quoted keys | 0.15 |
| `ArrayItemsDropped { count }` | Array items of the wrong type, dropped per item | 0.15 × count |
| `FixTruncatedJson` | Unclosed strings, brackets, and braces | 0.40 |
| `Custom { name, description, cost }` | User-defined rule | 0.0–1.0 |

Rules are applied in the order listed above for string-level repairs. Schema-level repairs (`FixWrongNumericTypes`, `FixNullValues`, `ArrayItemsDropped`) are applied after parsing, recursively through the value tree.

## Confidence Level

`RepairResult` carries a `confidence_level` between `0.0` and `1.0`. It is computed from the aggregate cost of applied rules — more invasive repairs produce lower confidence. A result with only `StripMarkdownFences` applied is near-certain. A result that required `FixTruncatedJson` plus several other rules should be treated with more caution.

`confidence_level` is informational. Otter does not use it internally to gate behaviour — that decision belongs to the consuming application.

## RepairResult

```rust
pub struct RepairResult {
    pub repaired: String,       // the repaired JSON string
    pub rule: Vec<RepairRule>,  // rules applied, in order
    pub confidence_level: f32,  // aggregate confidence after all repairs
}
```

`rule` is empty if no repairs were needed. `repaired` always contains the best output Otter could produce — callers should not assume it is valid JSON without checking the result of a subsequent `validate` call, which is what `enforce` does internally.

## Custom Rules

`RepairRule::Custom` allows consuming applications to register their own named rules into the audit trail with an explicit cost. Otter does not apply custom rules itself — they are intended to be pushed into `rule` by the caller when the caller performs its own pre-processing before handing input to Otter.

## Validation

`validate` is separate from repair. It parses the input and checks it against the schema using a JSON Schema validator, returning one of four states: `Valid`, `ParseError`, `SchemaErrors`, or `InvalidSchema`. It never mutates. Calling `validate` directly is useful when you want to check already-clean output, or when you want to separate the repair and validation steps in your own pipeline.

---
