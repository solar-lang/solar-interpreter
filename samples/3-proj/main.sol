# TODO with types coming, this line will be redundant 
use @std.types.concat
use @std.io..

fun main() =
    let name = readline "Hey there!\nWhat's your name? "
        greeting = greet name
    in
        println g

fun greet(n: String) -> String =
    "Hello " ++ n