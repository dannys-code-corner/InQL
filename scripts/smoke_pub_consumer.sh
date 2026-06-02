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

cat > "$PROJECT_DIR/src/query_blocks_smoke.incn" <<EOF
"""Pub-consumer smoke for the InQL query-block vocabulary surface."""

import pub::inql

from pub::inql import DataFrame, LazyFrame, Session, array, col, eq, lit
from std.testing import assert_is_ok, fail_t


@derive(Clone)
pub model AggregateOrder:
    pub customer_id: str
    pub amount: int


@derive(Clone)
pub model SelectedOrder:
    pub customer: str
    pub adjusted: int
    pub doubled: int


@derive(Clone)
pub model CustomerRollup:
    pub customer: str
    pub total: int
    pub order_count: int


@derive(Clone)
pub model CustomerOnly:
    pub customer: str


@derive(Clone)
pub model CustomerAmount:
    pub customer: str
    pub amount: int


@derive(Clone)
pub model JoinedAmount:
    pub customer: str
    pub joined_amount: int


@derive(Clone)
pub model LeftMatchedAmount:
    pub customer: str
    pub matched_amount: int


@derive(Clone)
pub model ExplodedWindowOrder:
    pub customer: str
    pub amount: int
    pub tag: str
    pub row_num: int


const AGGREGATE_ORDERS_CSV_FIXTURE: str = "$ROOT/tests/fixtures/aggregate_orders.csv"


def _orders(mut session: Session, table_name: str) -> LazyFrame[AggregateOrder]:
    """Load the shared aggregate-order fixture."""
    return assert_is_ok(
        session.read_csv(table_name, AGGREGATE_ORDERS_CSV_FIXTURE),
        "aggregate orders fixture should load",
    )


def _collect_or_fail[T with Clone](mut session: Session, frame: LazyFrame[T]) -> DataFrame[T]:
    """Collect a query-block frame or fail with the backend diagnostic."""
    match session.collect(frame):
        Ok(df) => return df
        Err(err) => return fail_t(err.error_message())


def _preview_line_contains_all(line: str, expected_cells: list[str]) -> bool:
    """Return whether one rendered preview row contains every expected cell value."""
    for cell in expected_cells:
        if not line.contains(cell):
            return false
    return true


def _assert_preview_row_contains(payload: str, expected_cells: list[str], context: str) -> None:
    """Assert one rendered preview row carries the expected materialized cells together."""
    for line in payload.split("\n"):
        if _preview_line_contains_all(line, expected_cells):
            return
    return fail_t(context)


def _brace_select_aliases_and_lateral_aliases_materialize() -> None:
    # -- Arrange --
    mut session = Session.default()
    orders = _orders(session, "query_orders_select")

    # -- Act --
    selected: LazyFrame[SelectedOrder] = query {
        FROM orders
        SELECT
            .customer_id as customer,
            .amount + 5 as adjusted,
            adjusted * 2 as doubled,
    }
    df = _collect_or_fail(session, selected)
    payload = df.preview_text()

    # -- Assert --
    assert df.row_count() == 3, "query SELECT should preserve source row count"
    assert df.resolved_columns() == ["customer", "adjusted", "doubled"], "query SELECT should publish requested aliases"
    _assert_preview_row_contains(payload, ["A", "15", "30"], "SELECT should materialize computed aliases")
    _assert_preview_row_contains(payload, ["B", "12", "24"], "SELECT should materialize later rows")


def _grouped_aggregate_select_materializes_group_and_measures() -> None:
    # -- Arrange --
    mut session = Session.default()
    orders = _orders(session, "query_orders_grouped")

    # -- Act --
    grouped: LazyFrame[CustomerRollup] = query {
        FROM orders
        GROUP BY .customer_id
        SELECT
            .customer_id as customer,
            sum(.amount) as total,
            count() as order_count,
    }
    df = _collect_or_fail(session, grouped)
    payload = df.preview_text()

    # -- Assert --
    assert df.row_count() == 2, "query GROUP BY should produce one row per group"
    assert df.resolved_columns() == ["customer", "total", "order_count"], "query aggregate SELECT should preserve SELECT order and aliases"
    _assert_preview_row_contains(payload, ["A", "25", "2"], "customer A grouped values should materialize")
    _assert_preview_row_contains(payload, ["B", "7", "1"], "customer B grouped values should materialize")


def _select_distinct_materializes_unique_rows() -> None:
    # -- Arrange --
    mut session = Session.default()
    orders = _orders(session, "query_orders_distinct")

    # -- Act --
    distinct_customers: LazyFrame[CustomerOnly] = query {
        FROM orders
        SELECT DISTINCT
            .customer_id as customer,
    }
    df = _collect_or_fail(session, distinct_customers)
    payload = df.preview_text()

    # -- Assert --
    assert df.row_count() == 2, "SELECT DISTINCT should collapse duplicate projection rows"
    assert df.resolved_columns() == ["customer"], "SELECT DISTINCT should preserve selected output columns"
    _assert_preview_row_contains(payload, ["A"], "SELECT DISTINCT should include customer A")
    _assert_preview_row_contains(payload, ["B"], "SELECT DISTINCT should include customer B")


