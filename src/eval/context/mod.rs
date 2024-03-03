mod function_store;
use self::function_store::{FunctionInfo, FunctionStore};

use super::interpreter::InterpreterContext;
use super::CompilationError;
use crate::{
    compile::{CustomInstructionCode, Instruction, StaticExpression},
    id::{FunctionId, IdModule, SymbolId, TypeId, SSID},
    project::{FileInfo, FindError, GlobalModules, Module, ProjectInfo, SymbolResolver},
    types::{
        buildin::{link_buildin_types, BuildinTypeId},
        Type,
    },
    util::{self, IdPath, Scope},
    value::Value,
};
use hotel::HotelMap;
use solar_parser::ast::{self, body::BodyItem, expr::FullExpression};
use std::io::{Read, Write};
use std::sync::{Mutex, RwLock};

/// Struct that gets created once globally
/// Containing Information about all Modules, ASTs, Projects
pub struct CompilerContext<'a> {
    /// Information about all loaded dependencies and sub-dependencies, flattend.
    pub project_info: &'a ProjectInfo,
    /// contains all ASTs across all modules and (sub-)dependencies
    pub module_info: GlobalModules<'a>,

    /// IDs of buildin types like Int32 etc.
    pub buildin_types: BuildinTypeId,

    /// Contains static, concrete Type Information.
    pub types: RwLock<HotelMap<SSID, Type>>,

    pub functions: RwLock<FunctionStore>,

    /// Contains runtime configurations, like stdin and stdout
    pub interpreter_ctx: Mutex<InterpreterContext>,
}

