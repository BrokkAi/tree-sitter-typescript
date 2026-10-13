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
    fn parse_bytes(language: tree_sitter_language::LanguageFn, source: &[u8]) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&language.into()).unwrap();
        parser.parse(source, None).unwrap()
    }

    fn has_node_bytes(
        node: tree_sitter::Node<'_>,
        source: &[u8],
        kind: &str,
        expected: &[u8],
    ) -> bool {
        if node.kind() == kind && source.get(node.start_byte()..node.end_byte()) == Some(expected) {
            return true;
        }

        (0..node.child_count())
            .any(|index| has_node_bytes(node.child(index).unwrap(), source, kind, expected))
    }

    fn count_nodes(node: tree_sitter::Node<'_>, kind: &str) -> usize {
        usize::from(node.kind() == kind)
            + (0..node.child_count())
                .map(|index| count_nodes(node.child(index).unwrap(), kind))
                .sum::<usize>()
    }

    fn assert_complete_source(tree: &tree_sitter::Tree, source: &[u8]) {
        let root = tree.root_node();
        assert_eq!(root.start_byte(), 0);
        assert_eq!(root.end_byte(), source.len());
        assert_eq!(root.utf8_text(source).unwrap().as_bytes(), source);
    }

    fn assert_node_bytes(tree: &tree_sitter::Tree, source: &[u8], kind: &str, expected: &[u8]) {
        assert!(
            has_node_bytes(tree.root_node(), source, kind, expected),
            "missing {kind} node with exact bytes {expected:?}: {}",
            tree.root_node().to_sexp()
        );
    }

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
    fn test_import_type_compositions_in_both_dialects() {
        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&language.into()).unwrap();
            for (source, kind) in [
                (r#"type ItemList = import("./m").Thing[];"#, "array_type"),
                (
                    r#"type Member = import("./m").Container["member"];"#,
                    "lookup_type",
                ),
                (
                    r#"type Nested = import("./m").NS.Box<string>["items"][][];"#,
                    "array_type",
                ),
                (r#"type Module = import("./m")["Thing"];"#, "lookup_type"),
                (
                    r#"type Keys = keyof import("./m").Thing;"#,
                    "index_type_query",
                ),
                (
                    r#"type ReadonlyList = readonly import("./m").Thing[];"#,
                    "readonly_type",
                ),
                (
                    r#"type Union = import("./m").Thing | undefined;"#,
                    "union_type",
                ),
                (
                    r#"type Intersection = import("./m").Thing & { id: string };"#,
                    "intersection_type",
                ),
            ] {
                let tree = parser.parse(source, None).unwrap();
                assert!(
                    !tree.root_node().has_error(),
                    "{source}: {}",
                    tree.root_node().to_sexp()
                );
                let alias = tree.root_node().named_child(0).unwrap();
                assert_eq!(
                    alias.child_by_field_name("value").unwrap().kind(),
                    kind,
                    "{source}"
                );
            }
        }
    }

    #[test]
    fn test_tsx_literal_ampersands_and_entities() {
        fn entity_count(node: tree_sitter::Node) -> usize {
            let mut cursor = node.walk();
            usize::from(node.kind() == "html_character_reference")
                + node
                    .named_children(&mut cursor)
                    .map(entity_count)
                    .sum::<usize>()
        }

        let mut parser = tree_sitter::Parser::new();
        parser.set_language(&super::LANGUAGE_TSX.into()).unwrap();
        for (source, entities) in [
            (
                r#"const x = <p a="&" b='&' c="x&" d='x&'>Roles & Groups</p>;"#,
                0,
            ),
            (
                r#"const x = <p a="?a=1&b=2" b='?a=1&b=2'>&foo &; &#bad &</p>;"#,
                0,
            ),
            (
                r#"const x = <p a="&amp;&#38;&#x26;" b='&amp;&#38;&#x26;'>&amp;&#38;&#x26;</p>;"#,
                9,
            ),
            (
                r#"const x = <><p a="&" />&amp;&{value}<span>&</span></>;"#,
                1,
            ),
        ] {
            let tree = parser.parse(source, None).unwrap();
            assert!(
                !tree.root_node().has_error(),
                "{source}: {}",
                tree.root_node().to_sexp()
            );
            assert_eq!(entity_count(tree.root_node()), entities, "{source}");
        }
        for source in [
            "const x = <p a=\"&>text</p>;",
            "const x = <p a='&>text</p>;",
            "const x = <p>&;",
        ] {
            let tree = parser.parse(source, None).unwrap();
            assert!(
                tree.root_node().has_error(),
                "unexpectedly accepted: {source}"
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

    #[test]
    fn test_raw_nul_content_is_preserved_in_both_dialects() {
        // NUL in a quoted module specifier is still string content. This checks
        // syntax parsing only; it makes no claim about resolving that module.
        let cases: &[(&[u8], &str, &[u8])] = &[
            (
                b"const double_raw = \"before\0after\";\nconst after = 1;",
                "string",
                b"\"before\0after\"",
            ),
            (
                b"const single_raw = 'before\0after';\nconst after = 1;",
                "string",
                b"'before\0after'",
            ),
            (
                b"const double_escape = \"before\\0after\";\nconst after = 1;",
                "string",
                b"\"before\\0after\"",
            ),
            (
                b"const single_escape = 'before\\0after';\nconst after = 1;",
                "string",
                b"'before\\0after'",
            ),
            (
                b"const slash_nul = \"before\\\0after\";\nconst after = 1;",
                "escape_sequence",
                b"\\\0",
            ),
            (
                b"const adjacent = \"\0\0\";\nconst after = 1;",
                "string",
                b"\"\0\0\"",
            ),
            (
                b"const only_nul = '\0';\nconst after = 1;",
                "string",
                b"'\0'",
            ),
            (
                b"// before\0after\nconst after = 1;",
                "comment",
                b"// before\0after",
            ),
            (b"//\0\nconst after = 1;", "comment", b"//\0"),
            (
                b"const block = 1; /* before\0after */\nconst after = 1;",
                "comment",
                b"/* before\0after */",
            ),
            (b"/*\0*/\nconst after = 1;", "comment", b"/*\0*/"),
            (b"<!--\0-->\nconst after = 1;", "html_comment", b"<!--\0-->"),
            (
                b"const template = `before\0after`;\nconst after = 1;",
                "template_string",
                b"`before\0after`",
            ),
            (
                b"const only_nul_template = `\0`;\nconst after = 1;",
                "template_string",
                b"`\0`",
            ),
            (
                b"const template = `before\0${value}after`;\nconst after = 1;",
                "string_fragment",
                b"before\0",
            ),
            (
                b"type Pattern = `before\0${string}after`;\nconst after = 1;",
                "template_literal_type",
                b"`before\0${string}after`",
            ),
            (
                b"type Imported = import(\"pkg\0suffix\").Member;\nconst after = 1;",
                "string",
                b"\"pkg\0suffix\"",
            ),
        ];

        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            for (source, kind, raw_node) in cases {
                let tree = parse_bytes(language, source);
                assert!(
                    !tree.root_node().has_error(),
                    "unexpected parse error for {source:?}: {}",
                    tree.root_node().to_sexp()
                );
                assert_complete_source(&tree, source);
                assert_node_bytes(&tree, source, kind, raw_node);
                assert_node_bytes(&tree, source, "lexical_declaration", b"const after = 1;");

                if source.starts_with(b"const template = `before\0${") {
                    assert_eq!(count_nodes(tree.root_node(), "template_substitution"), 1);
                }
            }
        }

        let line_comment_at_eof = b"// before\0";
        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            let tree = parse_bytes(language, line_comment_at_eof);
            assert!(
                !tree.root_node().has_error(),
                "{}",
                tree.root_node().to_sexp()
            );
            assert_complete_source(&tree, line_comment_at_eof);
            assert_node_bytes(&tree, line_comment_at_eof, "comment", line_comment_at_eof);
        }
    }

    #[test]
    fn test_raw_nul_is_preserved_in_tsx_attributes_and_text() {
        let cases: &[(&[u8], &str, &[u8])] = &[
            (
                b"const view = <Widget title=\"before\0after\" />;\nconst after = 1;",
                "string",
                b"\"before\0after\"",
            ),
            (
                b"const view = <p>before\0after</p>;\nconst after = 1;",
                "jsx_text",
                b"before\0after",
            ),
        ];

        for (source, kind, raw_node) in cases {
            let tree = parse_bytes(super::LANGUAGE_TSX, source);
            assert!(
                !tree.root_node().has_error(),
                "unexpected TSX parse error for {source:?}: {}",
                tree.root_node().to_sexp()
            );
            assert_complete_source(&tree, source);
            assert_node_bytes(&tree, source, kind, raw_node);
            assert_node_bytes(&tree, source, "lexical_declaration", b"const after = 1;");
        }
    }

    #[test]
    fn test_raw_nul_in_executable_syntax_is_an_error_in_both_dialects() {
        let cases: &[&[u8]] = &[
            b"const na\0me = 1;\nconst after = 2;",
            b"const result = left\0+ right;\nconst after = 2;",
            b"const result = `raw\0${left\0+right}`;\nconst after = 2;",
            b"type Module = import\0(\"pkg\").Member;\nconst after = 2;",
        ];

        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            for source in cases {
                let tree = parse_bytes(language, source);
                assert!(
                    tree.root_node().has_error(),
                    "unexpectedly accepted executable NUL in {source:?}: {}",
                    tree.root_node().to_sexp()
                );
                assert_complete_source(&tree, source);
                assert_node_bytes(&tree, source, "lexical_declaration", b"const after = 2;");
            }
        }
    }

    #[test]
    fn test_unterminated_raw_nul_content_errors_at_eof() {
        let cases: &[&[u8]] = &[
            b"const broken = \"before\0after",
            b"const broken = 'before\0after",
            b"const broken = 1; /* before\0after",
            b"const broken = `before\0after",
        ];

        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            for source in cases {
                let tree = parse_bytes(language, source);
                assert!(
                    tree.root_node().has_error(),
                    "unterminated raw-NUL content unexpectedly parsed: {source:?}: {}",
                    tree.root_node().to_sexp()
                );
                assert_complete_source(&tree, source);
            }
        }
    }

    #[test]
    fn test_incremental_raw_nul_edits_match_fresh_parses_in_both_dialects() {
        let content_cases: &[(&[u8], &[u8], &[u8])] = &[
            (b"const text = \"A\";\nconst tail = 1;", b"\"\0\"", b"\"A\""),
            (b"/*A*/\nconst tail = 1;", b"/*\0*/", b"/*A*/"),
            (b"const text = `A${value}`;\nconst tail = 1;", b"\0", b"A"),
        ];

        for language in [super::LANGUAGE_TYPESCRIPT, super::LANGUAGE_TSX] {
            for (source, nul_node, original_node) in content_cases {
                let mut parser = tree_sitter::Parser::new();
                parser.set_language(&language.into()).unwrap();
                let offset = source.iter().position(|byte| *byte == b'A').unwrap();
                let old_tree = parser.parse(*source, None).unwrap();
                assert!(!old_tree.root_node().has_error());

                let mut with_nul = source.to_vec();
                with_nul[offset] = 0;
                let incremental =
                    parse_after_single_byte_edit(&mut parser, &old_tree, &with_nul, offset);
                let fresh = parser.parse(with_nul.as_slice(), None).unwrap();
                assert!(!incremental.root_node().has_error());
                assert_complete_source(&incremental, &with_nul);
                assert_incremental_tree_matches_fresh(&incremental, &fresh, &with_nul);
                let node_kind = if source.starts_with(b"/*") {
                    "comment"
                } else if source.starts_with(b"const text = `") {
                    "string_fragment"
                } else {
                    "string"
                };
                assert_node_bytes(&incremental, &with_nul, node_kind, nul_node);

                let mut restored = with_nul.clone();
                restored[offset] = b'A';
                let incremental =
                    parse_after_single_byte_edit(&mut parser, &incremental, &restored, offset);
                let fresh = parser.parse(restored.as_slice(), None).unwrap();
                assert!(!incremental.root_node().has_error());
                assert_complete_source(&incremental, &restored);
                assert_incremental_tree_matches_fresh(&incremental, &fresh, &restored);
                assert_node_bytes(&incremental, &restored, node_kind, original_node);
            }

            let identifier_source = b"const name = 1;\nconst tail = 1;";
            let mut parser = tree_sitter::Parser::new();
            parser.set_language(&language.into()).unwrap();
            let offset = identifier_source
                .iter()
                .position(|byte| *byte == b'm')
                .unwrap();
            let old_tree = parser.parse(identifier_source.as_slice(), None).unwrap();
            assert!(!old_tree.root_node().has_error());
            let mut invalid_identifier = identifier_source.to_vec();
            invalid_identifier[offset] = 0;
            let incremental =
                parse_after_single_byte_edit(&mut parser, &old_tree, &invalid_identifier, offset);
            let fresh = parser.parse(invalid_identifier.as_slice(), None).unwrap();
            assert!(incremental.root_node().has_error());
            assert_complete_source(&incremental, &invalid_identifier);
            assert_incremental_tree_matches_fresh(&incremental, &fresh, &invalid_identifier);
            assert_node_bytes(
                &incremental,
                &invalid_identifier,
                "lexical_declaration",
                b"const tail = 1;",
            );
        }
    }

    fn parse_after_single_byte_edit(
        parser: &mut tree_sitter::Parser,
        old_tree: &tree_sitter::Tree,
        new_source: &[u8],
        offset: usize,
    ) -> tree_sitter::Tree {
        let mut edited_tree = old_tree.clone();
        edited_tree.edit(&tree_sitter::InputEdit {
            start_byte: offset,
            old_end_byte: offset + 1,
            new_end_byte: offset + 1,
            start_position: tree_sitter::Point {
                row: 0,
                column: offset,
            },
            old_end_position: tree_sitter::Point {
                row: 0,
                column: offset + 1,
            },
            new_end_position: tree_sitter::Point {
                row: 0,
                column: offset + 1,
            },
        });
        parser.parse(new_source, Some(&edited_tree)).unwrap()
    }

    fn assert_incremental_tree_matches_fresh(
        incremental: &tree_sitter::Tree,
        fresh: &tree_sitter::Tree,
        source: &[u8],
    ) {
        let incremental_root = incremental.root_node();
        let fresh_root = fresh.root_node();
        assert_eq!(incremental_root.to_sexp(), fresh_root.to_sexp());
        assert_eq!(incremental_root.start_byte(), fresh_root.start_byte());
        assert_eq!(incremental_root.end_byte(), fresh_root.end_byte());
        assert_eq!(incremental_root.has_error(), fresh_root.has_error());
        assert_eq!(
            incremental_root.utf8_text(source).unwrap(),
            fresh_root.utf8_text(source).unwrap()
        );
    }
}
