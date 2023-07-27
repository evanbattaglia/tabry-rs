use std::mem::swap;

use super::config_wrapper;
use super::types;
use super::machine_state::{MachineState, MachineStateMode};
use super::token_matching::TokenMatching;

use super::result::TabryResult;

pub struct Machine {
    config: config_wrapper::ConfigWrapper,
    pub state: MachineState,
    log: bool
}

// TODO replace error static string
// #[derive(Debug, thiserror::Error)]
// enum MachineError {
//     #[error("include {0} not found")]
//     IncludeNotFound(String),
// }
    

impl Machine {
    // TODO: want to be able to pass a reference in here. need named lifetime. or can clone it...
    pub fn new(conf: types::TabryConf) -> Machine {
        Machine {
            config: config_wrapper::ConfigWrapper::new(conf),
            state: MachineState::default(),
            log: match std::env::var("RABRY_DEBUG") {
              Ok(s) => s != "0" && s != "false",
              Err(_) => false
            }
        }
    }

    pub fn next(&mut self, token: &String) -> Result<(), &'static str> {
        match self.state.mode {
            MachineStateMode::Subcommand => self.match_mode_subcommand(token),
            MachineStateMode::Flagarg { .. } => Ok(self.match_mode_flagarg(token)),
        }
    }

    // TODO: error should be some class probably instead of a string
    fn match_mode_subcommand(&mut self, token: &String) -> Result<(), &'static str> {
        if self.match_subcommand(token)?
            || self.match_dashdash(token)
            || self.match_flag(token)?
            || self.match_help(token)
        {
            Ok(())
        } else {
            // Fallback -- machine treats anything unrecognized as an arg.
            // If the command doesn't take the required numbers of arguments,
            // that will be determined later.
            self.match_arg(token)
        }
    }


    /*
     * TODO using this doesn't work below
    fn current_sub(&mut self) -> Result<&types::TabryConcreteSub, &'static str> {
        self.config.dig_sub(&self.state.subcommand_stack)
    }
    */

    fn match_subcommand(&mut self, token: &String) -> Result<bool, &'static str> {
        if !self.state.args.is_empty() {
            return Ok(false);
        }

        // TODO using self.current_sub() causes weird borrow problem. But also want t
        // make self.find_in_subs etc. be able to mutate self which it can't right now
        // due to weird lifetime problem.
        let sub_here = self.config.dig_sub(&self.state.subcommand_stack)?;

        if let Some(sub) = self.config.find_in_subs(&sub_here.subs, token, true)? {
            let name = sub.name.as_ref().ok_or("sub must have name here")?;
            self.state.subcommand_stack.push(name.clone());
            self.log(format!("STEP subcommand, add {}", name));
            // TODO log
            Ok(true)
        } else {
            Ok(false)
        }

    }

    fn match_dashdash(&mut self, token: &String) -> bool {
        if !self.state.dashdash && token == "--" {
            self.state.dashdash = true;
            true
        } else {
            false
        }
    }

    fn match_flag(&mut self, token: &String) -> Result<bool, &'static str> {
        if self.state.dashdash {
            return Ok(false)
        }

        // Check flags for each Subcommand in stack, starting with the most specific Subcommand.
        for sub in self.config.dig_subs(&self.state.subcommand_stack)?.iter().rev() {
          for flag in self.config.expand_flags(&sub.flags) {
            if flag.match_token(token) {
              if flag.arg {
                self.state.mode = MachineStateMode::Flagarg { current_flag: flag.name.clone() }
              } else {
                self.state.flags.insert(flag.name.clone(), true);
              }
              return Ok(true);
            }
          }
        }

        return Ok(false);
    }

    fn match_help(&mut self, token: &String) -> bool {
        if !self.state.dashdash && (token == "help" || token == "--help" || token == "-?") {
            self.state.help = true;
            true
        } else {
            false
        }
    }

    fn match_arg(&mut self, token: &String) -> Result<(), &'static str> {
        self.log(format!("STEP fell back to argument {:?}", token));
        self.state.args.push(token.clone());
        return Ok(());
    }

    fn match_mode_flagarg(&mut self, token: &String) {
        // Set mode to subcommand and put string in flag_args
        let mut mode = MachineStateMode::Subcommand;
        swap(&mut mode, &mut self.state.mode);
        if let MachineStateMode::Flagarg { current_flag } = mode {
            self.state.flag_args.insert(current_flag, token.clone());
        } else {
            unreachable!();
        }
    }

    fn log(&self, msg: String) {
        if self.log {
          println!("{}; current state: {:?}", msg, self.state);
        }
    }

    /// Call this after machine is done to morph into a result
    pub fn to_result(self) -> TabryResult {
      TabryResult::new(self.config, self.state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use assert_json_diff::assert_json_eq;
    use serde::Deserialize;

    fn load_fixture_file<T: for<'a>Deserialize<'a>>(filename: &str) -> T {
        let file_str = fs::read_to_string(format!("fixtures/{filename}")).unwrap();
        serde_json::from_str::<T>(&file_str).unwrap()
    }

    fn add_expectation_defaults(mut expectation: serde_json::Value) -> serde_json::Value {
        // base is the default object, except no flag_args (expectation file merges "flags" and
        // "flag_args")
        let default_machine_state: MachineState = Default::default();
        let mut base = serde_json::value::to_value(default_machine_state).unwrap();
        let base_obj = base.as_object_mut().unwrap();
        base_obj.remove("flag_args");

        let to_add = expectation.as_object_mut().unwrap();
        base_obj.append(to_add);

        base
    }

    // Expectations file has "flags" with boolean and string values, that is, flag_args and flags
    // combined.
    fn merge_flags_and_flag_args(machine_state_as_serde_value: &mut serde_json::Value) {
        let val_as_obj = machine_state_as_serde_value.as_object_mut().unwrap();
        let mut flag_args = val_as_obj.get("flag_args").unwrap().as_object().unwrap().clone();
        val_as_obj.get_mut("flags").unwrap().as_object_mut().unwrap().append(&mut flag_args);
        val_as_obj.remove_entry("flag_args");
    }

    // TODO it would be nice to split this up into multiple test so it doesn't fail immediately,
    // but I don't know how to do that with the current test framework
    #[test]
    fn test_all_expectations() {
        // load fixture files
        let tabry_conf: types::TabryConf = load_fixture_file("vehicles.json");
        let expectations: serde_json::Value = load_fixture_file("vehicles-expectations.json");

        for (name, test_case) in expectations.as_object().unwrap() {
            println!("TESTING TEST CASE {name}");
            let mut machine = Machine::new(tabry_conf.clone());
            // test_case is an array with 1) the tokens and 2) the expected state
            let tokens = test_case[0].as_array().unwrap();
            let expected_state = add_expectation_defaults(test_case[1].clone());

            // loop over tokens:
            for token in tokens {
                machine.next(&token.as_str().unwrap().to_string()).unwrap();
            }


            let machine_state_as_serde_value = &mut serde_json::value::to_value(&machine.state).unwrap();
            merge_flags_and_flag_args(machine_state_as_serde_value);

            assert_json_eq!(machine_state_as_serde_value, expected_state);
        }
    }

    #[test]
    fn test_missing_include() {
      // let tabry_conf: types::TabryConf = load_fixture_file("missing_include.json");
      // TODO
      unimplemented!();
    }
}


