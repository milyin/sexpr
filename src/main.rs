#![feature(slice_patterns)]
#[macro_use]
extern crate nom;

use std::str::FromStr;
use std::env;

#[derive(Debug, Clone, Copy)]
enum OpType{ Plus, Minus, Mul }

impl OpType {
    fn cost(&self) -> usize {
        match self {
            &OpType::Plus => 2,
            &OpType::Minus => 3,
            &OpType::Mul => 10
        }
    }
}

#[derive(Debug)]
enum Sexpr {
    Op { op_type: OpType, sexprs: Vec<Sexpr>, depth_cost: usize },
    Int(i64)
}

impl Sexpr {
    fn interpret(&self) -> i64 {
        match self {
            &Sexpr::Op { op_type: op, sexprs: ref v, .. }  => match op {
                OpType::Plus => v.iter().fold(0, |acc, sexpr| acc + sexpr.interpret()),
                OpType::Minus => match v.as_slice() {
                        &[] => 0,
                        &[ref sexpr] => -sexpr.interpret(),
                        &[ref sexpr, ref rest..] => rest.iter().fold(sexpr.interpret(), |acc, sexpr| acc - sexpr.interpret())
                    },
                OpType::Mul => v.iter().fold(1, |acc, sexpr| acc * sexpr.interpret()),
            },
            &Sexpr::Int(n) => n
        }
    }

    fn exprs(&self) -> &[Sexpr] {
        match self {
            &Sexpr::Op  { sexprs : ref v, .. } => v.as_slice(),
            &Sexpr::Int(_) => &[]
        }
    }

    fn op_cost(&self) -> usize {
        match self {
            &Sexpr::Op { op_type: op, .. } => op.cost(),
            &Sexpr::Int(_) => 0
        }
    }

    fn network_cost(&self) -> usize {
        return self.op_cost() + self.exprs().iter()
            .fold(0, |acc, sexpr| std::cmp::max(acc,sexpr.network_cost()))
    }

    fn is_leaf_op(&self) -> bool {
        self.exprs().iter().fold(true, |acc, sexpr| acc && sexpr.exprs().is_empty())
    }

    fn cpu_cost(&self) -> usize {
        return self.op_cost() + self.exprs().iter()
            .fold(0, |acc, sexpr| acc+sexpr.cpu_cost())
    }

    fn update_depth_cost(&mut self, cost: usize) {
        if let &mut Sexpr::Op { op_type: op, sexprs: ref mut e, depth_cost: ref mut dc }  = self {
            *dc = cost;
            for sexpr in e {
                sexpr.update_depth_cost(cost + op.cost() )
            }
        }
    }
}

//fn interpret_cpu(sexpr: &Sexpr, ncpus: usize) -> (i64, usize) {
//    let cpus = Vec::<Sexpr>::new();
//}

named!(open_bracket<&str,&str>,
    ws!(tag_s!("("))
);

named!(close_bracket<&str,&str>,
    ws!(tag_s!(")"))
);

named!(operation<&str, OpType>,
   alt!(
       map!(ws!(tag!("+")), |_| OpType::Plus) |
       map!(ws!(tag!("-")), |_| OpType::Minus) |
       map!(ws!(tag!("*")), |_| OpType::Mul)
   )
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
        (exprs.map(|v| Sexpr::Op { op_type: op, sexprs: v, depth_cost: 0 } ))
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
            nom::IResult::Done(_, Ok(ref mut root)) => {
                root.update_depth_cost(0);
                println!("{:?}\n{}\n{}",
                         root,
                         root.interpret(),
                         root.network_cost());
            }
            e => println!("error {:?} on input {:?}", e, s)
        }
    } else {
        println!("sepxr expected");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn calc(s: &str) -> (i64, usize) {
        let sexpr = parse_sexpr(s).unwrap().1.unwrap();
        (sexpr.interpret(), sexpr.network_cost())
    }

    #[test]
    fn calc1() { assert_eq!(calc("(+ (* 4 4) (* 2 (- 7 5)) 1)"), (21,15)); }
    #[test]
    fn calc2() { assert_eq!(calc("10"), (10,0)); }
    #[test]
    fn calc3() { assert_eq!(calc("(* 10 (- 0 1))"), (-10,13)); }
    #[test]
    fn calc4() { assert_eq!(calc("(- (+ 10 10) -5 0)"), (25,5)); }
    #[test]
    fn calc5() { assert_eq!(calc("(+ (- (* (+ (- (* 1))))))"), (1,30)); }
    #[test]
    fn calc6() { assert_eq!(calc("(* 2 (+ (- 10 9) (- 3 (* 2 1))) (+ (- 10 9) (- 3 (* 2 1))))"), (8,25)); }
    #[test]
    fn calc7() { assert_eq!(calc("(+ (* 2 1) (+ 8 8) (- (+ 4 3 2 1) (* 3 3) (* 2 2)) (* 5 7))"), (50,15)); }
    #[test]
    fn calc8() { assert_eq!(calc("(- (+ (+ 3 3) (- 3 3) (+ 3 3) (- 3 3)) (* 2 2))"), (8,13)); }
    #[test]
    fn calc9() { assert_eq!(calc("(+ (- 6 1) (+ 0 1 1) (- 7 2) (* 3 4 5) (- 3 1) (+ 2) (- 0 10))"), (66,12)); }
}

