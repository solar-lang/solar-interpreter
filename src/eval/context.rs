use super::interpreter::InterpreterContext;
use super::EvalError;
use crate::{
    id::{IdModule, SymbolId, SSID},
    project::{FileInfo, FindError, GlobalModules, Module, ProjectInfo, SymbolResolver},
    types::Type,
    util::{self, IdPath, Scope},
    value::Value,
};
use hotel::HotelMap;
use solar_parser::ast::{self, body::BodyItem, expr::FullExpression};
use std::io::{Read, Write};
use std::sync::Mutex;

/// Struct that gets created once globally
/// Containing Information about all Modules, ASTs, Projects
pub struct CompilerContext<'a> {
    /// Information about all loaded dependencies and sub-dependencies, flattend.
    pub project_info: &'a ProjectInfo,
    /// contains all ASTs across all modules and (sub-)dependencies
    pub module_info: GlobalModules<'a>,

    /// Contains all Type Information.
    pub types: HotelMap<SSID, Type>,

    /// Contains runtime configurations, like stdin and stdout
    pub interpreter_ctx: Mutex<InterpreterContext>,
}

impl<'a> CompilerContext<'a> {
    /// Creates a new Compiler Context with stdin and stdout
    /// propagated
    pub fn with_default_io(
        project_info: &'a ProjectInfo,
        module_info: GlobalModules<'a>,
        types: HotelMap<SSID, Type>,
    ) -> Self {
        CompilerContext {
            project_info,
            module_info,
            interpreter_ctx: Mutex::new(InterpreterContext::default()),
            types,
        }
    }

    pub fn get_symbol(&self, (module, file, item): SymbolId) -> (&Module, &FileInfo, &BodyItem) {
        let module = self
            .module_info
            .get(&module)
            .expect("IdModule  to be valid");

        let fileinfo = module.files.get(file as usize).expect("IdFile to be valid");

        use crate::id::IdItem;
        let item = match item {
            IdItem::Func(id) => &fileinfo.ast.items[id as usize],
            IdItem::GlobalVar(id) => &fileinfo.ast.items[id as usize],
            IdItem::Type(id) => &fileinfo.ast.items[id as usize],
            IdItem::Method(_typeid, _fieldid) => {
                unimplemented!("accessing derived methods is not yet implemented")
            }
        };

        (module, fileinfo, item)
    }

    /// Finds the main function of the current target project
    pub fn find_target_main(&'a self) -> Result<SymbolId, FindError> {
        let path = util::target_id();
        let module = self.module_info.get(&path).unwrap();

        let mut candidates = module.find("main", &path)?;

        if candidates.len() != 1 {
            return Err(FindError::TooMany {
                symbol: "main".to_string(),
                module: path,
            });
        }

        let f_main = candidates.pop().unwrap();
        Ok(f_main)
    }

    /// Resolve module based on idpath
    pub fn resolve_module(&self, idpath: &[String]) -> Result<&Module<'a>, FindError> {
        self.module_info
            .get(idpath)
            .ok_or_else(|| FindError::ModuleNotFound(idpath.to_vec()))
    }
}

#[derive(Clone)]
struct Lookup<'a> {
    module: &'a Module<'a>,
    idmodule: IdModule,
    imports: &'a SymbolResolver,
}

/// Evaluation related stuff.
impl<'a> CompilerContext<'a> {
    // TODO rename to resolve symbol
    // and build up static table
    pub fn eval_symbol(
        &'a self,
        symbol_id: SymbolId,
        args: &[Value<'a>],
    ) -> Result<Value<'a>, EvalError> {
        let (module, fileinfo, item) = self.get_symbol(symbol_id.clone());

        let lookup = Lookup {
            module,
            idmodule: symbol_id.0,
            imports: &fileinfo.imports,
        };

        match item {
            BodyItem::Function(func) => self.eval(func, lookup, args),
            BodyItem::Let(var) => {
                if !args.is_empty() {
                    return Err(EvalError::CallingVariable {
                        identifer: var.identifier.span.to_string(),
                        file: fileinfo.filename.to_string(),
                    });
                }

                // TODO for pointers and mutability, you would return the index on the global stack of the variable.
                // here we just return the value??
                todo!("value of global variable")
            }
            BodyItem::Test(_) => {
                unreachable!("SymbolId should never reference Tests in this context")
            }
            BodyItem::TypeDecl(_ty) => {
                unimplemented!("generate functions from types to create types.")
            }
        }
    }

