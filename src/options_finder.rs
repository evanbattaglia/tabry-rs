use super::machine_state::MachineStateMode;
use super::result::TabryResult;
use super::types::TabryOpt;
use super::types::TabryConcreteArg;
use super::types::TabryConcreteFlag;
use super::config::TabryConfError;
use std::process::Command;
use std::collections::HashSet;

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
            prefix: token.to_owned(), options: HashSet::new(), special_options: HashSet::new()
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
            return
        }

        let opaque_subs = &self.result.current_sub().subs;
        let concrete_subs = self.result.config.flatten_subs(&opaque_subs).unwrap();
        for s in concrete_subs {
            // TODO: error here if no name -- only allowable for top level
            res.insert(s.name.as_ref().unwrap());
        }
    }

    fn flag_is_used(&self, flag: &TabryConcreteFlag) -> bool {
        self.result.state.flags.contains_key(&flag.name) ||
            self.result.state.flag_args.contains_key(&flag.name)
    }

    fn add_option_for_flag(res: &mut OptionsResults, flag: &TabryConcreteFlag) {
        let flag_str =
            if flag.name.len() == 1 {
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

        let mut current_sub_flags = self.result.config.expand_flags(&self.result.current_sub().flags);
        let first_reqd_flag = current_sub_flags.find(|f| f.required && !self.flag_is_used(f));
        if let Some(first_reqd_flag) = first_reqd_flag {
            Self::add_option_for_flag(res, first_reqd_flag);
            return Ok(());
        }

        // Don't suggest flags unless user has typed a dash
        if !res.prefix.starts_with("-") {
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

    fn add_options(&self, res: &mut OptionsResults, options: &Vec<TabryOpt>) -> Result<(), TabryConfError> {
        for opt in options.iter() {
            match &opt {
                TabryOpt::File => res.insert_special("file"),
                TabryOpt::Dir => res.insert_special("dir"),
                TabryOpt::Const { value } => res.insert(value),
                TabryOpt::Shell { value } => {
                    let output = Command::new("sh").arg("-c").arg(value).output();
                    // TODO bubble up errors instead on unwrap()
                    let output_bytes = output.unwrap();
                    let output_str = std::str::from_utf8(&output_bytes.stdout[..]).unwrap();
                    for line in output_str.split("\n") {
                        if line != "" {
                            res.insert(line);
                        }
                    }
                },
                TabryOpt::Include { value } => {
                    // TODO: what happens if there is an include loop?
                    self.add_options(res, self.result.config.get_option_include(value)?)?;
                }
            }
        }
        Ok(())
    }

    fn add_options_subcommand_args(&self, res: &mut OptionsResults) -> Result<(), TabryConfError> {
        let sub_args = self.result.config.expand_args(&self.result.current_sub().args).collect::<Vec<_>>();

        if let Some(arg) = sub_args.get(self.result.state.args.len()) {
            self.add_options(res, &arg.options)?;
        } else if let Some(TabryConcreteArg{varargs: true, options, ..}) = sub_args.last() {
            self.add_options(res, &options)?;
        }

        Ok(())
    }

    fn add_options_flagarg(&self, res: &mut OptionsResults) -> Result<(), TabryConfError> {
        let MachineStateMode::Flagarg { current_flag } = &self.result.state.mode else { unreachable!() };
        for sub in self.result.sub_stack.iter() {
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
