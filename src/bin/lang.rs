fn main() {
    let ast = rabry::lang::ast::parse("
        cmd control-vehicle
        arg {
          opts const car
        }
    ");
    let ast = ast.unwrap();
    let config = rabry::lang::translator::translate(&ast);
    let json = serde_json::to_string(&config);
    print!("{}", json.unwrap());
}
