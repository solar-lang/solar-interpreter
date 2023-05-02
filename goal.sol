
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
    let n = buildin_readline "What's your name?\n",
        g = greet n
    in
        print g

fun greet(n: String) -> String =
    "Hello " ++ n ++ "\n"
