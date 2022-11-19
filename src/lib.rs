use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    multi::many0,
    sequence::delimited,
    error::ParseError,
    character::complete::{multispace0},
};

use std::{error::Error, collections::HashMap, cmp::Ordering};

#[derive(Clone,Debug,PartialEq)]
pub enum Atom<'a> {
    Symbol(&'a str),
    String(&'a str),
    Number(isize),
}

#[derive(Clone,Debug)]
pub enum Elem<'a> {
    Atom(Atom<'a>),
    Single(Atom<'a>),
    Call(Vec<Elem<'a>>),
    List(Vec<Elem<'a>>),
}

#[derive(Debug)]
pub enum EvalError {
    Unreachable
}

impl Error for EvalError {
}

impl<'a> std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Eval Error...")
    }
}

impl<'a> std::fmt::Display for Elem<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Elem::Atom(name) => write!(f,"{}",name),
            Elem::Call(items) => {
                write!(f,"(")?;
                let mut first = true;
                for item in items {
                    if first {
                        first = false;
                    } else {
                        write!(f," ")?;
                    }
                    write!(f,"{}",item)?;
                }
                write!(f,")")
            },
            Elem::List(items) => {
                write!(f,"[")?;
                let mut first = true;
                for item in items {
                    if first {
                        first = false;
                    } else {
                        write!(f," ")?;
                    }
                    write!(f,"{}",item)?;
                }
                write!(f,"]")
            },
            Elem::Single(atom) => {
                write!(f,"#")?;
                write!(f,"{}",atom)
            }
        }
    }
}

impl<'a> std::fmt::Display for Atom<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self {
            Atom::Number(value) => write!(f,"{}",value),
            Atom::String(value) => write!(f,"\"{}\"",value),
            Atom::Symbol(name) => write!(f,"{}",name),
        }
    }
}

