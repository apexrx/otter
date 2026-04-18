from enum import Enum

class ValidationStatus(str, Enum):
    """Validation state.

    Values:
        Valid
        ParseError
        SchemaErrors
        InvalidSchema
    """

    Valid = "Valid"
    ParseError = "ParseError"
    SchemaErrors = "SchemaErrors"
    InvalidSchema = "InvalidSchema"

class PyRepairRule:
    """Single repair rule.

    Attributes:
        name (str): Rule identifier.
        description (str | None): Optional detail.
        cost (float): Heuristic cost.
    """

    name: str
    description: str | None
    cost: float

class PyRepairResult:
    """Repair output.

    Attributes:
        repaired (str): Final JSON.
        rules (list[PyRepairRule]): Applied rules.
        confidence_level (float): Heuristic score.
    """

    repaired: str
    rules: list[PyRepairRule]
    confidence_level: float

class PySchemaViolation:
    """Schema violation entry.

    Attributes:
        path (str): JSON path.
        message (str): Violation reason.
        invalid_value (str | None): Offending value.
    """

    path: str
    message: str
    invalid_value: str | None

class PyParseErrorInfo:
    """JSON parse error.

    Attributes:
        line (int): Line number.
        column (int): Column number.
        message (str): Error detail.
    """

    line: int
    column: int
    message: str

class PyValidationReport:
    """Validation result.

    Attributes:
        status (ValidationStatus): Overall state.
        parsed (str | None): Parsed JSON if valid.
        parse_error (PyParseErrorInfo | None): Parse failure info.
        schema_violations (list[PySchemaViolation] | None): Violations.
        invalid_schema_message (str | None): Schema error.
    """

    status: ValidationStatus
    parsed: str | None
    parse_error: PyParseErrorInfo | None
    schema_violations: list[PySchemaViolation] | None
    invalid_schema_message: str | None

class PyEnforcementResult:
    """Enforcement outcome.

    Attributes:
        status (str): Result state.
        json (str | None): Valid/repaired JSON.
        rules_applied (list[PyRepairRule] | None): Applied fixes.
        prompt (str | None): Correction prompt.
        error (str | None): Failure reason.
    """

    status: str
    json: str | None
    rules_applied: list[PyRepairRule] | None
    prompt: str | None
    error: str | None

def repair_py(input: str, schema: str) -> PyRepairResult:
    """Repair JSON via heuristics + schema fixes.

    Steps:
        fences → extract → truncate → commas → quotes → keys → bools → schema

    Args:
        input (str): Raw JSON/text.
        schema (str): JSON Schema.

    Returns:
        PyRepairResult: repaired JSON, applied rules, confidence.
    """

def enforce_py(input: str, schema: str) -> PyEnforcementResult:
    """Repair → validate → decide outcome.

    States:
        Valid | Repaired | NeedsCorrection | InvalidSchema

    Args:
        input (str): Raw JSON/text.
        schema (str): JSON Schema.

    Returns:
        PyEnforcementResult: json | rules_applied | prompt | error.
    """

def validate_py(output: str, schema: str) -> PyValidationReport:
    """Parse + schema validate (no mutation).

    Args:
        output (str): JSON string.
        schema (str): JSON Schema.

    Returns:
        PyValidationReport: parse status + violations.
    """

def generate_prompt_py(output: str, schema: str) -> str:
    """Build correction prompt from validation errors.

    Args:
        output (str): JSON string.
        schema (str): JSON Schema.

    Returns:
        str: prompt or empty if valid.
    """
