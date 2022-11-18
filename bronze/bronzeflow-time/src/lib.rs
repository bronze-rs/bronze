#![feature(assert_matches)]
//! # Bronze-Time: a common internal time crate for bronze
// #![deny(missing_docs)]

pub mod prelude;
mod schedule_expr;
pub mod schedule_time;

#[cfg(test)]
mod tests {
    #[test]
    fn test_li() {}
}
