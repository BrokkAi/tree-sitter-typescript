//! This crate provides TypeScript and TSX language support for the [tree-sitter][] parsing library.
//!
//! Typically, you will use the [LANGUAGE_TYPESCRIPT] constant to add this language to a
//! tree-sitter [Parser][], and then use the parser to parse some code:
//!
//! ```
//! use tree_sitter::Parser;
//!
//! let code = r#"
//! function double(x: number): number {
//!     return x * 2;
//! }
//! "#;
//! let mut parser = Parser::new();
//! let language = brokk_tree_sitter_typescript::LANGUAGE_TYPESCRIPT;
//! parser
//!     .set_language(&language.into())
//!     .expect("Error loading TypeScript parser");
//! let tree = parser.parse(code, None).unwrap();
//! assert!(!tree.root_node().has_error());
//! ```
//!
//! [Parser]: https://docs.rs/tree-sitter/*/tree_sitter/struct.Parser.html
//! [tree-sitter]: https://tree-sitter.github.io/

use tree_sitter_language::LanguageFn;

extern "C" {
    fn brokk_tree_sitter_typescript() -> *const ();
    fn brokk_tree_sitter_tsx() -> *const ();
}

/// The tree-sitter [`LanguageFn`] for TypeScript.
///
/// [LanguageFn]: https://docs.rs/tree-sitter-language/*/tree_sitter_language/struct.LanguageFn.html
pub const LANGUAGE_TYPESCRIPT: LanguageFn =
    unsafe { LanguageFn::from_raw(brokk_tree_sitter_typescript) };

/// The tree-sitter [`LanguageFn`] for TSX.
///
/// [LanguageFn]: https://docs.rs/tree-sitter-language/*/tree_sitter_language/struct.LanguageFn.html
pub const LANGUAGE_TSX: LanguageFn = unsafe { LanguageFn::from_raw(brokk_tree_sitter_tsx) };

/// The content of the [`node-types.json`][] file for TypeScript.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers#static-node-types
pub const TYPESCRIPT_NODE_TYPES: &str = include_str!("../../typescript/src/node-types.json");

/// The content of the [`node-types.json`][] file for TSX.
///
/// [`node-types.json`]: https://tree-sitter.github.io/tree-sitter/using-parsers#static-node-types
pub const TSX_NODE_TYPES: &str = include_str!("../../tsx/src/node-types.json");

/// The syntax highlighting query for TypeScript.
pub const HIGHLIGHTS_QUERY: &str = include_str!("../../queries/highlights.scm");

/// The local-variable syntax highlighting query for TypeScript.
pub const LOCALS_QUERY: &str = include_str!("../../queries/locals.scm");

/// The symbol tagging query for TypeScript.
pub const TAGS_QUERY: &str = include_str!("../../queries/tags.scm");

#[cfg(test)]
mod tests {
    #[test]
    fn test_can_load_typescript_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE_TYPESCRIPT.into())
            .expect("Error loading TypeScript parser");
    }

    #[test]
    fn test_can_load_tsx_grammar() {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&super::LANGUAGE_TSX.into())
            .expect("Error loading TSX parser");
    }

    #[test]
    fn test_can_coexist_with_upstream_grammars() {
        let mut parser = tree_sitter::Parser::new();
        for language in [
            super::LANGUAGE_TYPESCRIPT,
            super::LANGUAGE_TSX,
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT,
            tree_sitter_typescript::LANGUAGE_TSX,
        ] {
            parser
                .set_language(&language.into())
                .expect("grammar must coexist with its upstream counterpart");
            let tree = parser.parse("const pattern = /value+/;", None).unwrap();
            assert!(!tree.root_node().has_error());
        }
        assert_ne!(
            tree_sitter::Language::from(super::LANGUAGE_TYPESCRIPT),
            tree_sitter::Language::from(tree_sitter_typescript::LANGUAGE_TYPESCRIPT)
        );
        assert_ne!(
            tree_sitter::Language::from(super::LANGUAGE_TSX),
            tree_sitter::Language::from(tree_sitter_typescript::LANGUAGE_TSX)
        );
    }

    #[test]
    fn test_declaration_forms_in_both_dialects() {
        let source = r#"
type Ref<T> = import("widgets").Refs.Element<T>;
interface Options {
  previous?: Ref<HTMLElement>
  // Reserved words can name type members after a newline.
  in?: string[]
  instanceof: boolean
  abstract?: string
  readonly?: boolean
}
declare abstract class Base {
  abstract readonly kind: string;
}
declare abstract class Derived extends Base {
  public abstract override readonly kind: "number" | "bigint";
  tail: string;
}
"#;
        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&language.into()).unwrap();
            let tree = parser.parse(source, None).unwrap();
            assert!(
                !tree.root_node().has_error(),
                "{}",
                tree.root_node().to_sexp()
            );
        }
    }

    #[test]
    fn test_expression_and_malformed_near_misses() {
        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&language.into()).unwrap();
            for (source, operator) in [
                ("const found = key\n in record;", "in"),
                ("const found = item\n instanceof Widget;", "instanceof"),
            ] {
                let tree = parser.parse(source, None).unwrap();
                assert!(!tree.root_node().has_error());
                let value = tree
                    .root_node()
                    .named_child(0)
                    .unwrap()
                    .named_child(0)
                    .unwrap()
                    .child_by_field_name("value")
                    .unwrap();
                assert_eq!(value.kind(), "binary_expression");
                assert_eq!(
                    value
                        .child_by_field_name("operator")
                        .unwrap()
                        .utf8_text(source.as_bytes())
                        .unwrap(),
                    operator
                );
            }
            for source in [
                "type Broken = { previous: string in?: string[] };",
                "type Broken = import(\"widgets\").Ref<;",
                "abstract class Broken { abstract override readonly value: ; }",
            ] {
                let tree = parser.parse(source, None).unwrap();
                assert!(
                    tree.root_node().has_error(),
                    "unexpectedly accepted: {source}"
                );
            }
        }
    }
}
