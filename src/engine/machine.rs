use std::mem::swap;

use crate::core::config::TabryConf;
use crate::core::config::TabryConfError;
use crate::core::util::is_debug;

use super::machine_state::{MachineState, MachineStateMode};
use super::token_matching::TokenMatching;

use super::result::TabryResult;

/// The state machine responsible for parsing command line arguments and identifying
/// subcommands, flags, and positional arguments.
pub struct Machine {
    config: TabryConf,
    pub state: MachineState,
    log: bool,
}

impl Machine {
    pub fn new(config: TabryConf) -> Machine {
        Machine {
            config,
            state: MachineState::default(),
            log: is_debug(),
        }
    }

    /// Parse a command line. Specifically, given a TabryConf and token, build and run the state machine and return the TabryResult. Equivalent to `new()` + `next()` for each token + `to_result()`.
    pub fn run(config: TabryConf, tokens: &[String]) -> Result<TabryResult, TabryConfError> {
        let mut this = Self::new(config);
        for token in tokens {
            this.next(token)?;
        }
        Ok(this.to_result())
    }

    /// Feed the state machine one token.
    pub fn next(&mut self, token: &String) -> Result<(), TabryConfError> {
        match self.state.mode {
            MachineStateMode::Subcommand => self.match_mode_subcommand(token),
            MachineStateMode::Flagarg { .. } => {
                self.match_mode_flagarg(token);
                Ok(())
            }
        }
    }

    fn match_mode_subcommand(&mut self, token: &String) -> Result<(), TabryConfError> {
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
    fn current_sub(&mut self) -> Result<&types::TabryConcreteSub, TabryConfError> {
        self.config.dig_sub(&self.state.subcommand_stack)
    }
    */

    fn match_subcommand(&mut self, token: &String) -> Result<bool, TabryConfError> {
        if !self.state.args.is_empty() {
            return Ok(false);
        }

        // TODO using self.current_sub() causes weird borrow problem. But also want t
        // make self.find_in_subs etc. be able to mutate self which it can't right now
        // due to weird lifetime problem.
        let sub_here = self.config.dig_sub(&self.state.subcommand_stack)?;

        if let Some(sub) = self.config.find_in_subs(&sub_here.subs, &sub_here.includes, token, true)? {
            let name = TabryConf::unwrap_sub_name(sub)?;
            self.state.subcommand_stack.push(name.to_owned());
            self.log(format!("STEP subcommand, add {}", name));
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

    fn match_flag(&mut self, token: &str) -> Result<bool, TabryConfError> {
        if self.state.dashdash {
            return Ok(false);
        }

        // Check flags for each Subcommand in stack, starting with the most specific Subcommand.
        for sub in self
            .config
            .dig_subs(&self.state.subcommand_stack)?
            .iter()
            .rev()
        {
            for flag in self.config.expand_flags(&sub.flags, &sub.includes) {
                if flag.match_token(token) {
                    if flag.arg {
                        self.state.mode = MachineStateMode::Flagarg {
                            current_flag: flag.name.clone(),
                        }
                    } else {
                        self.state.flags.insert(flag.name.clone(), true);
                    }
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn match_help(&mut self, token: &str) -> bool {
        if !self.state.dashdash && (token == "help" || token == "--help" || token == "-?") {
            self.state.help = true;
            true
        } else {
            false
        }
    }

    fn match_arg(&mut self, token: &String) -> Result<(), TabryConfError> {
        self.log(format!("STEP fell back to argument {:?}", token));
        self.state.args.push(token.clone());
        Ok(())
    }

    fn match_mode_flagarg(&mut self, token: &str) {
        // Set mode to subcommand and put string in flag_args
        let mut mode = MachineStateMode::Subcommand;
        swap(&mut mode, &mut self.state.mode);
        if let MachineStateMode::Flagarg { current_flag } = mode {
            self.state.flag_args.insert(current_flag, token.to_owned());
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
    use assert_json_diff::assert_json_eq;

    use crate::test_helpers::load_fixture_file;

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
        let mut flag_args = val_as_obj
            .get("flag_args")
            .unwrap()
            .as_object()
            .unwrap()
            .clone();
        val_as_obj
            .get_mut("flags")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .append(&mut flag_args);
        val_as_obj.remove_entry("flag_args");
    }

    // TODO it would be nice to split this up into multiple test so it doesn't fail immediately,
    // but I don't know how to do that with the current test framework
    #[test]
    fn test_all_expectations() {
        // load fixture files
        let tabry_conf: TabryConf = load_fixture_file("vehicles.json");
        let expectations: serde_json::Value = load_fixture_file("vehicles-expectations.json");

        // TODO figure out how to use name. use a macro here?
        for (_name, test_case) in expectations.as_object().unwrap() {
            let mut machine = Machine::new(tabry_conf.clone());
            // test_case is an array with 1) the tokens and 2) the expected state
            let tokens = test_case[0].as_array().unwrap();
            let expected_state = add_expectation_defaults(test_case[1].clone());

            // loop over tokens:
            for token in tokens {
                machine.next(&token.as_str().unwrap().to_string()).unwrap();
            }

            let machine_state_as_serde_value =
                &mut serde_json::value::to_value(&machine.state).unwrap();
            merge_flags_and_flag_args(machine_state_as_serde_value);

            assert_json_eq!(machine_state_as_serde_value, expected_state);
        }
    }

    #[test]
    fn test_missing_include() {
        let tabry_conf: TabryConf = load_fixture_file("missing_include.json");
        let mut machine = Machine::new(tabry_conf.clone());
        machine.next(&"foo".to_owned()).unwrap();
        let result = machine.next(&"bar".to_owned());
        assert!(matches!(result, Err(TabryConfError::MissingInclude { .. })));
    }
}
