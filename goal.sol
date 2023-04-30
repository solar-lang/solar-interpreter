
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
    let n = readline,
        g = greet n
    in
        println g

fun greet(n: String) -> String =
    "Hello" ++ n
