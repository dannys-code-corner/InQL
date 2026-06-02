//! InQL query-block vocabulary companion.
//!
//! The Incan compiler owns the generic vocabulary contract. InQL owns this package-specific `query:` surface and
//! lowers it into ordinary InQL helper/method calls that continue through Prism, Substrait, and the active backend.

mod desugar;

use incan_vocab::{
    ClauseSurface, DeclarationSurface, DslSurface, HelperBinding, LibraryManifest,
    ScopedSurfaceDescriptor, ScopedSurfaceDiagnosticKind, ScopedSurfaceDiagnosticTemplate,
    ScopedSurfaceEligibility, ScopedSurfaceMisuseScope, ScopedSurfaceReceiver, VocabRegistration,
};

pub use desugar::InqlQueryDesugarer;

pub const NAMESPACE: &str = "inql";
pub const QUERY_KW: &str = "query";
pub const QUERY_FIELD_DESCRIPTOR: &str = "inql.query.field";

const HELPER_EXPORTS: &[&str] = &[
    "col",
    "lit",
    "array",
    "add",
    "sub",
    "mul",
    "div",
    "modulo",
    "eq",
    "ne",
    "lt",
    "lte",
    "gt",
    "gte",
    "and_",
    "or_",
    "not_",
    "neg",
    "asc",
    "desc",
    "explode",
    "sum",
    "count",
    "avg",
    "min",
    "max",
    "aggregate_as",
    "with_column_assignment",
    "window",
    "row_number",
    "rank",
    "dense_rank",
    "percent_rank",
    "cume_dist",
    "ntile",
    "lag",
    "lead",
    "first_value",
    "last_value",
    "nth_value",
];

#[must_use]
pub fn library_vocab() -> VocabRegistration {
    VocabRegistration::new()
        .with_surface(
            DslSurface::on_import(NAMESPACE)
                .with_declaration(
                    DeclarationSurface::named(QUERY_KW)
                        .with_clause_body()
                        .desugars_to_expression()
                        .with_clauses([
                            ClauseSurface::expr("FROM").required(),
                            ClauseSurface::expr("JOIN").repeating().after("FROM"),
                            ClauseSurface::expr("LEFT JOIN").repeating().after("FROM"),
                            ClauseSurface::expr("ON").repeating().after("JOIN"),
                            ClauseSurface::expr("WHERE").repeating().after("FROM"),
                            ClauseSurface::expr_list("GROUP BY")
                                .optional()
                                .after("WHERE"),
                            ClauseSurface::expr_list("EXPLODE")
                                .repeating()
                                .after("FROM"),
                            ClauseSurface::fields("WINDOW BY")
                                .optional()
                                .after("GROUP BY"),
                            ClauseSurface::expr_list("SELECT").optional().after("FROM"),
                            ClauseSurface::expr_list("SELECT DISTINCT")
                                .optional()
                                .after("FROM"),
                            ClauseSurface::expr_list("ORDER BY")
                                .optional()
                                .after("SELECT"),
                            ClauseSurface::expr("LIMIT").optional().after("ORDER BY"),
                        ]),
                )
                .with_scoped_surface(
                    ScopedSurfaceDescriptor::leading_dot_path(QUERY_FIELD_DESCRIPTOR)
                        .with_eligibilities([
                            ScopedSurfaceEligibility::clause_body(QUERY_KW, "WHERE"),
                            ScopedSurfaceEligibility::clause_body(QUERY_KW, "GROUP"),
                            ScopedSurfaceEligibility::clause_body(QUERY_KW, "SELECT"),
                            ScopedSurfaceEligibility::clause_body(QUERY_KW, "ORDER"),
                            ScopedSurfaceEligibility::clause_body(QUERY_KW, "EXPLODE"),
                            ScopedSurfaceEligibility::clause_body(QUERY_KW, "WINDOW"),
                            ScopedSurfaceEligibility::clause_body(QUERY_KW, "ON"),
                        ])
                        .with_receiver(ScopedSurfaceReceiver::OwningDeclaration)
                        .with_misuse_scope(ScopedSurfaceMisuseScope::ActivatingFile)
                        .with_diagnostic(
                            ScopedSurfaceDiagnosticTemplate::new(
                                "inql-query-field-outside-scope",
                                ScopedSurfaceDiagnosticKind::OutsideScope,
                                "query field shorthand is only valid inside InQL query clauses",
                            )
                            .with_help(
                                "move the leading-dot field reference into a `query:` clause",
                            ),
                        ),
                ),
        )
        .with_library_manifest(LibraryManifest {
            helper_bindings: helper_bindings(),
            ..LibraryManifest::default()
        })
        .with_desugarer(InqlQueryDesugarer)
}

fn helper_bindings() -> Vec<HelperBinding> {
    HELPER_EXPORTS
        .iter()
        .map(|name| HelperBinding {
            key: (*name).to_string(),
            exported_name: (*name).to_string(),
        })
        .collect()
}

incan_vocab::export_wasm_desugarer!(InqlQueryDesugarer);
