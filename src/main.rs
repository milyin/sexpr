#![feature(slice_patterns)]
#[macro_use]
extern crate nom;

use std::str::FromStr;
use std::env;
use std::cell::Cell;
use std::fmt;


#[derive(Debug, Clone, Copy)]
enum OpType{ Plus, Minus, Mul, Int(i64) }

impl OpType {
    fn cost(&self) -> usize {
        match self {
            &OpType::Plus => 2,
            &OpType::Minus => 3,
            &OpType::Mul => 10,
            _ => 0
        }
    }
}

#[derive(Debug, Copy, Clone)]
struct Cpu {
    cpu: usize,
    start: usize,
    end: usize
}

#[derive(Debug)]
struct Sexpr {
    op: OpType,
    sexprs: Vec<Sexpr>,
    depth_cost: usize,
    cpu: Cell<Option<Cpu>>
}

impl fmt::Display for Sexpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let list = self.sexprs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join(" ");
        match self.op {
            OpType::Plus => write!(f, "(+ {} )", list),
            OpType::Minus => write!(f, "(- {} )", list),
            OpType::Mul => write!(f, "(* {} )", list),
            OpType::Int(n) => write!(f, "{}", n),
        }
    }
}

impl Sexpr {
    fn interpret(&self) -> i64 {
        match self.op {
            OpType::Plus => self.sexprs.iter().fold(0, |acc, sexpr| acc + sexpr.interpret()),
            OpType::Minus => match self.sexprs.as_slice() {
                &[] => 0,
                &[ref sexpr] => -sexpr.interpret(),
                &[ref sexpr, ref rest..] => rest.iter().fold(sexpr.interpret(), |acc, sexpr| acc - sexpr.interpret())
            },
            OpType::Mul => self.sexprs.iter().fold(1, |acc, sexpr| acc * sexpr.interpret()),
            OpType::Int(n) => n
        }
    }

    fn network_cost(&self) -> usize {
        return self.op.cost() + self.sexprs.iter()
            .fold(0, |acc, sexpr| std::cmp::max(acc,sexpr.network_cost()))
    }

    fn update_depth_cost(&mut self, cost: usize) {
        self.depth_cost = cost;
        for sexpr in &mut self.sexprs {
            sexpr.update_depth_cost(cost + self.op.cost() )
        }
    }

    // Returns Some(time) if all subexprs are scheduled and it's known
    // when their result is available
    fn get_available_at(&self) -> Option<usize> {
        let mut available_at : usize = 0;
        for expr in &self.sexprs {
            if expr.op.cost() > 0 {
                if let Some(cpu) = expr.cpu.get() {
                    available_at = std::cmp::max(available_at, cpu.end);
                } else {
                    return None
                }
            }
        }
        Some(available_at)
    }

    fn find_deepest_pending_subexpr(&self, min_depth_cost: usize, cpu_available_at: usize) -> Option<&Sexpr> {
        if self.cpu.get().is_none() && self.op.cost() > 0 && self.depth_cost > min_depth_cost {
            let mut best_depth_cost = self.depth_cost;
            let mut best = None;
            for expr in &self.sexprs {
                if let Some(e) = expr.find_deepest_pending_subexpr(best_depth_cost, cpu_available_at) {
                    if let Some(data_available_at) = e.get_available_at() {
                        if cpu_available_at >= data_available_at {
                            best_depth_cost = e.depth_cost;
                            best = Some(e);
                        }
                    }
                }
            }
            if best.is_some() { return best }
            if let Some(data_available_at) = self.get_available_at() {
                if cpu_available_at >= data_available_at {
                    return Some(self)
                }
            }
            None
        } else { None }
    }

}

fn schedule_next_chunk(root: &Sexpr, cpus: &mut Vec<usize>) -> bool {
    let mut ordered_cpus = cpus.iter_mut().enumerate().collect::<Vec<_>>();
    ordered_cpus.sort_by(|a, b| a.1.cmp(&b.1) );
    for (cpu, available_at) in ordered_cpus {
        if let Some(ref e) = root.find_deepest_pending_subexpr(0, *available_at) {
            let cost = e.op.cost();
            e.cpu.set(Some(Cpu {cpu, start: *available_at, end: *available_at + cost} ));
            println!("{} on cpu {} start {} end {} takes {}", e, cpu, *available_at, *available_at+cost, cost);
            *available_at += cost;
            return true
        }
    }
    false
}

fn schedule_to_cpus(root: &Sexpr, ncpus: usize) {
    let mut cpus = Vec::<usize>::new();
    cpus.resize(ncpus, 0);
    while schedule_next_chunk(root, &mut cpus) {}
    println!("cpu load {:?}\nExecution time on {} cpus is {} s", cpus, ncpus, cpus.iter().max().unwrap());
}

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
        (exprs.map(|v| Sexpr { op: op, sexprs: v, depth_cost: 0, cpu: Cell::new(None) } ))
    )
);

named!(parse_sexpr<&str,Result<Sexpr, &str> >,
  alt!(
    sexpr_brackets |
    map!(integer, |v| v.map( |n| Sexpr { op: OpType::Int(n), sexprs: Vec::new() , depth_cost: 0, cpu: Cell::new(None) } ))
  )
);

fn run_sample(cpus: usize, s: &str) {
     match parse_sexpr(&s) {
        nom::IResult::Done(_, Ok(ref mut root)) => {
            root.update_depth_cost(1);
            println!("Expression {}\nResult {}\nNetwork execution time {}",
                     root,
                     root.interpret(),
                     root.network_cost());
            schedule_to_cpus(root, cpus);
            println!();
        }
        e => println!("error {:?} on input {:?}", e, s)
    }
}

fn main() {

    let samples = vec! [
        (2, "(+ (* 4 4) (* 2 (- 7 5)) 1)"),
        (2, "10"),
        (2, "(* 10 (- 0 1))"),
        (2, "(- (+ 10 10) -5 0)"),
        (2, "(+ (- (* (+ (- (* 1))))))"),
        (2, "(* 2 (+ (- 10 9) (- 3 (* 2 1))) (+ (- 10 9) (- 3 (* 2 1))))"),
        (3, "(* 2 (+ (- 10 9) (- 3 (* 2 1))) (+ (- 10 9) (- 3 (* 2 1))))"),
        (2, "(+ (* 2 1) (+ 8 8) (- (+ 4 3 2 1) (* 3 3) (* 2 2)) (* 5 7))"),
        (3, "(+ (* 2 1) (+ 8 8) (- (+ 4 3 2 1) (* 3 3) (* 2 2)) (* 5 7))"),
        (4, "(+ (* 2 1) (+ 8 8) (- (+ 4 3 2 1) (* 3 3) (* 2 2)) (* 5 7))"),
        (2, "(- (+ (+ 3 3) (- 3 3) (+ 3 3) (- 3 3)) (* 2 2))"),
        (3, "(- (+ (+ 3 3) (- 3 3) (+ 3 3) (- 3 3)) (* 2 2))"),
        (2, "(+ (- 6 1) (+ 0 1 1) (- 7 2) (* 3 4 5) (- 3 1) (+ 2) (- 0 10))"),
    ];

    let  args : Vec<String> = env::args().collect();
    if  args.len() > 2 {
        let cpus = args[1].parse::<usize>().unwrap();
        let s = args[2..].join(" ");
        run_sample(cpus,&s);
    } else {
        for (cpus,s) in samples {
            run_sample(cpus,s)
        }
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

