
# for now
# - no imports
# - no modules
# - no overloading
# - no generics
# - no Interfaces

# but
# - static typing
# - function resolve
# - buildin functions

fun main() =
    let n = buildin_readline "What's your name? ",
        g = greet n
    in
        println g

fun greet(n: String) -> String =
    "Hello " ++ &n

fun &(a) = buildin_identity a

fun concat(a: String, b: String) -> String = 
    buildin_str_concat a b

fun println(msg: String) = 
    let msg = &msg ++ "\n" in
        buildin_print msg