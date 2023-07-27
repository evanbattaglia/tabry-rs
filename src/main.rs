mod example_config_json;
mod types;
mod config_wrapper;
mod machine_state;
mod machine;

fn main() {
    let mut machine = machine::Machine::new(
        serde_json::from_str(example_config_json::STR).unwrap()
    );

    let tokens = ["move", "go", "vehicle1"];
    for token in tokens {
        machine.next(&String::from(token)).unwrap();
    }

    println!("{:?}", machine.state);
}