    /// Evaluate a function,
    fn eval(
        &'a self,
        ast: &ast::Function,
        lookup: Lookup,
        args: &[Value<'a>],
    ) -> Result<Value<'a>, EvalError> {
        let mut scope = Scope::new();

        // TODO what to do with the type here?
        for ((ident, _ty), val) in ast.args.iter().zip(args) {
            scope.push(ident.value, val.clone());
        }

        self.eval_full_expression(&ast.body, lookup, &mut scope)
    }

    fn eval_full_expression(
        &'a self,
        expr: &FullExpression,
        lookup: Lookup,
        scope: &mut Scope<'a>,
    ) -> Result<Value, EvalError> {
        match expr {
            FullExpression::Let(expr) => {
                // Insert all let bindings into scope
                // and evaluate their expressions
                for (ident, value) in &expr.definitions {
                    let value = self.eval_full_expression(value, lookup.clone(), scope)?;
                    scope.push(ident.value, value)
                }

                // We now have readied the scope and are able to evaluate the body

                let v = self.eval_full_expression(&expr.body, lookup, scope);

                // Now we remove the let bindings from the scope
                for _ in &expr.definitions {
                    scope.pop();
                }

                v
            }

            FullExpression::Expression(ref expr) => self.eval_minor_expr(expr, lookup, scope),
            FullExpression::Concat(expr) => {
                let e = expr.to_expr();
                self.eval_minor_expr(&e, lookup, scope)
            }
            expr => panic!("Unexpected type of expression: {expr:#?}"),
        }
    }

