# Format Functions (Reference)

Format functions transform scalar values that are already present in a relation. Source discovery, file reads, and
relation reshaping belong to the session and relational APIs rather than this function family.

The format catalog includes deterministic hashes, URL helpers, JSON helpers, and CSV helpers:

| Function | Meaning |
| --- | --- |
| `md5(expr)` | Return the lowercase hexadecimal MD5 digest for a string expression. |
| `sha1(expr)` | Return the lowercase hexadecimal SHA-1 digest for a string expression. |
| `sha224(expr)` | Return the lowercase hexadecimal SHA-224 digest for a string expression. |
| `sha256(expr)` | Return the lowercase hexadecimal SHA-256 digest for a string expression. |
| `sha384(expr)` | Return the lowercase hexadecimal SHA-384 digest for a string expression. |
| `sha512(expr)` | Return the lowercase hexadecimal SHA-512 digest for a string expression. |
| `sha2(expr, bit_length)` | Compatibility helper that rewrites to `sha224`, `sha256`, `sha384`, or `sha512` for supported literal bit lengths. |
| `crc32(expr)` | Return the lowercase eight-character hexadecimal CRC-32 digest for a string expression. |
| `xxhash64(expr)` | Return the lowercase sixteen-character hexadecimal xxHash64 digest for a string expression. |
| `parse_url(expr, key)` | Extract the first query parameter value for `key` from a URL string, returning null when the key is absent. |
| `url_encode(expr)` | Percent-encode a URL component string. |
| `url_decode(expr)` | Decode a percent-encoded URL component string and fail on malformed escapes. |
| `try_url_decode(expr)` | Decode a percent-encoded URL component string, returning null on malformed escapes. |
| `parse_json(expr)` | Validate and normalize a JSON payload string. |
| `check_json(expr)` | Return whether a string expression contains valid JSON. |
| `schema_of_json(expr)` | Infer a deterministic schema description from a JSON payload string. |
| `json_array_length(expr)` | Return the number of array elements for a JSON array payload, or null for non-array payloads. |
| `json_object_keys(expr)` | Return object keys from a JSON object payload as a JSON array string. |
| `get_json_object(expr, path)` | Extract a JSON value at a literal path and return it as JSON text. |
| `json_extract_path_text(expr, path)` | Extract a JSON value at a literal path and return scalar strings as plain text. |
| `from_json(expr, schema)` | Validate JSON with an explicit schema description and return a normalized JSON payload string. |
| `try_from_json(expr, schema)` | Validate JSON with an explicit schema description and return null when the payload is invalid. |
| `to_json(expr)` | Serialize a scalar expression as JSON text. |
| `schema_of_csv(expr)` | Infer a deterministic schema description from a CSV row string. |
| `from_csv(expr, schema)` | Parse a CSV row string into a logical map keyed by schema field names, or `_cN` positional keys when no schema is provided. |
| `to_csv(expr)` | Serialize a scalar or JSON array/object payload as a CSV row string. |

```incan
from pub::inql.functions import col, from_csv, from_json, get_json_object, parse_url, sha2, to_json

projected = (
    events
        .with_column("user_hash", sha2(col("user_id"), 256))
        .with_column("campaign", parse_url(col("landing_page"), "utm_campaign"))
        .with_column("event_type", get_json_object(col("payload"), "$.type"))
        .with_column("payload_obj", from_json(col("payload"), "STRUCT<type: STRING>"))
        .with_column("row_fields", from_csv(col("csv_line"), "STRUCT<id: STRING, status: STRING>"))
        .with_column("payload_out", to_json(col("event_type")))
)
```

Hash helpers operate on UTF-8 string bytes and return lowercase hexadecimal strings. `sha2(...)` accepts `224`, `256`,
`384`, and `512`; other digest lengths are rejected during expression construction.

JSON helpers validate, normalize, and project payload text. CSV parsing returns logical map values instead of JSON text.
These helpers do not read external files or introduce a dynamic variant value type.

The DataFusion adapter executes the full RFC 022 catalog with native DataFusion functions where available and
Incan-authored adapter callbacks for helpers that DataFusion does not expose natively.
