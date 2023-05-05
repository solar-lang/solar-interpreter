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