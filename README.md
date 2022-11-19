# Qb Script
A LISP-like language. It's really quite simple.

`[]` is an empty list. Square brackets are equivalent to quoting a list in most LISPs.

`()` is an empty call.

`A`, `bcd`, and `DeFgH` are examples of atoms.

`"This..."` is a string.

`69` is a number. Nice!

`(let x 7)` makes the atom x evaluate to 7.

`#x` is the equivalent to `(quote x)` in most LISPs.

`(let double (fun [n] (add n n)))` defines a function called double that adds a value to itself.

`(double 2453)` should then evaluate to 4906.

## Reference
`(cons A B)` prepends A to B

`(append A B)` appends B to A

`(list A B C ... )` makes a list from the results of evaluating each argument.

`(head A)` first element of A

`(tail A)` all but the first element of A

`(atom A)` returns #t if A evaluates to an atom (i.e., not a list.)

`(not A)` returns #t if A evaluates to an empty list.

`(eq A B)` returns #t if A and B evaluate to atoms and are equal.

`(ne A B)` returns #t if A and B evaluate to atoms and are not equal.

`(lt A B)`, `(gt A B)`, `(le A B)`, `(ge A B)` return #t if A and B evaluate to atoms and A < B, A > B, A <= B and A >= B respectively.

`(if A B C)` evaluates and returns B if A evaluates to an atom, otherwise C is evaluated and returned.

`(cond [C1 E1] [C2 E2] [C3 E3] ... )` goes through each pair of expressions, evaluates the first, and if it is an atom it evaluates and returns the second.

`(add A B C ... )` returns the sum of the results of all the expressions provided to it.

`(let A B)` defines the atom A as B.

`(fun A B)` is a lambda expression binding names in the list A to values from the surrounding call, and evaluating B.