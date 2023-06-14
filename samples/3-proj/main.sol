# TODO with types coming, this line will be redundant 
use @std.types.concat
use @std.io.(println, readline)

fun main() =
    let name = readline "Hey there!\nWhat's your name? ",
        greeting = greet name
    in
        println greeting

fun greet(n: String) -> String =
    "Hello " ++ n