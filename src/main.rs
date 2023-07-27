mod example_config_json;
mod types;
mod config_wrapper;
mod machine;

use serde::Deserialize;

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


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn load_fixture_file<T: for<'a>Deserialize<'a>>(filename: &str) -> T {
        let file_str = fs::read_to_string(format!("fixtures/{filename}")).unwrap();
        serde_json::from_str::<T>(&file_str).unwrap()
    }

    // TODO it would be nice to split this up into multiple test so it doesn't fail immediately,
    // but I don't know how to do that with the current test framework
    #[test]
    fn test_all_expectations() {
        // load fixture files
        let tabry_conf: types::TabryConf = load_fixture_file("vehicles.json");
        let expectations: serde_json::Value = load_fixture_file("vehicles-expectations.json");

        // let tabry_conf = fs::read_to_string("fixtures/vehicles.json").unwrap();
        // let tabry_conf : types::TabryConf = serde_json::from_str(&tabry_conf).unwrap();
        // let expectations = fs::read_to_string("fixtures/vehicles-expectations.json").unwrap();
        // let expectations: serde_json::Value = serde_json::from_str(&expectations).unwrap();
        
        for (name, test_case) in expectations.as_object().unwrap() {
            println!("TESTING TEST CASE {name}");
            let mut machine = machine::Machine::new(tabry_conf.clone());
            // test_case is an array with 1) the tokens and 2) the expected state
            let tokens = test_case[0].as_array().unwrap();
            let expected_state = &test_case[1];

            // loop over tokens:
            for token in tokens {
                machine.next(&token.as_str().unwrap().to_string()).unwrap();
            }

            let machine_state_as_serde_value = &serde_json::value::to_value(&machine.state).unwrap();
            assert_eq!(machine_state_as_serde_value, expected_state);
        }
    }
}

