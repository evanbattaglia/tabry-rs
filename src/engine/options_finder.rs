use super::{machine_state::MachineStateMode, result::TabryResult};
use crate::core::config::TabryConfError;
use crate::core::types::{TabryConcreteArg, TabryConcreteFlag, TabryOpt};
use std::collections::HashSet;
use std::process::Command;

use serde_json::json;

pub struct OptionsFinder {
    result: TabryResult,
}

pub struct OptionsResults {
    prefix: String,
    pub options: HashSet<String>,
    pub special_options: HashSet<String>,
}

impl OptionsResults {
    fn insert(&mut self, value: &str) {
        if value.starts_with(&self.prefix) {
            // TODO get_or_insert_owned() in nightly would be ideal
            self.options.insert(value.to_owned());
        }
    }

    fn insert_special(&mut self, value: &str) {
        self.special_options.insert(value.to_owned());
    }
}

impl OptionsFinder {
    pub fn new(result: TabryResult) -> Self {
        Self { result }
    }

    pub fn options(&self, token: &str) -> Result<OptionsResults, TabryConfError> {
        let mut res = OptionsResults {
            prefix: token.to_owned(),
            options: HashSet::new(),
            special_options: HashSet::new(),
        };

        match self.result.state.mode {
            MachineStateMode::Subcommand => self.add_options_subcommand(&mut res)?,
            MachineStateMode::Flagarg { .. } => self.add_options_flagarg(&mut res)?,
        };

        Ok(res)
    }

    fn add_options_subcommand(&self, res: &mut OptionsResults) -> Result<(), TabryConfError> {
        // TODO: required flags
        self.add_options_subcommand_subs(res);
        self.add_options_subcommand_flags(res)?;
        self.add_options_subcommand_args(res)?;

        Ok(())
    }

    fn add_options_subcommand_subs(&self, res: &mut OptionsResults) {
        // once arg has been given, can no longer use a subcommand
        if !self.result.state.args.is_empty() {
            return;
        }

        let opaque_subs = &self.result.current_sub().subs;
        let concrete_subs = self.result.config.flatten_subs(opaque_subs).unwrap();
        for s in concrete_subs {
            // TODO: error here if no name -- only allowable for top level
            res.insert(s.name.as_ref().unwrap());
        }
    }

    fn flag_is_used(&self, flag: &TabryConcreteFlag) -> bool {
        self.result.state.flags.contains_key(&flag.name)
            || self.result.state.flag_args.contains_key(&flag.name)
    }

    fn add_option_for_flag(res: &mut OptionsResults, flag: &TabryConcreteFlag) {
        let flag_str = if flag.name.len() == 1 {
            format!("-{}", flag.name)
        } else {
            format!("--{}", flag.name)
        };
        res.insert(&flag_str);
    }

    fn add_options_subcommand_flags(&self, res: &mut OptionsResults) -> Result<(), TabryConfError> {
        if self.result.state.dashdash {
            return Ok(());
        }

        let mut current_sub_flags = self
            .result
            .config
            .expand_flags(&self.result.current_sub().flags);
        let first_reqd_flag = current_sub_flags.find(|f| f.required && !self.flag_is_used(f));
        if let Some(first_reqd_flag) = first_reqd_flag {
            Self::add_option_for_flag(res, first_reqd_flag);
            return Ok(());
        }

        // Don't suggest flags unless user has typed a dash
        if !res.prefix.starts_with('-') {
            return Ok(());
        }

        for sub in self.result.sub_stack.iter() {
            for flag in self.result.config.expand_flags(&sub.flags) {
                if !self.flag_is_used(flag) {
                    Self::add_option_for_flag(res, flag);
                }
            }
        }

        Ok(())
    }

