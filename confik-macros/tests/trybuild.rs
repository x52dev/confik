// only run on MSRV to avoid changes to compiler output causing CI failures
#[rustversion::stable(1.66)]
#[test]
fn compile_macros() {
    let t = trybuild::TestCases::new();

    t.pass("tests/trybuild/01-parse.rs");
    t.pass("tests/trybuild/02-create-builder.rs");
    t.pass("tests/trybuild/03-simple-impl.rs");
    t.pass("tests/trybuild/04-simple-build.rs");
    t.pass("tests/trybuild/05-simple-build-with-enum.rs");
    t.pass("tests/trybuild/06-nested-struct.rs");
    t.pass("tests/trybuild/07-phantom-data.rs");
    t.pass("tests/trybuild/08-redefined-prelude-types.rs");
    t.pass("tests/trybuild/09-pub-target.rs");
    t.pass("tests/trybuild/10-unnamed_struct_fields.rs");
    t.pass("tests/trybuild/11-simple-secret-source.rs");
    t.pass("tests/trybuild/12-complex-secret-source.rs");
    t.pass("tests/trybuild/13-unnamed-secret-source.rs");
    t.pass("tests/trybuild/14-simple-default.rs");
    t.pass("tests/trybuild/15-default-default.rs");
    t.pass("tests/trybuild/16-partial-default.rs");
    t.pass("tests/trybuild/17-comments.rs");
    t.pass("tests/trybuild/18-secret-default.rs");
    t.pass("tests/trybuild/19-derive.rs");
    t.pass("tests/trybuild/20-enum.rs");
    t.pass("tests/trybuild/21-field-from.rs");
    t.pass("tests/trybuild/22-dataless-types.rs");
    t.pass("tests/trybuild/23-where-clause.rs");
    t.pass("tests/trybuild/24-field-try-from.rs");

    t.compile_fail("tests/trybuild/fail-default-parse.rs");
    t.compile_fail("tests/trybuild/fail-default-invalid-expr.rs");
    t.compile_fail("tests/trybuild/fail-config-name-value.rs");
    t.compile_fail("tests/trybuild/fail-secret-extra-attr.rs");
    t.compile_fail("tests/trybuild/fail-derive-literal.rs");
    t.compile_fail("tests/trybuild/fail-field-from-unknown-type.rs");
    t.compile_fail("tests/trybuild/fail-uncreatable-type.rs");
    t.compile_fail("tests/trybuild/fail-not-a-type.rs");
    t.compile_fail("tests/trybuild/fail-default-not-expression.rs");
    t.compile_fail("tests/trybuild/fail-from-and-try-from.rs");
    t.compile_fail("tests/trybuild/fail-try-from-not-implemented.rs");
}