impl<'a> CompilerContext<'a> {
    /// Creates a new Compiler Context with stdin and stdout
    /// propagated
    pub fn with_default_io(project_info: &'a ProjectInfo, module_info: GlobalModules<'a>) -> Self {
        let (types, buildin_types) = link_buildin_types(&module_info);
        let types = types.into();

        // TODO fill with buildin functions
        let functions = Default::default();

        CompilerContext {
            project_info,
            module_info,
            interpreter_ctx: Mutex::new(InterpreterContext::default()),
            types,
            functions,
            buildin_types,
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

/// Lookuptable to resolve symbols inside a module
#[derive(Clone)]
struct Lookup<'a> {
    module: &'a Module<'a>,
    idmodule: IdModule,
    imports: &'a SymbolResolver,
}

/// Evaluation related stuff.
impl<'a> CompilerContext<'a> {
    /// Main entrypoint for compiling a function.
    /// Will recursively compile all downstream functions, that are getting called within the AST.
    pub fn compile_symbol(
        &'a self,
        symbol_id: SymbolId,
        args: &[TypeId],
    ) -> Result<(FunctionId, TypeId), CompilationError> {
        let (module, fileinfo, item) = self.get_symbol(symbol_id.clone());

        let lookup = Lookup {
            module,
            idmodule: symbol_id.0,
            imports: &fileinfo.imports,
        };

        match item {
            BodyItem::Function(func) => self.compile(func, lookup, &(symbol_id, args.to_vec())),
            BodyItem::Let(var) => {
                // TODO there are no arguments to a global let. the let itself has an expression assigned to it.
                if !args.is_empty() {
                    return Err(CompilationError::CallingVariable {
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
            BodyItem::BuildinTypeDecl(_ty) => {
                unimplemented!("no fields on buildin types")
            }
        }
    }

    /// Compile a function.
    /// The instructions for the function will get stored inside the context.
    /// All this returns is the lookup symbol/index (and the return type) of the function.
    /// If it is already compiled,
    /// this will simply return the index of the function and not compile it again.
    fn compile(
        &'a self,
        ast: &ast::Function,
        lookup: Lookup,
        ssid: &SSID,
    ) -> Result<(FunctionId, TypeId), CompilationError> {
        // First, check if function is already compiled
        {
            let fnstore = self
                .functions
                .read()
                .expect("aquire readlock for functions");

            if let Some((fnid, info)) = fnstore.get_by_key(&ssid) {
                match info {
                    FunctionInfo::Complete { args, body } => {
                        return Ok((fnid, body.ty));
                    }
                    FunctionInfo::Partial => {
                        panic!("only found partial function information during compilation. Find out later what to do here");
                    }
                }
            }
        }

        // The function is not compiled yet.
        // Compile the function

        // First, reserve an index for the function.
        let id = {
            self.functions
                .write()
                .expect("reserve function")
                .reserve(ssid.clone())
        };

        // Then we can start compiling it.
        // First, add the arguments to the scope.
        let mut scope = Scope::new();

        let mut types = Vec::new();
        let args = ssid.1;
        for ((ident, _ty), static_type) in ast.args.iter().zip(args) {
            // TODO what to do with the arguments type here?
            // This might be the right place, for
            //     - autocasting integers.
            //     - autocasting to interface types
            //
            // if _ty != static_type { return Error }

            // we can ignore the index, it's just 1, 2, 3, ... anyway
            let _index = scope.push(ident.value);

            types.push(static_type);
        }

        // TODO there's a problem here.
        // Actually we would like to reserve a spot for our function now.
        // otherwise we can't do recursion.

        // compile the static expression
        let body = self.compile_full_expression(&ast.body, lookup, &mut scope)?;

        let return_type = body.ty;
        // TODO check, that the return type matches the functions return type.
        // TODO possibly map the return value to the type specified in the AST. (e.g. map to interfaces etc.)

        // save function
        self.functions
            .write()
            .expect("store function")
            .update_complete_function(id, types, body);

        Ok((id, return_type))
    }

    fn compile_full_expression(
        &'a self,
        expr: &FullExpression,
        lookup: Lookup,
        scope: &mut Scope,
    ) -> Result<StaticExpression, CompilationError> {
        match expr {
            FullExpression::Let(expr) => {
                let mut let_list = Vec::new();
                // Insert all let bindings into scope
                // and evaluate their expressions
                for (ident, value) in &expr.definitions {
                    let var_value = self.compile_full_expression(value, lookup.clone(), scope)?;
                    let var_index = scope.push(ident);
                    let_list.push((var_index, var_value));
                }

                // We now have readied the scope and are able to evaluate the body
                let body_expression = self.compile_full_expression(&expr.body, lookup, scope)?;

                // It's only now that we know the final return type of the let bindings.
                // It's the one from the body. We can start with building the tree now, in reverse order :)

                let (var_index, var_value) = let_list
                    .pop()
                    .expect("let binding th have at least one definition");

                // return type of the let binding
                let ty = body_expression.ty;

                // The tree we're building (in reverse)
                let mut let_tree = Instruction::NewLocalVar {
                    var_index,
                    var_value,
                    body: body_expression,
                };

                for (var_index, var_value) in let_list.into_iter().rev() {
                    let_tree = Instruction::NewLocalVar {
                        var_index,
                        var_value,
                        body: body_expression,
                    }
                }

                // Now we remove the let bindings from the scope again
                for _ in &expr.definitions {
                    scope.pop();
                }

                Ok(StaticExpression {
                    instr: Box::new(let_tree),
                    ty,
                })
            }

            FullExpression::Expression(ref expr) => self.compile_minor_expr(expr, lookup, scope),
            FullExpression::Concat(expr) => {
                let e = expr.to_expr();
                self.compile_minor_expr(&e, lookup, scope)
            }
            expr => panic!("Unexpected type of expression: {expr:#?}"),
        }
    }

    fn compile_minor_expr(
        &'a self,
        expr: &ast::expr::Expression,
        lookup: Lookup,
        scope: &mut Scope,
    ) -> Result<StaticExpression, CompilationError> {
        match expr {
            ast::expr::Expression::FunctionCall(fc) => {
                // TODO this might be the place for autocasting
                //
                // Start, by compiling the arguments.
                // The static types of them are needed to look up,
                // which function was called.
                // e.g. was is f(Int, Int) or f(String, Int) etc.
                let mut args: Vec<StaticExpression> = Vec::with_capacity(fc.args.len());

                let args = fc
                    .args
                    .iter()
                    .map(|arg| self.compile_value(&arg.value, lookup.clone(), scope))
                    .collect::<Result<Vec<_>, _>>()?;

                // See, if we're calling a special buildin function
                if let Some(result) = self.check_buildin_func(fc, args) {
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
                    v if !args.is_empty() => Err(CompilationError::TypeError {
                        got: format!("{v}"),
                        wanted: "fun(...) -> ...".to_string(),
                    }),
                    value => Ok(value),
                }
            }
            ast::expr::Expression::Value(value) => self.compile_value(value, lookup, scope),
        }
    }

    fn compile_value(
        &'a self,
        expr: &ast::expr::Value,
        lookup: Lookup,
        scope: &mut Scope,
    ) -> Result<StaticExpression, CompilationError> {
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

                self.compile_full_expression(expr, lookup, scope)
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
        first_arg_type: TypeId,
        scope: &Scope,
    ) -> Result<Vec<Value<'a>>, CompilationError> {
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
        args: &[StaticExpression],
    ) -> Option<Result<CustomInstructionCode, CompilationError>> {
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

            _ => Err(CompilationError::WrongBuildin {
                found: fname.to_string(),
            }),
        };

        Some(res)
    }

    /// Assert that all types have the desired type
    fn assert_type_ids(
        &self,
        args: &[StaticExpression],
        wanted_id: u8,
        wanted: &str,
    ) -> Result<(), CompilationError> {
        // verify that all args are strings.
        for arg in args {
            if arg.ty != wanted_id as TypeId {
                let typename = self
                    .types
                    .read()
                    .map(|map| {
                        // Lookup Type info
                        let ty = map.get_by_index(arg.ty).expect("find type in type store");
                        ty.info_name.clone()
                    })
                    .expect("to lookup name of type");

                return Err(CompilationError::TypeError {
                    got: typename,
                    // TODO maybe look up in type info directly
                    wanted: wanted.to_string(),
                });
            }
        }

        Ok(())
    }

    pub(crate) fn buildin_str_concat(
        &self,
        args: &[StaticExpression],
    ) -> Result<CustomInstructionCode, CompilationError> {
        self.assert_type_ids(args, self.buildin_types.string, "String")?;
        Ok(CustomInstructionCode::StrConcat)
    }

    pub(crate) fn buildin_print(
        &self,
        args: &[StaticExpression],
    ) -> Result<CustomInstructionCode, CompilationError> {
        // allowed overloadings:
        // [String]
        // []
        self.assert_type_ids(args, self.buildin_types.string, "String")?;

        Ok(CustomInstructionCode::Print)
    }

    pub(crate) fn buildin_identity(
        &'a self,
        args: &[StaticExpression],
    ) -> Result<CustomInstructionCode, CompilationError> {
        // only the identiy overloading is implemented for now.
        // Later we will implent currying using this, but in solar code itself probably.
        if args.len() != 1 {
            return Err(CompilationError::WrongBuildin {
                found: "& is only implemented with 1 argument".to_string(),
            });
        }

        Ok(CustomInstructionCode::Identity)
    }

    pub(crate) fn buildin_readline(
        &self,
        args: &[StaticExpression],
    ) -> Result<CustomInstructionCode, CompilationError> {
        let mut iio = self.interpreter_ctx.lock().expect("lock interpreter io");

        // allowed overloadings:
        // [String]
        // []

        self.assert_type_ids(args, self.buildin_types.string, "String")?;

        if args.len() > 1 {
            return Err(CompilationError::WrongBuildin {
                found: "readline is only implemented for 0 or 1 (String) argument".to_string(),
            });
        }

        Ok(CustomInstructionCode::Readline)
    }
}
