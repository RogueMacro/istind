use std::path::Path;

pub mod mac;

pub trait Executable {
    fn build(&self, out_path: impl AsRef<Path>);
}
