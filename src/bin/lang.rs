fn main() {
    let ast = rabry::lang::ast::parse("
        cmd foo
        varargs bar
        opt arg
    ");
    let ast = ast.unwrap();
    dbg!(&ast);
    let config = rabry::lang::translator::translate(&ast);
    dbg!(config);
}
