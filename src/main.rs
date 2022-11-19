use std::{hash::Hash, collections::HashMap, error::Error};

use qbscript::eval_and_print;

fn main() -> Result<(),Box<dyn Error>> {
    let mut env=HashMap::new();
    let mut input = "
(let x 7)
(let double (fun [n] (add n n)))
(double x)
(let reverse (fun [l] (if (not l)
    ()
    (append (reverse (tail l)) (head l)))))
(reverse [A B C D E F G])
(let dec (fun [n] (add n -1)))
(let iota (fun [n] (if (gt n 0) (append (iota (dec n)) n) n)))
(iota 10)
    ";
    loop {
        input = eval_and_print(input,&mut env)?;
        if input.len() == 0 {
            break;
        }
    }
    Ok(())
}