//! Compile-fail UI tests pinning the derive macros' targeted parser
//! diagnostics. Regenerate the expected `.stderr` after an *intentional*
//! diagnostic change with `TRYBUILD=overwrite cargo test -p eventric-model
//! --test ui`.
#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
