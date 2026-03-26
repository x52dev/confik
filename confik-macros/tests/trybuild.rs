#[test]
fn macro_pass() {
    let t = trybuild::TestCases::new();
    t.pass("tests/trybuild/compile-pass/*.rs");
}

// only run on MSRV to avoid changes to compiler output causing CI failures
#[rustversion_msrv::msrv]
#[test]
fn macro_fail() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/trybuild/compile-fail/*.rs");
}
