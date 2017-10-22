#![feature(slice_patterns)]
#[macro_use]
extern crate nom;

use std::str::FromStr;
use std::env;

#[derive(Debug, PartialEq)]
enum Sexpr {
    Plus (Vec<Sexpr>),
    Minus (Vec<Sexpr>),
    Mul (Vec<Sexpr>),
    Int(i64)
}

impl Sexpr {
    fn interpret(&self) -> i64 {
        match self {
            &Sexpr::Plus(ref v) => v.iter().fold(0, |acc, sexpr| acc + sexpr.interpret()),
            &Sexpr::Minus(ref v) => match v.as_slice() {
                &[] => 0,
                &[ref sexpr] => -sexpr.interpret(),
                &[ref sexpr, ref rest..] => rest.iter().fold(sexpr.interpret(), |acc, sexpr| acc - sexpr.interpret())
            },
            &Sexpr::Mul(ref v) => v.iter().fold(1, |acc, sexpr| acc * sexpr.interpret()),
            &Sexpr::Int(n) => n
        }
    }
}

type MakeSexpr = &'static Fn(Vec<Sexpr>) -> Sexpr;

named!(open_bracket<&str,&str>,
    ws!(tag_s!("("))
);

named!(close_bracket<&str,&str>,
    ws!(tag_s!(")"))
);

named!(plus<&str, MakeSexpr>,
   map!(ws!(tag!("+")), |_| &Sexpr::Plus)
);

named!(minus<&str, MakeSexpr>,
   map!(ws!(tag!("-")), |_| &Sexpr::Minus)
);

named!(mul<&str, MakeSexpr>,
   map!(ws!(tag!("*")), |_| &Sexpr::Mul)
);

named!(operation<&str, MakeSexpr>,
   alt!(plus | minus | mul)
);

named!(integer<&str,Result<i64, &str> >,
    do_parse!(
        minus: opt!(ws!(tag_s!("-"))) >>
        value: ws!(map!(take_while_s!(|c: char| c.is_digit(10)), FromStr::from_str)) >>
        (value.map(|v:i64| minus.map_or(v, |_| {-v} )).map_err(|_| "error on convert integer"))
    )
);

fn collect_sexprs<'a>(acc: Result<Vec<Sexpr>, &'a str>, expr: Result<Sexpr, &'a str>) -> Result<Vec<Sexpr>, &'a str> {
    match (acc, expr) {
        (_, Err(e)) => Err(e),
        (acc, Ok(ex)) => acc.map(|mut v| {v.push(ex); v} )
    }
}

named!(sexpr_brackets<&str, Result<Sexpr, &str> >,
    do_parse!(
        open_bracket >>
        op: operation >>
        exprs: fold_many1!(parse_sexpr, Ok(Vec::new()), collect_sexprs) >>
        close_bracket >>
        (exprs.map(op))
    )
);

named!(parse_sexpr<&str,Result<Sexpr, &str> >,
  alt!(
    sexpr_brackets |
    map!(integer, |v| v.map(Sexpr::Int))
  )
);

fn main() {
    let  args : Vec<String> = env::args().collect();
    if  args.len() > 1 {
        let s = args[1..].join(" ");
        match parse_sexpr(&s) {
            nom::IResult::Done(_, Ok(sexpr)) => println!("{:?}\n{}", sexpr, sexpr.interpret()),
            e => println!("error {:?} on input {:?}", e, s)
        }
    } else {
        println!("sepxr expected");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(s: &str) -> i64 {
        parse_sexpr(s).unwrap().1.unwrap().interpret()
    }

    #[test]
    fn calc1() { assert_eq!(calc("(+ (* 4 4) (* 2 (- 7 5)) 1)"), 21); }
    #[test]
    fn calc2() { assert_eq!(calc("10"), 10); }
    #[test]
    fn calc3() { assert_eq!(calc("(* 10 (- 0 1))"), -10); }
    #[test]
    fn calc4() { assert_eq!(calc("(- (+ 10 10) -5 0)"), 25); }
    #[test]
    fn calc5() { assert_eq!(calc("(+ (- (* (+ (- (* 1))))))"), 1); }
    #[test]
    fn calc6() { assert_eq!(calc("(* 2 (+ (- 10 9) (- 3 (* 2 1))) (+ (- 10 9) (- 3 (* 2 1))))"), 8); }
    #[test]
    fn calc7() { assert_eq!(calc("(+ (* 2 1) (+ 8 8) (- (+ 4 3 2 1) (* 3 3) (* 2 2)) (* 5 7))"), 50); }
    #[test]
    fn calc8() { assert_eq!(calc("(- (+ (+ 3 3) (- 3 3) (+ 3 3) (- 3 3)) (* 2 2))"), 8); }
    #[test]
    fn calc9() { assert_eq!(calc("(+ (- 6 1) (+ 0 1 1) (- 7 2) (* 3 4 5) (- 3 1) (+ 2) (- 0 10))"), 66); }
}

