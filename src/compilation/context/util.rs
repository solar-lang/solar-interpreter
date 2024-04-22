use solar_parser::ast::expr::{let_in::LetExpression, FullExpression};


pub struct SimpleLet<'a> {
    span: &'a str,
    name: &'a str,
    assign: &'a FullExpression<'a>,
    body: &'a FullExpression<'a>,
}

pub fn flatten(expr: &LetExpression) -> SimpleLet {
    flatten_h(expr, 0)
}

fn flatten_h(expr: &LetExpression, i: usize) -> SimpleLet {
    let def = &expr.definitions[i..];

    if def.len() == 1 {
        let (name, assign) = &def[0];
        let body = &expr.body;
        let span = unsafe {solar_parser::from_to(name.span, body.span());
        let name = name.value;

        return SimpleLet {
            span,
            name,
            assign,
            body
        };
    }

}