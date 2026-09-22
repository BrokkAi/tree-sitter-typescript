fn main() {
    let root_dir = std::path::Path::new(".");
    let typescript_dir = root_dir.join("typescript").join("src");
    let tsx_dir = root_dir.join("tsx").join("src");
    let common_dir = root_dir.join("common");

    let mut config = cc::Build::new();
    config.include(&typescript_dir);
    config
        .flag_if_supported("-std=c11")
        .flag_if_supported("-Wno-unused-parameter")
        .define("tree_sitter_typescript", "brokk_tree_sitter_typescript")
        .define("tree_sitter_tsx", "brokk_tree_sitter_tsx")
        .define(
            "tree_sitter_typescript_external_scanner_create",
            "brokk_tree_sitter_typescript_external_scanner_create",
        )
        .define(
            "tree_sitter_typescript_external_scanner_destroy",
            "brokk_tree_sitter_typescript_external_scanner_destroy",
        )
        .define(
            "tree_sitter_typescript_external_scanner_scan",
            "brokk_tree_sitter_typescript_external_scanner_scan",
        )
        .define(
            "tree_sitter_typescript_external_scanner_serialize",
            "brokk_tree_sitter_typescript_external_scanner_serialize",
        )
        .define(
            "tree_sitter_typescript_external_scanner_deserialize",
            "brokk_tree_sitter_typescript_external_scanner_deserialize",
        )
        .define(
            "tree_sitter_tsx_external_scanner_create",
            "brokk_tree_sitter_tsx_external_scanner_create",
        )
        .define(
            "tree_sitter_tsx_external_scanner_destroy",
            "brokk_tree_sitter_tsx_external_scanner_destroy",
        )
        .define(
            "tree_sitter_tsx_external_scanner_scan",
            "brokk_tree_sitter_tsx_external_scanner_scan",
        )
        .define(
            "tree_sitter_tsx_external_scanner_serialize",
            "brokk_tree_sitter_tsx_external_scanner_serialize",
        )
        .define(
            "tree_sitter_tsx_external_scanner_deserialize",
            "brokk_tree_sitter_tsx_external_scanner_deserialize",
        );

    for path in &[
        typescript_dir.join("parser.c"),
        typescript_dir.join("scanner.c"),
        tsx_dir.join("parser.c"),
        tsx_dir.join("scanner.c"),
    ] {
        config.file(path);
        println!("cargo:rerun-if-changed={}", path.to_str().unwrap());
    }

    println!(
        "cargo:rerun-if-changed={}",
        common_dir.join("scanner.h").to_str().unwrap()
    );

    config.compile("brokk-tree-sitter-typescript");
}