    fn add_options(
        &self,
        res: &mut OptionsResults,
        options: &Vec<TabryOpt>,
    ) -> Result<(), TabryConfError> {
        for opt in options {
            match &opt {
                TabryOpt::File => res.insert_special("file"),
                TabryOpt::Dir => res.insert_special("dir"),
                TabryOpt::Const { value } => res.insert(value),
                TabryOpt::Delegate { value } => {
                    res.insert_special(format!("delegate {}", value).as_str())
                }
                TabryOpt::Shell { value } => {
                    let auto_complete_state = json!({
                        "cmd": self.result.config.cmd,
                        "flags": self.result.state.flags,
                        // TODO: these are merged in ruby version.
                        "flag_args": self.result.state.flag_args,
                        "args": self.result.state.args,
                        // current_token. result.prefix???
                        // "current_flag": self.result.state.current_flag,
                        // ^ this doesn't seem to exist either for the rust version?
                    });
                    let output = Command::new("sh")
                        .arg("-c")
                        .arg(value)
                        .env("TABRY_AUTOCOMPLETE_STATE", auto_complete_state.to_string())
                        .output();
                    // TODO bubble up errors instead on unwrap()
                    let output_bytes = output.unwrap();
                    let output_str = std::str::from_utf8(&output_bytes.stdout[..]).unwrap();
                    for line in output_str.split('\n') {
                        if !line.is_empty() {
                            res.insert(line);
                        }
                    }
                }
                TabryOpt::Include { value } => {
                    // TODO: what happens if there is an include loop?
                    self.add_options(res, self.result.config.get_option_include(value)?)?;
                }
            }
        }
        Ok(())
    }

    fn add_options_subcommand_args(&self, res: &mut OptionsResults) -> Result<(), TabryConfError> {
        let sub_args = self
            .result
            .config
            .expand_args(&self.result.current_sub().args)
            .collect::<Vec<_>>();

        if let Some(arg) = sub_args.get(self.result.state.args.len()) {
            self.add_options(res, &arg.options)?;
        } else if let Some(TabryConcreteArg {
            varargs: true,
            options,
            ..
        }) = sub_args.last()
        {
            self.add_options(res, options)?;
        }

        Ok(())
    }

