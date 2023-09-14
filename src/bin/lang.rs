fn main() {
    let program = rabry::lang::grammar::parse("
        cmd foo
        varargs bar
        opt arg
    ");
    dbg!(program);
}
