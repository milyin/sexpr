#[macro_use]
extern crate nom;

#[derive(Debug, PartialEq)]
struct Sexpr {

}

fn parse_sexpr(sexpr: &str) -> Result<Sexpr,&str> {
   Err("unmatched quotes")
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn parse_test1() { assert_eq!(parse_sexpr("((())"), Err("unmatched quotes"))}
}