def _post_select_where_order_and_limit_materialize() -> None:
    # -- Arrange --
    mut session = Session.default()
    orders = _orders(session, "query_orders_order_limit")

    # -- Act --
    selected: LazyFrame[CustomerAmount] = query {
        FROM orders
        SELECT
            .customer_id as customer,
            .amount as amount,
        WHERE .amount > 10
        ORDER BY desc(.amount)
        LIMIT 1
    }
    df = _collect_or_fail(session, selected)
    payload = df.preview_text()

    # -- Assert --
    assert df.row_count() == 1, "post-SELECT WHERE plus LIMIT should leave one row"
    assert df.resolved_columns() == ["customer", "amount"], "post-SELECT filtering should keep projected columns"
    _assert_preview_row_contains(payload, ["A", "15"], "ORDER BY DESC should keep the highest filtered amount")


def _join_and_left_join_materialize() -> None:
    # -- Arrange --
    mut session = Session.default()
    orders = _orders(session, "query_orders_join_left")
    discounts = _orders(session, "query_orders_join_right")
    missing = _orders(session, "query_orders_left_missing").filter(eq(col("customer_id"), lit("Z")))

    # -- Act --
    joined: LazyFrame[JoinedAmount] = query {
        FROM orders
        JOIN discounts
        ON .customer_id == discounts.customer_id
        SELECT
            .customer_id as customer,
            discounts.amount as joined_amount,
        LIMIT 1
    }
    left_joined: LazyFrame[LeftMatchedAmount] = query {
        FROM orders
        LEFT JOIN missing
        ON .customer_id == missing.customer_id
        SELECT
            .customer_id as customer,
            missing.amount as matched_amount,
    }
    joined_df = _collect_or_fail(session, joined)
    left_df = _collect_or_fail(session, left_joined)

    # -- Assert --
    assert joined_df.row_count() == 1, "JOIN query block should materialize joined rows"
    assert joined_df.resolved_columns() == ["customer", "joined_amount"], "JOIN SELECT should publish requested aliases"
    assert left_df.row_count() == 3, "LEFT JOIN query block should preserve unmatched left rows"
    assert left_df.resolved_columns() == ["customer", "matched_amount"], "LEFT JOIN SELECT should publish requested aliases"


def _explode_and_window_by_materialize() -> None:
    # -- Arrange --
    mut session = Session.default()
    orders = _orders(session, "query_orders_explode_window")
    enriched = orders.with_column("tags", array([lit("paid"), col("customer_id")]))

    # -- Act --
    windowed: LazyFrame[ExplodedWindowOrder] = query {
        FROM enriched
        EXPLODE .tags as tag
        WINDOW BY row_num = row_number().over(window().partition_by([.customer_id]).order_by([desc(.amount)]))
        SELECT
            .customer_id as customer,
            .amount as amount,
            .tag as tag,
            .row_num as row_num,
        ORDER BY desc(.amount)
        LIMIT 2
    }
    df = _collect_or_fail(session, windowed)
    payload = df.preview_text()

    # -- Assert --
    assert df.row_count() == 2, "EXPLODE plus WINDOW BY should materialize generated rows"
    assert df.resolved_columns() == ["customer", "amount", "tag", "row_num"], "EXPLODE/WINDOW SELECT should publish requested aliases"
    _assert_preview_row_contains(payload, ["A", "paid", "1"], "WINDOW BY should rank generated rows")


def _colon_spelling_materializes_same_select_surface() -> None:
    # -- Arrange --
    mut session = Session.default()
    orders = _orders(session, "query_orders_colon_select")

    # -- Act --
    selected: LazyFrame[CustomerAmount] = query:
        FROM orders
        SELECT:
            .customer_id as customer
            .amount as amount
    df = _collect_or_fail(session, selected)
    payload = df.preview_text()

    # -- Assert --
    assert df.row_count() == 3, "query: SELECT should preserve source row count"
    assert df.resolved_columns() == ["customer", "amount"], "query: SELECT should publish requested aliases"
    _assert_preview_row_contains(payload, ["A", "10"], "query: SELECT should materialize the first row")
    _assert_preview_row_contains(payload, ["B", "7"], "query: SELECT should materialize later rows")


def main() -> None:
    println("query smoke: select")
    _brace_select_aliases_and_lateral_aliases_materialize()
    println("query smoke: aggregate")
    _grouped_aggregate_select_materializes_group_and_measures()
    println("query smoke: distinct")
    _select_distinct_materializes_unique_rows()
    println("query smoke: post-select")
    _post_select_where_order_and_limit_materialize()
    println("query smoke: joins")
    _join_and_left_join_materialize()
    println("query smoke: explode-window")
    _explode_and_window_by_materialize()
    println("query smoke: colon")
    _colon_spelling_materializes_same_select_surface()
EOF

(cd "$PROJECT_DIR" && "$INCAN_BIN" lock >/dev/null)
(cd "$PROJECT_DIR" && "$INCAN_BIN" --check src/main.incn >/dev/null)
(cd "$PROJECT_DIR" && "$INCAN_BIN" --check src/query_blocks_smoke.incn >/dev/null)
(cd "$PROJECT_DIR" && "$INCAN_BIN" run src/query_blocks_smoke.incn)

echo "✓ pub consumer smoke check passed"
