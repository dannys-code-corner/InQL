# Variant Functions (Reference)

Variant helpers model semi-structured payloads as typed logical values, not as ordinary JSON strings. Use RFC 022 JSON
helpers when you want text validation or normalized payload strings. Use variant helpers when you need kind-aware
inspection while preserving the distinction between SQL null and a present semi-structured null value.

| Function | Meaning |
| --- | --- |
| `variant_type(kind=VariantKind.Any, encoding=VariantEncoding.Json)` | Build variant logical type metadata. |
| `variant_col(name, variant_type=variant_type())` | Reference a column as typed variant state. |
| `variant_value(value, variant_type)` | Attach variant logical metadata to a primitive value or scalar expression. |
| `parse_variant_json(payload)` | Strictly parse JSON text from a string value or string-producing expression into typed variant state. |
| `try_parse_variant_json(payload)` | Parse JSON text from a string value or string-producing expression into typed variant state with recoverable malformed input. |
| `variant_get(variant, path)` | Access a path from a typed variant value. Literal paths are validated as `$`-rooted; string columns or expressions are accepted as dynamic paths. |
| `typeof(variant)` | Return the stable lowercase variant kind name. |
| `is_null_value(variant)` | Return whether the value is a present semi-structured null. |
| `is_boolean(variant)` | Return whether the value is a present boolean. |
| `is_integer(variant)` | Return whether the value is a present integer. |
| `is_float(variant)` | Return whether the value is a present floating-point value. |
| `is_string(variant)` | Return whether the value is a present string. |
| `is_timestamp(variant)` | Return whether the value is a present timestamp. |
| `is_array(variant)` | Return whether the value is a present array. |
| `is_object(variant)` | Return whether the value is a present object. |

```incan
from pub::inql.functions import col, is_array, is_null_value, parse_variant_json, typeof, variant_get

payload = parse_variant_json(col("payload"))
literal_payload = parse_variant_json("{\"status\":\"paid\"}")

projected = (
    events
        .with_column("payload_kind", typeof(payload))
        .with_column("items_are_array", is_array(variant_get(payload, "$.items")))
        .with_column("dynamic_value", variant_get(literal_payload, col("json_path")))
        .with_column("deleted_was_variant_null", is_null_value(variant_get(payload, "$.deleted_at")))
)
```

`typeof(...)` accepts a `VariantExpr` value and returns a `StringColumnExpr`. Variant predicates accept
`VariantExpr` values and return `BoolColumnExpr` values. They do not parse strings directly. Parse helpers accept
`StrValueOrColumn` inputs; that keeps parsing, variant inspection, and RFC 022 JSON text helpers separate without
forcing authors to wrap literal payloads in `lit(...)`.

RFC 026 helpers lower through InQL-owned Substrait extension mappings and carry variant metadata in function options.
The DataFusion adapter currently reports a backend planning diagnostic for typed variant execution because it has no
variant runtime implementation. That rejection is an adapter capability boundary; the InQL plan remains typed and
backend-neutral.