    fn eval_minor_expr(
        &'a self,
        expr: &ast::expr::Expression,
        lookup: Lookup,
        scope: &mut Scope<'a>,
    ) -> Result<Value<'a>, EvalError> {
        match expr {
            ast::expr::Expression::FunctionCall(fc) => {
                // First, evaluate all arguments
                let mut args: Vec<Value> = Vec::with_capacity(fc.args.len());
                for arg in fc.args.iter() {
                    let v = self.eval_sub_expr(&arg.value, lookup.clone(), scope)?;
                    args.push(v);
                }

                // See, if we're calling a special buildin function
                if let Some(result) = self.check_buildin_func(fc, &args) {
                    return result;
                }

                // Find function name in scope
                let path = util::normalize_path(&fc.function_name);

                // TODO this does not yet check, if the module where the type of the first
                // argument is declared, contains the symbol. This has precedence over imports and declarations.
                let mut symbol_candidates = self.resolve_symbol(&path, lookup, scope)?;

                // TODO check all candidates first!
                if symbol_candidates.len() > 1 {
                    panic!("found multiple candidates for {path:?}:\n{symbol_candidates:#?}");
                }

                let symbol: Value = symbol_candidates.pop().unwrap();

                match symbol {
                    // Only evaluate functions directly
                    // otherwise return value
                    Value::Function(function_index) => {
                        // TODO dont create functioncontext here. Instead, move fc to global context.
                        // A given FC needs to be created only once.
                        // With the FC AND the argument-types we have all needed context to compile a function.
                        // let ctx = func.ctx(&self.ctx);
                        self.eval_symbol(function_index, &args)
                    }
                    // if there are argument supplied to values,
                    // this is definitly and error.
                    v if !args.is_empty() => Err(EvalError::TypeError {
                        got: format!("{v}"),
                        wanted: "fun(...) -> ...".to_string(),
                    }),
                    value => Ok(value),
                }
            }
            ast::expr::Expression::Value(value) => self.eval_sub_expr(value, lookup, scope),
        }
    }

    fn eval_sub_expr(
        &'a self,
        expr: &ast::expr::Value,
        lookup: Lookup,
        scope: &mut Scope<'a>,
    ) -> Result<Value, EvalError> {
        use ast::expr::Literal;
        use ast::expr::Value as V;
        match expr {
            V::Literal(lit) => match lit {
                Literal::StringLiteral(s) => Ok(s.value.to_string().into()),
                Literal::Bool { value, .. } => Ok(Value::Bool(*value)),
                Literal::Int(int) => {
                    let i = util::eval_int(int);
                    if let Err(e) = i {
                        return Err(e.into());
                    }

                    Ok(Value::Int(i.unwrap()))
                }
                Literal::Float(f) => {
                    let f = f.parse::<f64>().expect("float to be in valid f64 form");
                    Ok(Value::Float(f))
                }
            },
            V::FullIdentifier(path) => {
                // Actually, I don't think I want to allow Paths here.
                // just field access.
                // this line is likely to be deleted.
                let path = util::normalize_path(path);

                if path.len() != 1 {
                    panic!("no field access like this");
                }

                let mut result = self.resolve_symbol(&path, lookup, scope)?;
                if result.len() != 1 {
                    if result.is_empty() {
                        panic!("no results looking up {path:?}:\n {result:#?}")
                    }
                    panic!("found multiple results for {path:?}:\n {result:#?}")
                }

                Ok(result.pop().unwrap())
            }
            V::Tuple(expr) => {
                if expr.values.len() > 1 {
                    panic!("tuple values are not ready");
                }
                let expr = &expr.values[0];

                self.eval_full_expression(expr, lookup, scope)
            }
            _ => panic!("evaluation not ready for \n{expr:#?}"),
        }
    }

    ///
    /// Returns a set of candidates for the symbol.
    /// Resolving the candidates requires further knowledge.
    ///
    /// how do we find symbols?
    /// 0.) Maybe it's just a symbol in scope
    /// [name] = path => might be symbolic lookup
    ///      if `name` is in scope:
    ///      return `scope[name]`
    ///
    /// candidates := []
    ///
    /// 1.) if the path has only one element,
    ///     we might be doing symbolic lookup in current module.
    ///     No Need to check imports for this.
    ///     But remember, there's a catch.
    /// candidates.append_all(find_inn_module(this_module))
    ///
    /// 2.) see, if the element is from an import
    ///
    /// basepath := imports.contains(path[0])
    /// full_path := basepath ++ path[1..]
    /// now, find the symbol full_path.last() in module fullpath[..(-1)]
    /// module: collection of files (ASTs) in directory and lib
    /// e.g. seek through all ASTs in module
    /// candidates.append_all(find_in_module(full_path))
    ///
    /// return candidates
    fn resolve_symbol(
        &'a self,
        path: &[String],
        Lookup {
            module,
            idmodule,
            imports,
        }: Lookup,
        // TODO type of first argument is also relevant! Add as argument
        scope: &Scope<'a>,
    ) -> Result<Vec<Value<'a>>, EvalError> {
        // TODO check if it was found before, and return compiled version

        // if the length of the path is > 1, it's guaranteed looking up an import.

        // if there is no path, this might
        // be just a symbol declared earlier
        // via let ... in, or passed as an argument
        if let [name] = path {
            // 0.) See, if it's a symbol in scope.
            // Local scope overrides everything.
            // The scope only holds arguments and let declarations.
            // Only one item will be returned by this.
            if let Some(item) = scope.get(name) {
                // TODO this is the place where we can return references
                // e.g. in order to assign to stuff.
                return Ok(vec![item.clone()]);
            }
        }

        let mut candidates: Vec<Value<'a>> = Vec::new();
        if let [name] = path {
            // if the path is only one element long,
            // we must also look up the local module.
            // that is ALL Asts within this module.

            if let Ok(res) = module.find(name, &idmodule) {
                for symbolid in res {
                    candidates.push(Value::Function(symbolid));
                }
            }

            // else not found in current module
        }

        // 2.) see, if the element is from an import
        // Note, this might result in a number of candidates to check!
        // E.g.  add(Int, Float) -> Float     declared in local scope
        //       add(Int, Int) -> Int         imported
        //
        // basepath := imports.contains(path[0])
        // full_path := basepath ++ path[1..]
        // now, find the symbol full_path.last() in module fullpath[..(-1)]
        // module: collection of files (ASTs) in directory and lib
        // e.g. seek through all ASTs in module
        // candidates.append_all(find_in_module(full_path))
        //
        // return candidates

        let symbol = &path[0];
        if let Some(imports) = imports.get(symbol) {
            for import in imports {
                // TODO if path[1..].len() > 1, then imports should be length 1.
                // because it means we are importing an entire module, and we shouldn't import multiple modules
                // with the same name, I think.

                let idmodule: IdPath = import
                    .iter()
                    .cloned()
                    .chain(path.iter().skip(1).cloned())
                    .collect();

                // now basepath contains the full path id!
                // neat :)

                // let symbol = &basepath.last().expect("find element in path");

                let Ok(module) = self.resolve_module(&idmodule) else {
                    // eprintln!("skipping over module {idmodule:?}, not found");
                    continue;
                };

                // candidates from this module
                let Ok(cs) = module.find(symbol, &idmodule) else {
                    continue;
                };

                for c in cs {
                    candidates.push(Value::Function(c));
                }
            }
        }

        Ok(candidates)
    }
}

