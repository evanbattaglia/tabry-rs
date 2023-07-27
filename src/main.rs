mod example_config_json;
mod types;
mod config_wrapper;
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


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn assert_state_equals(state: &machine::MachineState, expected: &serde_json::Value) {
        let expected_state = expected.as_object().unwrap();
        let expected_mode = expected_state.get("mode").unwrap().as_str().unwrap();
        let expected_subcommand_stack = expected_state.get("subcommand_stack").unwrap().as_array().unwrap();
        let expected_flags = expected_state.get("flags").unwrap().as_object().unwrap();
        let expected_flag_args = expected_state.get("flag_args").unwrap().as_object().unwrap();
        let expected_args = expected_state.get("args").unwrap().as_array().unwrap();
        let expected_help = expected_state.get("help").unwrap().as_bool().unwrap();
        let expected_dashdash = expected_state.get("dashdash").unwrap().as_bool().unwrap();

        assert_eq!(state.mode, expected_mode);
    }

            

    #[test]
    fn test_some_stuff() {
        // read the fixture file:
        let expectations = fs::read_to_string("fixtures/vehicles-expectations.json").unwrap();
        let tabry_config_str = fs::read_to_string("fixtures/vehicles.json").unwrap();
        let tabry_conf : types::TabryConf = serde_json::from_str(&tabry_config_str).unwrap();
        
        // expectations JSON is an object with key the name of the test and the values the test
        // parameters:
        let exp1: serde_json::Value = serde_json::from_str(&expectations).unwrap();
        let exp2 = exp1.as_object().unwrap();
        // loop over keys:
        for (name, test) in exp2 {
            // loop over test cases:
            for test_case in test.as_array().unwrap() {
                let mut machine = machine::Machine::new(tabry_conf);
                // test_case is an array with 1) the tokens and 2) the expected state
                let tokens = test_case[0].as_array().unwrap();
                let expected_state = test_case[1].as_object().unwrap();

                // loop over tokens:
                for token in tokens {
                    machine.next(&token.as_str().unwrap().to_string()).unwrap();
                }
                assert_state_equals(machine.state, test_case[1]);
                println!("{:?}", machine.state);
            }
        }




        assert_eq!(2 + 2, 4);
    }
}

