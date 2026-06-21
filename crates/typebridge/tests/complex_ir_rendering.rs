use typebridge::ir::{Decl, Field, TsType};

#[test]
fn nested_container_precedence_is_stable() {
    assert_eq!(
        TsType::array(TsType::nullable(TsType::string())).render(),
        "(string | null)[]"
    );

    assert_eq!(
        TsType::nullable(TsType::array(TsType::string())).render(),
        "string[] | null"
    );

    assert_eq!(
        TsType::array(TsType::union([TsType::string(), TsType::number()])).render(),
        "(string | number)[]"
    );

    assert_eq!(
        TsType::record(
            TsType::string(),
            TsType::array(TsType::nullable(TsType::union([
                TsType::string(),
                TsType::number(),
                TsType::boolean(),
            ]))),
        )
        .render(),
        "Record<string, ((string | number | boolean) | null)[]>"
    );
}

#[test]
fn inline_objects_with_optional_docs_and_unknowns_are_rendered() {
    let decl = Decl::interface(
        "SearchEnvelope",
        [
            Field::rust("request_id", TsType::string()).with_docs("Stable id for logs."),
            Field::rust(
                "filters",
                TsType::array(TsType::object([
                    Field::new("field", TsType::string()),
                    Field::new("op", TsType::string_literals(["eq", "neq", "in", "between"])),
                    Field::new(
                        "value",
                        TsType::union([
                            TsType::string(),
                            TsType::number(),
                            TsType::boolean(),
                            TsType::array(TsType::string()),
                        ]),
                    ),
                ])),
            )
            .optional(),
            Field::rust(
                "metadata",
                TsType::record(TsType::string(), TsType::nullable(TsType::unknown())),
            ),
        ],
    )
    .with_docs("A realistic search request envelope.");

    let ts = decl.render();

    assert!(ts.contains("/** A realistic search request envelope. */"), "{ts}");
    assert!(ts.contains("export interface SearchEnvelope {"), "{ts}");
    assert!(ts.contains("  /** Stable id for logs. */\n  requestId: string;"), "{ts}");
    assert!(ts.contains("  filters?: { field: string; op: \"eq\" | \"neq\" | \"in\" | \"between\"; value: string | number | boolean | string[]; }[];"), "{ts}");
    assert!(
        ts.contains("  metadata: Record<string, unknown | null>;"),
        "{ts}"
    );
}

#[test]
fn raw_declaration_is_preserved_as_a_backend_seam() {
    let raw = Decl::raw(
        "ExternalPluginMessage",
        "export type ExternalPluginMessage =\n  | { kind: \"ready\" }\n  | { kind: \"error\"; message: string };\n",
    );

    let ts = raw.render();

    assert!(ts.starts_with("export type ExternalPluginMessage ="), "{ts}");
    assert!(ts.contains("| { kind: \"ready\" }"), "{ts}");
    assert!(ts.ends_with(";\n"), "{ts:?}");
}

#[test]
fn empty_unions_stay_never_instead_of_any_or_unknown() {
    assert_eq!(TsType::union(Vec::<TsType>::new()).render(), "never");
    assert_eq!(TsType::string_literals(Vec::<String>::new()).render(), "never");

    let decl = Decl::alias("Impossible", TsType::union(Vec::<TsType>::new()));
    assert_eq!(decl.render(), "export type Impossible = never;\n");
}

#[test]
fn complex_domain_surface_does_not_collapse_to_unknown() {
    let command_payload = TsType::object([
        Field::new("sql", TsType::string()),
        Field::new("params", TsType::array(TsType::nullable(TsType::unknown()))),
        Field::new(
            "limits",
            TsType::object([
                Field::new("maxRows", TsType::number()),
                Field::new("timeoutMs", TsType::nullable(TsType::number())),
            ]),
        ),
    ]);

    let rendered = command_payload.render();

    assert!(rendered.contains("sql: string;"), "{rendered}");
    assert!(
        rendered.contains("params: (unknown | null)[];"),
        "{rendered}"
    );
    assert!(
        rendered.contains("limits: { maxRows: number; timeoutMs: number | null; };"),
        "{rendered}"
    );
    assert!(!rendered.contains("any"), "{rendered}");
}
