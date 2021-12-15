from typing import Optional

import pytest

from pre.common import filter_data_with_types, get_defaults, types_from_annotations


def test_types_from_annotations():
    def _fn(a: int, b: float, c: Optional[int]):
        pass

    types = types_from_annotations(_fn)

    assert types.get("a") == int
    assert types.get("b") == float
    assert types.get("c") == (int, type(None))


def test_filter_with_types():
    data = {"a": 1, "b": "str"}
    types = {"a": int, "b": str}

    # check types are ok
    assert filter_data_with_types(data, types) == data

    # extra not ok
    with pytest.raises(ValueError, match="Extra keys found"):
        filter_data_with_types({**data, "extra_key": 12}, types, allow_extras=False)

    # extra ok
    assert (
        filter_data_with_types({**data, "extra_key": 12}, types, allow_extras=True)
        == data
    )

    # empty is ok
    assert filter_data_with_types({}, types) == {}

    # bad type
    assert "a" not in filter_data_with_types({"a": "asd"}, types)


def test_get_func_defaults():
    def _fn(a: int = 1, b: str = "str"):
        pass

    defaults = get_defaults(_fn)

    assert defaults.get("a") == 1
    assert defaults.get("b") == "str"