/// Buildin Functions
impl<'a> CompilerContext<'a> {
    /// Checks, whether supplied function call is a buildin function
    /// buildin functions behave quite different from values in some respect,
    /// which is fine. They will be hidden away in the stdlib.
    /// Returns None, if the supplied function call does not address a buildin function.
    pub(crate) fn check_buildin_func(
        &'a self,
        func: &ast::expr::FunctionCall,
        args: &[Value<'a>],
    ) -> Option<Result<Value<'a>, EvalError>> {
        if func.function_name.value.len() != 1 {
            return None;
        }

        let fname = func.function_name.value[0].value;

        if !fname.starts_with("buildin_") && !fname.starts_with("Buildin_") {
            return None;
        }

        // cut off "buildin_" or "Buildin_"
        let shortened = &fname["buildin_".len()..];

        let res = match shortened {
            "str_concat" => self.buildin_str_concat(args),
            "identity" => self.buildin_identity(args),
            "readline" => self.buildin_readline(args),
            "print" => self.buildin_print(args),

            _ => Err(EvalError::WrongBuildin {
                found: fname.to_string(),
            }),
        };

        Some(res)
    }

    pub(crate) fn buildin_str_concat(&self, args: &[Value]) -> Result<Value, EvalError> {
        let mut s = String::new();

        for arg in args {
            match arg {
                Value::String(arg) => s.push_str(arg),
                _ => {
                    return Err(EvalError::TypeError {
                        got: arg.type_as_str().to_string(),
                        wanted: "String".to_string(),
                    })
                }
            }
        }

        Ok(s.into())
    }

    pub(crate) fn buildin_print(&self, args: &[Value]) -> Result<Value, EvalError> {
        // allowed overloadings:
        // [String]
        // []

        let mut out = self.interpreter_ctx.lock().expect("lock interpreter io");
        for arg in args {
            write!(*out, "{arg}").expect("write to interpreter io");
        }
        out.flush().expect("write to interpreter io");

        Ok(Value::Void)
    }

    pub(crate) fn buildin_identity(&'a self, args: &[Value<'a>]) -> Result<Value<'a>, EvalError> {
        // only the identiy overloading is implemented for now.
        if args.len() != 1 {
            panic!("& is only implemented with 1 argument");
        }

        Ok(args[0].clone())
    }

    pub(crate) fn buildin_readline(&self, args: &[Value]) -> Result<Value, EvalError> {
        let mut iio = self.interpreter_ctx.lock().expect("lock interpreter io");

        // allowed overloadings:
        // [String]
        // []
        if !args.is_empty() {
            // Check that no more than 1 argument got supplied
            if args.len() > 1 {
                panic!("Expected 1 argument of type string to buildin_readline");
            }

            // Verify that it is of type string
            let s = if let Value::String(s) = &args[0] {
                s
            } else {
                panic!("Expected argument to buildin_readline to be of type string");
            };

            write!(iio, "{s}").expect("write to interpreter io");
            iio.flush().expect("flush interpreter io");
        }

        let mut s = Vec::new();

        loop {
            // read exactly one character
            let mut buf = [0];
            iio.read_exact(&mut buf).expect("read from input");

            // grab buffer as character
            let b = buf[0];

            if b == b'\n' {
                break;
            }

            s.push(b)
        }

        let s = String::from_utf8(s).expect("parse stdin as a string");
        Ok(s.into())
    }
}
