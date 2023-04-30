

fun main() =
    let n = readline,
    let g = greet n
    in
        println g

fun greet(n: String) -> String =
    "Hello" ++ n
