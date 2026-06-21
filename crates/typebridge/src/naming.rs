//! Naming as an isomorphism.
//!
//! Rust fields are idiomatic `snake_case`; JSON and TypeScript are idiomatic
//! `lowerCamelCase`. Treating the conversion as a (partial) isomorphism — rather
//! than two independent string manglers — is what keeps encode and decode in sync.
//!
//! The iso holds on the subset of *well-formed identifiers*: a `snake_case`
//! identifier with no leading/trailing/double underscores, and no underscore
//! immediately before a digit, round-trips exactly. The digit case is inherent,
//! not a bug: camelCase has no way to distinguish `col_1` from `col1`, so both map
//! to `col1` (this is exactly what serde's `rename_all = "camelCase"` does too).
//! Type names (`PascalCase`) are intentionally **not** transformed; they pass
//! through the renderers unchanged.

/// `snake_case` (or `snake_case_id`) to `lowerCamelCase`.
///
/// ```
/// use typebridge::naming::to_camel_case;
/// assert_eq!(to_camel_case("active_connection_id"), "activeConnectionId");
/// assert_eq!(to_camel_case("latency_ms"), "latencyMs");
/// assert_eq!(to_camel_case("id"), "id");
/// ```
pub fn to_camel_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut upper_next = false;
    let mut seen_first = false;
    for ch in input.chars() {
        if ch == '_' {
            // Leading underscores are dropped; interior ones uppercase the next char.
            if seen_first {
                upper_next = true;
            }
            continue;
        }
        if upper_next {
            out.extend(ch.to_uppercase());
            upper_next = false;
        } else {
            out.push(ch);
        }
        seen_first = true;
    }
    out
}

/// `lowerCamelCase` to `snake_case`.
///
/// ```
/// use typebridge::naming::to_snake_case;
/// assert_eq!(to_snake_case("activeConnectionId"), "active_connection_id");
/// assert_eq!(to_snake_case("latencyMs"), "latency_ms");
/// ```
pub fn to_snake_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 4);
    for (i, ch) in input.chars().enumerate() {
        if ch.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(ch.to_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camel_matches_irodori_fields() {
        assert_eq!(to_camel_case("latency_ms"), "latencyMs");
        assert_eq!(to_camel_case("active_connection_id"), "activeConnectionId");
        assert_eq!(to_camel_case("workspace_snapshot"), "workspaceSnapshot");
    }

    #[test]
    fn snake_round_trips_with_camel() {
        // The well-formed subset: single underscores, no digit-after-underscore.
        for id in [
            "id",
            "latency_ms",
            "active_connection_id",
            "rows",
            "a",
            "a_b_c_d",
            "server_version",
            "r2_d2",     // digits NOT adjacent to an underscore boundary are fine
            "über_wert", // non-ASCII lowercase passes through
        ] {
            assert_eq!(to_snake_case(&to_camel_case(id)), id, "iso failed for {id}");
        }
    }

    #[test]
    fn single_segment_is_identity_through_camel() {
        for id in ["id", "rows", "sql", "name"] {
            assert_eq!(to_camel_case(id), id);
        }
    }

    #[test]
    fn degenerate_inputs_do_not_panic() {
        assert_eq!(to_camel_case(""), "");
        assert_eq!(to_camel_case("_leading"), "leading");
        assert_eq!(to_camel_case("trailing_"), "trailing");
        assert_eq!(to_camel_case("double__under"), "doubleUnder");
        assert_eq!(to_camel_case("___"), "");
        assert_eq!(to_snake_case(""), "");
    }

    #[test]
    fn known_non_roundtrip_cases_are_pinned() {
        // These are INHERENT to camelCase, not bugs. camelCase cannot represent the
        // underscore before a digit, so the snake form is unrecoverable. Pin the
        // actual outputs so the behavior is intentional and stable.
        assert_eq!(to_camel_case("col_1"), "col1");
        assert_eq!(to_snake_case("col1"), "col1"); // not "col_1"
        assert_eq!(to_camel_case("user_id_2"), "userId2");
        assert_eq!(to_snake_case("userId2"), "user_id2"); // not "user_id_2"
    }

    #[test]
    fn snake_inserts_boundary_before_each_uppercase() {
        assert_eq!(to_snake_case("latencyMs"), "latency_ms");
        // Consecutive capitals each get a boundary (acronyms are not special-cased).
        assert_eq!(to_snake_case("HTTPServer"), "h_t_t_p_server");
        // Leading capital does not get a leading underscore.
        assert_eq!(to_snake_case("Pascal"), "pascal");
    }
}
