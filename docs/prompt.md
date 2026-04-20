# Prompt Format

When `enforce` cannot repair invalid JSON, it calls `generate_correction_prompt` internally and returns the result as the `prompt` field on the result object. You can also call `generate_correction_prompt` directly if you're managing your own retry loop.

The prompt format varies by failure type.

## Parse Error

When the input isn't valid JSON at all, the prompt tells the model exactly where parsing broke down:

```
Your previous response was not valid JSON and could not be parsed. Parse error: Unexpected token '}' at line 4, column 12. Please return only valid JSON with no additional text, markdown, or code fences.
```

The parse error message is taken directly from the parser and capped at 120 characters. Line and column numbers are always included so the model has a precise location to work from.

## Schema Violations

When the input is valid JSON but doesn't conform to the schema, the prompt lists each violation by its JSON path, includes the expected type where the schema provides one, and appends the full schema so the model has complete context for its retry.

```
Your previous response was valid JSON but did not conform to the required schema.
Please fix the following 2 violation(s) and try again:
1. At '/age': <validator message> (expected: number)
2. At '/address/zip': <validator message> (expected: string)

The required schema is:
{
  "type": "object",
  "properties": { ... }
}

Return only valid JSON that satisfies this schema, with no additional text, markdown, or code fences.
```

Violations are numbered and each one references its exact field path in JSON Pointer notation (`/age`, `/address/zip`). The violation message is passed through from the validator as-is. Type hints are appended inline when `extract_type` can resolve the expected type by walking the schema to that path.

`extract_type` navigates object properties via `properties` and array positions via `items` or `prefixItems`. For numeric path segments, it checks `items` first — if present, it follows that node regardless of the index, since `items` describes a uniform array type. `prefixItems` is used when `items` is absent, allowing index-specific type resolution for tuple schemas. If the path leads through a structure the walker doesn't recognise, the hint is omitted rather than guessed at.

When the violations list is empty — valid JSON that fails schema conformance but with no enumerable violations — the prompt omits the list and includes only the full schema, asking the model to review it in full.

## Invalid Schema

`generate_correction_prompt` returns an `Err` for `InvalidSchema` rather than a prompt string. This is intentional — a broken schema is a programmer error, not a model error. No prompt is generated and the retry loop should not proceed. `enforce` surfaces this as an `InvalidSchema` status on the result.

## Design Notes

**The schema is always included in schema error prompts.** The model needs full context to fix violations correctly; asking it to correct field-level errors without the schema would produce speculative retries.

**Type hints are best-effort.** `extract_type` walks the schema along the violation path and reads the `type` field at the destination node. If the path cannot be resolved, the hint is silently omitted.

**Prompts are plain text.** No markdown, no JSON fences in the prompt itself. The instruction to return JSON without fences or prose is included explicitly at the end of every prompt, since LLMs tend to wrap JSON in code blocks by default.
