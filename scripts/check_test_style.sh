#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_DIR="$ROOT_DIR/tests"

if [[ ! -d "$TEST_DIR" ]]; then
  echo "tests directory not found: $TEST_DIR" >&2
  exit 1
fi

status=0

while IFS= read -r -d '' file; do
  awk '
    BEGIN {
      in_test = 0
      missing = 0
      test_name = ""
      test_line = 0
      has_arrange = 0
      has_act = 0
      has_assert = 0
    }

    function finish_test() {
      if (in_test == 0) {
        return
      }
      if (has_arrange == 0 || has_act == 0 || has_assert == 0) {
        printf("%s:%d: test %s is missing style markers:", FILENAME, test_line, test_name)
        if (has_arrange == 0) {
          printf(" Arrange")
        }
        if (has_act == 0) {
          printf(" Act")
        }
        if (has_assert == 0) {
          printf(" Assert")
        }
        printf("\n")
        missing = 1
      }
      in_test = 0
      test_name = ""
      test_line = 0
      has_arrange = 0
      has_act = 0
      has_assert = 0
    }

    /^[[:space:]]*def[[:space:]]+test_[a-zA-Z0-9_]*\(/ {
      finish_test()
      in_test = 1
      test_line = NR
      test_name = $0
      sub(/^[[:space:]]*def[[:space:]]+/, "", test_name)
      sub(/\(.*/, "", test_name)
      next
    }

    /^[[:space:]]*def[[:space:]]+/ {
      finish_test()
      next
    }

    {
      if (in_test == 1 && $0 ~ /# --/) {
        if ($0 ~ /Arrange/) {
          has_arrange = 1
        }
        if ($0 ~ /Act/) {
          has_act = 1
        }
        if ($0 ~ /Assert/) {
          has_assert = 1
        }
      }
    }

    END {
      finish_test()
      if (missing == 1) {
        exit 2
      }
    }
  ' "$file" || status=1
done < <(find "$TEST_DIR" -maxdepth 1 -name '*.incn' -print0 | sort -z)

if [[ "$status" -ne 0 ]]; then
  echo "test style check failed: every test must include Arrange/Act/Assert section markers" >&2
  exit 1
fi

echo "✓ test style check passed"
