#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
INCAN_BIN="${INCAN:-incan}"

SMOKE_ROOT="$ROOT/target/smoke_pub_consumer"
PROJECT_DIR="$SMOKE_ROOT/project"

rm -rf "$PROJECT_DIR"
mkdir -p "$PROJECT_DIR"

"$INCAN_BIN" init "$PROJECT_DIR" --name inql_pub_consumer_smoke >/dev/null

cat > "$PROJECT_DIR/incan.toml" <<EOF
[project]
name = "inql_pub_consumer_smoke"
version = "0.1.0"

[dependencies]
inql = { path = "$ROOT" }

[project.scripts]
main = "src/main.incn"
EOF

cat > "$PROJECT_DIR/src/main.incn" <<'EOF'
from pub::inql import Session, SessionError, always_true

@derive(Clone)
model Order:
    id: int


def main() -> Result[None, SessionError]:
    mut session = Session.default()
    orders = session.read_csv[Order]("orders", "tests/fixtures/orders.csv")?
    transformed = orders.filter(always_true()).limit(1)
    session.collect(transformed)?
    return Ok(None)
EOF

(cd "$PROJECT_DIR" && "$INCAN_BIN" lock >/dev/null)
(cd "$PROJECT_DIR" && "$INCAN_BIN" --check src/main.incn >/dev/null)

echo "✓ pub consumer smoke check passed"
