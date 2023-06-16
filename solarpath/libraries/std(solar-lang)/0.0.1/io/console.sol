use types.concat

# Reads a single line from standard input
# and returns it as string
fun readline(prompt: String) = buildin_readline prompt

# pub fun readline() = buildin_readline


# Prints the string to standart output
# Without appending a new line character
fun print(message: String) = buildin_print message

# Prints the message to the sandart output,
# along with a new line
fun println(message: String) =
  let message = message ++ "\n" in
    buildin_print message
