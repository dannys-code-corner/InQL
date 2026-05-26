# Format Functions (Reference)

Format helpers operate on scalar payloads that are already present in a relation. They do not read files, infer source
schemas from external locations, or change relation cardinality.

The current implemented slice is deterministic string hashing:

| Function | Meaning |
| --- | --- |
| `md5(expr)` | Return the lowercase hexadecimal MD5 digest for one string expression. |
| `sha224(expr)` | Return the lowercase hexadecimal SHA-224 digest for one string expression. |
| `sha256(expr)` | Return the lowercase hexadecimal SHA-256 digest for one string expression. |
| `sha384(expr)` | Return the lowercase hexadecimal SHA-384 digest for one string expression. |
| `sha512(expr)` | Return the lowercase hexadecimal SHA-512 digest for one string expression. |
| `sha2(expr, bit_length)` | Compatibility helper that rewrites to `sha224`, `sha256`, `sha384`, or `sha512` for supported literal bit lengths. |

```incan
from pub::inql.functions import col, md5, sha2

projected = (
    events
        .with_column("user_hash", sha2(col("user_id"), 256))
        .with_column("payload_md5", md5(col("payload")))
)
```

Hash helpers operate on UTF-8 string bytes and return lowercase hexadecimal strings. `sha2(...)` accepts `224`, `256`,
`384`, and `512`; unsupported digest lengths are rejected by the helper rather than being passed through to a backend.

JSON, CSV, URL, and dynamic-value predicate helpers remain future format-function slices until their schema arguments,
option records, path validation rules, and dynamic value model are specified.
