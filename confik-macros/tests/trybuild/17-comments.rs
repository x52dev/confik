//! Check that comments don't interfere with parsing

#[derive(confik::Configuration)]
struct _Config {
    /// Outer doc comments!
    // Outer non-doc comments!
    /** Out long doc comments */
    /* Outer long non-doc comments! */
    _param: String,
}

fn main() {}