    fn add_options_flagarg(&self, res: &mut OptionsResults) -> Result<(), TabryConfError> {
        let MachineStateMode::Flagarg { current_flag } = &self.result.state.mode else {
            unreachable!()
        };
        for sub in &self.result.sub_stack {
            for flag in self.result.config.expand_flags(&sub.flags) {
                if &flag.name == current_flag {
                    self.add_options(res, &flag.options)?;
                    return Ok(());
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::TabryConf;
    use crate::engine::machine_state::{MachineState, MachineStateMode::*};
    use crate::test_helpers::load_fixture_file;
    use std::collections::HashMap;
    // TODO fill in from ~/dev/tabry/spec/tabry/options_finder_spec.rb

    fn options_with_machine_state(machine_state: MachineState, token: &str) -> OptionsResults {
        let tabry_conf: TabryConf = load_fixture_file("vehicles.json");
        let tabry_result = TabryResult::new(tabry_conf, machine_state);
        let options_finder = OptionsFinder::new(tabry_result);
        options_finder.options(token).unwrap()
    }

    macro_rules! token_or_default(
        ($token:expr) => { $token };
        () => { "" };
    );

    macro_rules! test_options_finder(
        (
            $name:ident,
            (
                $($expected:expr),*
                $(; $($expected_special_options:expr),*)?
            ),
            {$($machine_state_key:ident : $machine_state_val:expr),*}
            $(, $token:expr)?
         ) => {
            #[test]
            fn $name() {
                let token = token_or_default!($($token)?);
                let machine_state = MachineState {
                    $($machine_state_key : $machine_state_val ,)*
                    ..Default::default()
                };
                let options_results = options_with_machine_state(machine_state, token);
                let actual_strs : HashSet<&str> =
                    options_results.options.iter().map(|s| s.as_str()).collect();
                let actual_specials_strs : HashSet<&str> =
                    options_results.special_options.iter().map(|s| s.as_str()).collect();

                let expected = [$($expected),*];
                let expected_special_options = [$($($expected_special_options),*)?];

                assert_eq!(actual_strs, HashSet::from(expected));
                assert_eq!(actual_specials_strs, HashSet::from(expected_special_options));
            }
        };
    );

    macro_rules! vec_owned {
        ($($str:expr),*) => {
            (vec![$($str.to_owned()),*])
        };
    }

    macro_rules! hashmap_owned {
        ($($k:expr => $v:expr),* $(,)?) => {{
            HashMap::from([$(($k.to_owned(), $v.to_owned()),)*])
        }};
    }

    test_options_finder!(
        test_possible_subcommands_of_the_main_command,
        (
            "build",
            "list-vehicles",
            "move",
            "sub-with-sub-or-arg",
            "sub-with-sub-or-opt-arg",
            "sub-with-mandatory-flag"
        ),
        {}
    );

    test_options_finder!(
        test_possible_subcommands_of_a_subcommand,
        ("go", "stop", "crash", "freeway-crash"),
        {subcommand_stack: vec_owned!("move")}
    );

    test_options_finder!(
        test_lists_possible_arguments_const,
        ("car","bike"),
        {subcommand_stack: vec_owned!("move", "go")}
    );

    test_options_finder!(
        test_lists_options_for_varargs,
        ("car", "bike"),
        {subcommand_stack: vec_owned!("build")}
    );

    test_options_finder!(
        test_lists_both_possible_args_and_subcommand_stack_if_a_subcommand_can_take_either,
        ("x","y","z","subsub"),
        {subcommand_stack: vec_owned!("sub-with-sub-or-arg")}
    );

    test_options_finder!(
        test_lists_possible_flags_if_the_last_token_starts_with_a_dash,
        ("--verbose", "--speed", "--output-to-file", "--output-to-directory", "--dry-run"),
        {subcommand_stack: vec_owned!("move", "crash")},
        "-"
    );

    test_options_finder!(
        test_doesn_t_list_a_flag_if_it_has_already_been_given,
        ("--verbose","--speed","--output-to-file","--output-to-directory"),
        {
          flags: hashmap_owned!("dry-run" => true),
          subcommand_stack: vec_owned!("move", "crash")
        },
        "-"
    );

    test_options_finder!(
        test_doesnt_suggests_flags_if_double_dash_has_been_used,
        (),
        {
            dashdash: true,
            subcommand_stack: vec_owned!("move", "crash")
        },
        "-"
    );

    // TODO
    // test_options_finder!(
    //     test_lists_only_a_mandatory_flag_if_it_hasnt_been_given_yet,
    //     ("--mandatory"),
    //     {subcommand_stack: vec_owned!("sub-with-mandatory-flag")}
    // );

    test_options_finder!(
        test_lists_other_args_after_a_mandatory_flag_has_been_given,
        ("a","b","c"),
        {
          subcommand_stack: vec_owned!("sub-with-mandatory-flag"),
          flag_args: hashmap_owned!("mandatory" => "foo")
        }
    );

    test_options_finder!(
        test_lists_possibilities_for_a_flag_arguments_shell,
        ("fast","slow"),
        {
            subcommand_stack: vec_owned!("move", "crash"),
            mode: Flagarg { current_flag: "speed".to_owned() }
        }
    );

    test_options_finder!(
        test_lists_possibilities_for_a_flag_arguments_file_const,
        ("-"; "file"), // special "file" option
        {
          subcommand_stack: vec_owned!("move", "crash"),
          mode: Flagarg { current_flag: "output-to-file".to_owned() }
        }
    );

    test_options_finder!(
        test_lists_possibilities_for_a_flag_arguments_dir,
        (; "dir"), // special "dir" option
        {
          subcommand_stack: vec_owned!("move", "crash"),
          mode: Flagarg { current_flag: "output-to-directory".to_owned() }
        }
    );

    test_options_finder!(
        test_lists_nothing_if_no_options_are_defined,
        (),
        {
            subcommand_stack: vec_owned!("sub-with-sub-or-arg"),
            mode: Flagarg { current_flag: "mandatory".to_owned() }
        }
    );
}
