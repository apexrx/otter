# Preprocessor

Before the repair engine runs, Otter passes raw LLM output through a sanitization step. The preprocessor's job is narrow: ensure that string literals in the input contain only characters that a JSON parser will accept. It does not validate, repair structure, or touch anything outside of strings.

## What `sanitize_json` Does

`sanitize_json` walks the input character by character, tracking whether the cursor is inside a string literal. Outside strings, characters are passed through unchanged. Inside strings, two categories of problem are fixed:

**Invalid escape sequences.** When a backslash is encountered, the following character is checked against the set of escapes the JSON spec permits: `"`, `\`, `/`, `b`, `f`, `n`, `r`, `t`, and `u`. If the character is in that set, it is passed through as-is. If it isn't, the backslash is kept and the character is handled as a literal — if it's a control character it is properly escaped, otherwise it is emitted directly. This recovers from outputs where a model has written something like `\p` or `\s` that would cause a strict parser to reject the document.

**Raw control characters.** Any character below `U+0020` inside a string is illegal in JSON and must be escaped. `sanitize_json` catches these and converts them to their correct escape sequences before they reach the parser.

Control character mapping:

| Character | Escape |
|---|---|
| `\n` (line feed) | `\n` |
| `\r` (carriage return) | `\r` |
| `\t` (tab) | `\t` |
| `\x08` (backspace) | `\b` |
| `\x0C` (form feed) | `\f` |
| Any other `< U+0020` | `\uXXXX` |

## What It Doesn't Do

`sanitize_json` does not fix structural problems. It does not close unclosed strings, balance brackets, remove markdown fences, or touch characters outside string literals. Those are the repair engine's responsibilities. The preprocessor runs first precisely because the repair engine's string-level rules work on raw text — giving them input where string contents are already clean reduces the surface area for regex rules to misfire.

## Allocation

`sanitize_json` pre-allocates the result buffer to the length of the input. Most inputs come through unchanged or slightly larger, so this avoids reallocation in the common case.
