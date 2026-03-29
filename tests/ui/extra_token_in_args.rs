#![feature(proc_macro_hygiene)]
use better_tokio_select::tokio_select;

fn main() {
    #[tokio_select(biased !)]
    match () {};
}
