
# Resolving symbols

This is a two step problem in solar.
Step one is finding all candidates.
Step two is selecting exactly one candidate to be used.


## Finding candiates

- 0.) Maybe it's just a symbol in scope
  [name] = path => might be symbolic lookup
       if `name` is in scope:
       return `scope[name]`
  candidates := []
- 1.) if the path has only one element,
    we might be doing symbolic lookup in current module.
    No Need to check imports for this.
    But remember, there's a catch.
      candidates.append_all(find_inn_module(this_module))
- 2.) see, if the element is from an import
    basepath := imports.contains(path[0])
    full_path := basepath ++ path[1..]
  now, find the symbol full_path.last() in module fullpath[..(-1)]
  module: collection of files (ASTs) in directory and lib
  e.g. seek through all ASTs in module
    candidates.append_all(find_in_module(full_path))
    return candidates

## Selecting candidates

```julia
function find_appropriate(candidates, args)
  if candidates.len() == 1 {
    return candidates[0]
  }

  let types = map(args, types)

  # now, we need to see which ones fit nicely.
  # we hope it's only one!

  # only select candidates, which adhere to type
  let candidates = filter(candidates, begin (c)
    # note, if args == [], anything may match.
    matches(c, types)
  end)

  return candidates
end

# lots of error handling here.

c = find_appropriate(candidates, args)
if len(c) == 0
  error ""
end

if exists(type_hint)
  cn = filter(c, matches(type_hint))

  if len(cn) == 0
    if len(c) > 0
      error "no matches found for typehint. Found:" + c
    else
      error "no matches found for typehint."
    end
  end

  if len(cn) > 1
    error "Multiple declarations found for function. Remove import or delete one."
  end

  return cn
end

if len(c) > 1
  error "Please specify type hint. Found multiple candidates: " + c
end

return c[0]
```

