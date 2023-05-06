# Libraries and Modules

## Solar Path

the solar path is ~/.solar.
Can be configured using the env SOLAR_PATH.
if .solar is mentioned, actually we mean the SOLAR_PATH.


## Libraries

a library can be uniquely identified be (name, publisher, version).
That way it's possible to have multiple versions of the same library
in a given project.

Libraries can be found in ~/.solar/libraries.

The directory path is "$name($publisher)/$version/"
e.g. "std(solar-lang)/0.0.1"


### Library layout
.
|- solar.yaml
|- files.. .sol


## Modules

Structures for resolving Modules
and Symbols inside modules:

    ProjectConfig
        # (only) for the current project (and local libs), an publisher will be ommitted
        basepath: ["nice-gui()", "2.1.3"],
        depmap:
            "std" => ["std(solar-lang)", "0.0.1"]

    ModuleConfig
        project_id: usize
        # include projectconfig reference?
        # include basepath?
        # path: ["types"]
        files: [FileContext("string.sol"), FileContext("string_util.sol")]
        # TODO static_functions: Array<{name: String, args: Vec<Type>, ret: Type}>

    FileContext    (owned by ModuleConfig)
        # notice, how the function names are not path of their path (e.g. module identifier)
        imports:
            # e.g.
            # use @std.string.concat
            "concat" => ["std(solar-lang)", "0.0.1", "string"]
            # use util.(flipbits, redraw)
            "flipbits" => ["nice-gui()", "0.0.1", "util" ]
            "redraw" => ["nice-gui()", "0.0.1", "util"]
        ast: Ast
        # TODO compiled functions here?
        # or on module level?
        # ==> on module level, because +
            1.) we need file distinction only for resolving imports
            2.) we have a flat hierarchy inside a module.

}

*/