fn ws<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E> where
    F: Fn(&'a str) -> IResult<&'a str, O, E>
{
    delimited(multispace0, inner, multispace0)
}

fn dq<'a, F: 'a, O, E: ParseError<&'a str>>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O, E> where
    F: Fn(&'a str) -> IResult<&'a str, O, E>
{
    delimited(tag("\""), inner, tag("\""))
}

fn is_atom(c: char) -> bool {
    !c.is_whitespace() && c != '(' && c != ')' && c != '[' && c != ']'
}

fn is_string(c: char) -> bool {
    c != '"'
}

fn is_number(c: char) -> bool {
    c.is_digit(10) || c == '-'
}

fn number(input: &str) -> IResult<&str, Elem> {
    let (input, svalue) = take_while1(is_number)(input)?;
    Ok((input,Elem::Atom(Atom::Number(isize::from_str_radix(svalue, 10).unwrap()))))
}

fn symbol(input: &str) -> IResult<&str, Elem> {
    let (input, name) = take_while1(is_atom)(input)?;
    Ok((input,Elem::Atom(Atom::Symbol(name))))
}

fn string(input: &str) -> IResult<&str, Elem> {
    let (input, name) = dq(take_while1(is_string))(input)?;
    Ok((input,Elem::Atom(Atom::String(name))))
}

fn atom(input: &str) -> IResult<&str, Elem> {
    alt((string,number,symbol))(input)
}

fn single(input: &str) -> IResult<&str, Elem> {
    let (input, _) = tag("#")(input)?;
    let (input, name) = take_while1(is_atom)(input)?;
    Ok((input,Elem::Single(Atom::Symbol(name))))
}

fn call(input: &str) -> IResult<&str, Elem> {
    let (input, _) = tag("(")(input)?;
    let (input, items) = many0(expr)(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, Elem::Call(items)))
}

fn list(input: &str) -> IResult<&str, Elem> {
    let (input, _) = tag("[")(input)?;
    let (input, items) = many0(expr)(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((input, Elem::List(items)))
}

fn expr(input: &str) -> IResult<&str, Elem> {
    alt((ws(single),ws(list),ws(call),ws(atom)))(input)
}

impl<'a> Elem<'a> {
    fn eval(self, env: &mut HashMap<&'a str,Elem<'a>>) -> Elem<'a> {
        match self {
            Elem::Atom(_) => self.eval_atom(env),
            Elem::List(_) => self,
            Elem::Call(_) => self.eval_call(env),
            Elem::Single(value) => Elem::Atom(value)
        }
    }

    fn eval_atom(self, env: &mut HashMap<&'a str,Elem<'a>>) -> Elem<'a> {
        if let Elem::Atom(Atom::Symbol(name)) = self {
            if env.contains_key(name) {
                env[name].clone()
            } else {
                self
            }
        } else {
            self
        }
    }

    fn eval_call(self, env: &mut HashMap<&'a str,Elem<'a>>) -> Elem<'a> {
        if let Elem::Call(ref items) = self {
            if items.len() == 0 {
                return self
            }
            match items[0] {
                Elem::Atom(Atom::Symbol("cons")) => items[1].clone().eval(env).cons(items[2].clone().eval(env)),
                Elem::Atom(Atom::Symbol("append")) => items[1].clone().eval(env).rcons(items[2].clone().eval(env)),
                Elem::Atom(Atom::Symbol("list")) => {
                    let mut ls = Vec::new();
                    let mut first=true;
                    for item in items {
                        if first {
                            first=false;
                            continue;
                        }
                        ls.push(item.clone().eval(env));
                    }
                    Elem::List(ls)
                },
                Elem::Atom(Atom::Symbol("head")) => items[1].clone().eval(env).car(),
                Elem::Atom(Atom::Symbol("tail")) => items[1].clone().eval(env).cdr(),
                Elem::Atom(Atom::Symbol("atom")) => items[1].clone().eval(env).atom(),
                Elem::Atom(Atom::Symbol("not")) => items[1].clone().eval(env).not(),
                Elem::Atom(Atom::Symbol("eq")) => items[1].clone().eval(env).eq(items[2].clone().eval(env)),
                Elem::Atom(Atom::Symbol("ne")) => items[1].clone().eval(env).ne(items[2].clone().eval(env)),
                Elem::Atom(Atom::Symbol("lt")) => items[1].clone().eval(env).compare(items[2].clone().eval(env),Ordering::Less),
                Elem::Atom(Atom::Symbol("gt")) => items[1].clone().eval(env).compare(items[2].clone().eval(env),Ordering::Greater),
                Elem::Atom(Atom::Symbol("le")) => items[1].clone().eval(env).compare(items[2].clone().eval(env),Ordering::Greater).not(),
                Elem::Atom(Atom::Symbol("ge")) => items[1].clone().eval(env).compare(items[2].clone().eval(env),Ordering::Less).not(),
                Elem::Atom(Atom::Symbol("if")) => items[1].clone().eval(env).ifelse(items[2].clone(),items[3].clone(),env),
                Elem::Atom(Atom::Symbol("cond")) => self.clone().cond(items.clone(),env),
                Elem::Atom(Atom::Symbol("add")) => {
                    let mut sum=0;
                    for item in items {
                        if let Elem::Atom(Atom::Number(addend)) = item.clone().eval(env) {
                            sum += addend;
                        }
                    }
                    Elem::Atom(Atom::Number(sum))
                },
                Elem::Atom(Atom::Symbol("let")) => {
                    if let Elem::Atom(Atom::Symbol(name)) = items[1].clone() {
                        env.insert(name, items[2].clone());
                        items[1].clone()
                    } else {
                        self
                    }
                },
                Elem::Atom(Atom::Symbol(name)) => {
                    if env.contains_key(name) {
                        let mut items_m = items.clone();
                        items_m[0] = env[name].clone();
                        Elem::Call(items_m).eval(env)
                    } else {
                        self
                    }
                },
                Elem::Call(ref subitems) => {
                    match subitems[0] {
                        Elem::Atom(Atom::Symbol("fun")) => {
                            let mut env_m = env.clone();
                            if let Elem::List(names) = subitems[1].clone() {
                                let mut i=1;
                                for name in names {
                                    if let Elem::Atom(Atom::Symbol(name_a)) = name {
                                        env_m.insert(name_a,items[i].clone().eval(env));
                                    }
                                    i+=1;
                                }
                                subitems[2].clone().eval(&mut env_m)
                            } else {
                                self
                            }
                        },
                        _ => self
                    }
                }
                _ => self
            }
        } else {
            self
        }
    }

    fn cons(self, other:Elem<'a>) -> Elem<'a> {
        match other {
            Elem::Call(mut items) | Elem::List(mut items) => {
                items.insert(0, self);
                return Elem::List(items);
            },
            _ => Elem::List(vec![self, other])  
        }
    }

    fn rcons(self, other:Elem<'a>) -> Elem<'a> {
        match self {
            Elem::Call(mut items) | Elem::List(mut items) => {
                items.push(other);
                return Elem::List(items);
            },
            _ => Elem::List(vec![self, other])  
        }
    }

    fn car(self) -> Elem<'a> {
        match self {
            Elem::Call(ref items) | Elem::List(ref items) => {
                if items.len() == 0 {
                    return Elem::List(vec![])
                }
                return items[0].clone();
            },
            _ => self
        }
    }

    fn cdr(mut self) -> Elem<'a> {
        match self {
            Elem::Call(ref mut items) | Elem::List(ref mut items) => {
                if items.len() == 0 {
                    return Elem::List(vec![])
                }
                items.remove(0);
                return Elem::List(items.to_vec());
            },
            _ => return Elem::List(vec![])
        }
    }

    fn atom(self) -> Elem<'a> {
        match self {
            Elem::Atom(_) | Elem::Single(_) => Elem::Single(Atom::Symbol("t")),
            _ => Elem::List(vec![])
        }
    }

    fn not(self) -> Elem<'a> {
        match self {
            Elem::List(items) | Elem::Call(items) => if items.len() == 0 {
                Elem::Single(Atom::Symbol("t"))
            } else {
                Elem::List(vec![])
            }
            _ => Elem::List(vec![])
        }
    }

    fn eq(self, other:Elem<'a>) -> Elem<'a> {
        match self {
            Elem::Atom(a) | Elem::Single(a) => match other {
                Elem::Atom(b) | Elem::Single(b) => if a == b {
                    Elem::Single(Atom::Symbol("t")) 
                } else {
                    Elem::List(vec![])
                }
                _ => Elem::List(vec![]),
            },
            _ => Elem::List(vec![]),
        }
    }

    fn ne(self, other:Elem<'a>) -> Elem<'a> {
        match self {
            Elem::Atom(a) | Elem::Single(a) => match other {
                Elem::Atom(b) | Elem::Single(b) => if a != b {
                    Elem::Single(Atom::Symbol("t")) 
                } else {
                    Elem::List(vec![])
                }
                _ => Elem::List(vec![]),
            },
            _ => Elem::List(vec![]),
        }
    }

    fn compare(self, other:Elem<'a>, order:Ordering) -> Elem<'a> {
        match self {
            Elem::Atom(Atom::Number(a)) | Elem::Single(Atom::Number(a)) => match other {
                Elem::Atom(Atom::Number(b)) | Elem::Single(Atom::Number(b)) => if a.cmp(&b) == order {
                    Elem::Single(Atom::Symbol("t")) 
                } else {
                    Elem::List(vec![])
                }
                _ => Elem::List(vec![]),
            },
            _ => Elem::List(vec![]),
        }
    }

    fn ifelse(self, t:Elem<'a>, f:Elem<'a>, env: &mut HashMap<&'a str,Elem<'a>>) -> Elem<'a> {
        match self {
            Elem::Atom(_) | Elem::Single(_) => t.eval(env),
            _ => f.eval(env),
        }
    }

    fn cond(self, items:Vec<Elem<'a>>, env: &mut HashMap<&'a str,Elem<'a>>) -> Elem<'a> {
        let mut first=true;
        for item in items {
            if first {
                first=false;
                continue;
            }
            match item {
                Elem::List(pair) => match pair[0].clone().eval(env) {
                    Elem::Atom(_) | Elem::Single(_) => return pair[1].clone().eval(env),
                    _ => {},
                },
                _ => {}
            }
        }
        Elem::List(vec![])
    }
}

pub fn eval_and_print<'a>(input:&'a str,env:&mut HashMap<&'a str,Elem<'a>>) -> Result<&'a str,Box<dyn Error + 'a>>{
    let (input, elem) = expr(input)?;
    println!("{}",elem.eval(env));
    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing() {
        let result = expr("(cons #A [B C :D \"EFG\" 1 2 3])");
        println!("{}",result.unwrap().1.eval(&mut HashMap::new()));
    }

    #[test]
    fn things() {
        let result = expr("(value (head [:KEY #VALUE]))");
        println!("{}",result.unwrap().1.eval(&mut HashMap::new()));
    }

    #[test]
    fn numbers() {
        let result = expr("(let second (car (cdr x)))");
        let mut env = HashMap::new();
        println!("{}",result.unwrap().1.eval(&mut env));
        let result2 = expr("(second A B C)");
        println!("{}",result2.unwrap().1.eval(&mut env));
    }

    #[test]
    fn cond_test() {
        let result = expr("(cond [(le (add 3 2) 5) \"3 + 2 <= 5\"] [T \"Catch-all\"])");
        println!("{}",result.unwrap().1.eval(&mut HashMap::new()));
    }

    #[test]
    fn fun_test() {
        let result = expr("(let tri (fun [n] (if (gt n 0) (add n (tri (add n -1))) (0))))");
        let mut env = HashMap::new();
        println!("{}",result.unwrap().1.eval(&mut env));
        let result2 = expr("(tri 5)");
        println!("{}",result2.unwrap().1.eval(&mut env));
    }
}
