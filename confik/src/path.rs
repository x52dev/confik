use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
};

#[derive(Debug, Default)]
pub(crate) struct Path(pub(crate) Vec<Cow<'static, str>>);

impl Path {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, segment) in self.0.iter().rev().enumerate() {
            if i > 0 {
                f.write_str(".")?;
            }
            f.write_str(segment)?;
        }
        Ok(())
    }
}
