use std::fmt::Formatter;

mod session;
mod link;
mod download;
mod errors;
mod resource;


pub trait OnProgressCallbackFunction<F> {}

impl std::fmt::Debug for dyn OnProgressCallbackFunction<u64> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "OnProgressCallbackFunction(bytes_written: u64)")
    }
}

