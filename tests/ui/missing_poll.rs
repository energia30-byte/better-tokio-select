#![feature(stmt_expr_attributes)]
#![feature(proc_macro_hygiene)]
use better_tokio_select::tokio_select;

fn main() {
    #[tokio_select]
    match () {
        _ | _ => invalid,
    }
}
