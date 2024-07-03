
use lalrpop_util::lalrpop_mod;

lalrpop_mod!(pub grammar);
pub mod ast;


fn print(s: &str) {
    println!("Parsing: {}", s);
    
    let parser = grammar::TypeParser::new();
    let type_expr = parser.parse(s).unwrap();
    println!("Parsed: {:?}", type_expr);

    let context = vec![];
    let type_check_result = type_expr.type_check(&context);
    println!("Type checked: {:?}", type_check_result);
}



fn main() {
    // print("forall {kek} {star:1, star:1}");
    // print("star:11");
    // print("var{Nat, 0}");
    // print("lambda {Nat} {star:0, var{Nat, 0}}");
    // print("lambda {Nat} {star:0, lambda {Succ} {arrow var{Nat, 0} -> var{Nat, 0}, lambda {Zero} {var{Nat, 0}, var{Zero, 0}}}}");
    // print("λ(Nat : *) → Zero");
    // print("λ(Nat : *) → λ(Succ : Nat → Nat) → λ(Zero : Nat) → Zero");
    print("lambda {Nat} {star:0, lambda {Succ} {arrow var{Nat, 0} -> var{Nat, 0}, lambda {Zero} {var{Nat, 0}, var{Zero, 0}}}}");
}