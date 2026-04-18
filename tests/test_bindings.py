import otter


def test_valid_json_and_schema():
    valid_json = '{"name": "test", "value": 42}'
    valid_schema = '{"type": "object", "properties": {"name": {"type": "string"}, "value": {"type": "number"}}}'

    result = otter.validate_py(valid_json, valid_schema)

    assert result.status == otter.ValidationStatus.Valid, (
        f"Expected Valid, got {result.status!r}"
    )


def test_invalid_schema_raises_valueerror():
    broken_schema = "{invalid json}"

    try:
        otter.validate_py('{"key": "value"}', broken_schema)
        assert False, "Expected ValueError to be raised"
    except ValueError:
        pass


def test_enforce_repairs_json():
    broken_json = '{"name": "test", "value": 42, }'
    valid_schema = '{"type": "object", "properties": {"name": {"type": "string"}, "value": {"type": "number"}}}'

    result = otter.enforce_py(broken_json, valid_schema)

    assert result.status == "Repaired", f"Expected 'Repaired', got {result.status!r}"


if __name__ == "__main__":
    test_valid_json_and_schema()
    test_invalid_schema_raises_valueerror()
    test_enforce_repairs_json()
    print("All tests passed!")
