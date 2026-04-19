import traceback

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


def run_tests():
    tests = [
        test_valid_json_and_schema,
        test_invalid_schema_raises_valueerror,
        test_enforce_repairs_json,
    ]

    passed = 0
    failed = 0

    for test in tests:
        try:
            test()
            print(f"  PASS  {test.__name__}")
            passed += 1
        except Exception as e:
            print(f"  FAIL  {test.__name__}: {e}")
            traceback.print_exc()
            failed += 1

    print(f"\n{passed + failed} tests run — {passed} passed, {failed} failed.")
    if failed:
        print("Some tests FAILED.")
    else:
        print("All tests passed!")


if __name__ == "__main__":
    run_tests()
