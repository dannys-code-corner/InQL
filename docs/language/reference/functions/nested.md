# Nested Data Functions (Reference)

Nested data helpers build and inspect row-level arrays, maps, and structs. They are scalar expressions: every helper returns one value for each input row and does not change relation cardinality.

Generator or table-valued operations such as row-expanding `explode(...)` are separate from this page.

## Arrays

| Function | Meaning |
| --- | --- |
| `array(values)` | Build an array expression from one or more scalar expressions. |
| `cardinality(value)` | Return the size of an array or map. |
| `array_contains(array_expr, value)` | Return whether an array contains a value. |
| `arrays_overlap(left, right)` | Return whether two arrays have any elements in common. |
| `array_position(array_expr, value)` | Return the one-based position of a value. |
| `element_at(array_expr, index)` | Return an array element by one-based index. |
| `array_sort(array_expr)` | Sort one array value. |
| `array_distinct(array_expr)` | Remove duplicate elements from one array value. |
| `array_except(left, right)` | Return elements from `left` that are not in `right`. |
| `array_intersect(left, right)` | Return elements shared by both arrays. |
| `array_union(left, right)` | Return the union of both arrays. |
| `array_join(array_expr, delimiter)` | Join a string array into one string. |
| `array_range(start, stop)` | Build a row-level integer array from `start` inclusive to `stop` exclusive. |
| `array_slice(array_expr, start, stop)` | Return a one-based array slice using the backend adapter's slice contract. |
| `array_reverse(array_expr)` | Reverse one array value. |
| `array_flatten(array_expr)` | Flatten an array-of-arrays into one row-level array value. |

## Maps And Structs

| Function | Meaning |
| --- | --- |
| `map_from_arrays(keys, values)` | Build a map from key and value arrays. |
| `map_extract(map_expr, key)` | Return the values associated with a key. |
| `map_contains_key(map_expr, key)` | Return whether `map_extract(...)` finds at least one value for the key. |
| `map_keys(map_expr)` | Return the map's keys as an array. |
| `map_values(map_expr)` | Return the map's values as an array. |
| `map_entries(map_expr)` | Return map entries. |
| `named_struct(field_names, values)` | Build a struct expression with explicit field names. |

## Example

```incan
from pub::inql.functions import array, array_contains, cardinality, col, element_at, lit

projected = (
    events
        .with_column("tags", array([lit("paid"), col("source")]))
        .with_column("tag_count", cardinality(col("tags")))
        .with_column("has_paid_tag", array_contains(col("tags"), lit("paid")))
        .with_column("first_tag", element_at(col("tags"), lit(1)))
)
```

## Semantics

- Array indexing is one-based for `element_at(...)`, `array_position(...)`, and `array_slice(...)`.
- `element_at(...)` currently maps to the portable array-element adapter path. Out-of-range behavior follows the current backend adapter's recoverable result until InQL has a richer static/runtime error-policy split for strict versus try-style element access.
- `array_flatten(...)` is intentionally named to stay distinct from the relation-shaping generator `flatten(...)`.
- Grouping or ordering by nested values is not documented as portable until equality and ordering semantics for arrays, maps, and structs are specified.
