
# goal is to not call types, other than Functions

fun main() =
    let g = greet "Peter"
    in
        println g

fun greet(n: String) -> String =
    "Hello " ++ n

fun concat(a: String, b: String) -> String =
    buildin_str_concat a b

fun println(msg: String) =
    let msg = msg ++ "\n" in
        buildin_print msg
