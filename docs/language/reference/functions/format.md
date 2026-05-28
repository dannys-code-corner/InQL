# Format Functions (Reference)

Format helpers operate on scalar payloads that are already present in a relation. They do not read files, infer source
schemas from external locations, or change relation cardinality.

The implemented RFC 022 surface covers deterministic hashes, URL helpers, JSON scalar payload helpers, and CSV scalar
payload helpers:

| Function | Meaning |
| --- | --- |
| `md5(expr)` | Return the lowercase hexadecimal MD5 digest for one string expression. |
| `sha1(expr)` | Return the lowercase hexadecimal SHA-1 digest for one string expression. |
| `sha224(expr)` | Return the lowercase hexadecimal SHA-224 digest for one string expression. |
| `sha256(expr)` | Return the lowercase hexadecimal SHA-256 digest for one string expression. |
| `sha384(expr)` | Return the lowercase hexadecimal SHA-384 digest for one string expression. |
| `sha512(expr)` | Return the lowercase hexadecimal SHA-512 digest for one string expression. |
| `sha2(expr, bit_length)` | Compatibility helper that rewrites to `sha224`, `sha256`, `sha384`, or `sha512` for supported literal bit lengths. |
| `crc32(expr)` | Return the lowercase eight-character hexadecimal CRC-32 digest for one string expression. |
| `xxhash64(expr)` | Return the lowercase sixteen-character hexadecimal xxHash64 digest for one string expression. |
| `parse_url(expr, part)` | Extract `scheme`, `host`, `path`, `query`, `fragment`, `port`, `username`, or `password` from a URL string. |
| `url_encode(expr)` | Percent-encode one URL component string. |
| `url_decode(expr)` | Strictly decode one percent-encoded URL component string and fail on malformed escapes. |
| `try_url_decode(expr)` | Decode one percent-encoded URL component string, returning null on malformed escapes. |
| `parse_json(expr)` | Strictly validate and normalize one JSON payload string. |
| `check_json(expr)` | Return whether one string expression contains valid JSON. |
| `schema_of_json(expr)` | Infer a deterministic schema description from one JSON payload string. |
| `json_array_length(expr)` | Return the number of array elements for a JSON array payload, or null for non-array payloads. |
| `json_object_keys(expr)` | Return object keys from a JSON object payload as a JSON array string. |
| `get_json_object(expr, path)` | Extract one JSON value at a literal path and return it as JSON text. |
| `json_extract_path_text(expr, path)` | Extract one JSON value at a literal path and return scalar strings as plain text. |
| `from_json(expr, schema)` | Validate JSON with an explicit schema description and return a normalized JSON payload string. |
| `try_from_json(expr, schema)` | Validate JSON with an explicit schema description and return null when the payload is invalid. |
| `to_json(expr)` | Serialize one scalar expression as JSON text. |
| `schema_of_csv(expr)` | Infer a deterministic schema description from one CSV row string. |
| `from_csv(expr, schema)` | Parse one CSV row string into a JSON payload string, using schema field names when provided. |
| `to_csv(expr)` | Serialize one scalar or JSON array/object payload as one CSV row string. |

```incan
from pub::inql.functions import col, from_json, get_json_object, parse_url, sha2, to_json

projected = (
    events
        .with_column("user_hash", sha2(col("user_id"), 256))
        .with_column("host", parse_url(col("landing_page"), "host"))
        .with_column("event_type", get_json_object(col("payload"), "$.type"))
        .with_column("payload_obj", from_json(col("payload"), "STRUCT<type: STRING>"))
        .with_column("payload_out", to_json(col("event_type")))
)
```

Hash helpers operate on UTF-8 string bytes and return lowercase hexadecimal strings. `sha2(...)` accepts `224`, `256`,
`384`, and `512`; unsupported digest lengths are rejected by the helper rather than being passed through to a backend.

JSON and CSV parser helpers are scalar payload helpers in the current InQL value model: they validate, normalize, and
project payload text. They do not read external files and do not introduce a dynamic variant value type. Names for
variant-style predicates such as `typeof(...)`, `is_array(...)`, `is_object(...)`, `is_integer(...)`, `is_timestamp(...)`,
and `is_null_value(...)` are reserved by RFC 022 and remain intentionally unavailable until InQL defines the variant value
model those predicates would inspect.
