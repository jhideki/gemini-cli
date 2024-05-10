use std::error::Error;

#[derive(Debug)]
pub struct InvalidArgument;
impl std::fmt::Display for InvalidArgument {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid argument. Use -h to view valid commands.")
    }
}
impl Error for InvalidArgument {}
