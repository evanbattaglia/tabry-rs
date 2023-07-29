use super::machine_state::MachineStateMode;
use super::result::TabryResult;
use super::types::TabryOpt;
use super::config::TabryConfError;

pub struct OptionsFinder {
    result: TabryResult,
}

impl OptionsFinder {
    pub fn new(result: TabryResult) -> Self {
        Self { result }
    }

    pub fn options(&self, token: &str) -> Result<Vec<String>, TabryConfError> {
        match self.result.state.mode {
            MachineStateMode::Subcommand => self.options_subcommand(token),
            MachineStateMode::Flagarg { .. } => self.options_flagarg(token),
        }
    }

    fn options_subcommand(&self, token: &str) -> Result<Vec<String>, TabryConfError> {
        let mut res = vec![];
        // TODO: required flags
        self.add_options_subcommand_subs(&mut res, token);
        self.add_options_subcommand_flags(&mut res, token)?;
        self.add_options_subcommand_args(&mut res, token);

        // TODO: uniqify options
        Ok(res)
    }

    fn add_options_subcommand_subs(&self, buffer: &mut Vec<String>, token: &str) {
        // once arg has been given, can no longer use a subcommand
        if !self.result.state.args.is_empty() {
            return
        }

        let opaque_subs = &self.result.current_sub().subs;
        let concrete_subs = self.result.config.flatten_subs(&opaque_subs).unwrap();
        for s in concrete_subs {
            // TODO: error here if no name -- only allowable for top level
            let name = s.name.as_ref().unwrap();
            if name.starts_with(token) {
                buffer.push(s.name.as_ref().unwrap().clone());
            }
        }
    }

    fn add_options_subcommand_flags(&self, buffer: &mut Vec<String>, token: &str) -> Result<(), TabryConfError> {
        if self.result.state.dashdash {
            return Ok(());
        }

        for sub in self.result.sub_stack.iter() {
            for flag in self.result.config.expand_flags(&self.result.current_sub().flags) {
                self.add_options(buffer, &flag.options)?;
            }
        }

        Ok(())
    }

    fn add_options(&self, buffer: &mut Vec<String>, options: &Vec<TabryOpt>) -> Result<(), TabryConfError> {
        for opt in options.iter() {
            match opt {
                TabryOpt::File => (),
                TabryOpt::Dir => (),
                TabryOpt::Const { value } => {
                    buffer.push(value.to_owned());
                }
                TabryOpt::Shell { value } => (),
                TabryOpt::Include { value } => {
                    // TODO: what happens if there is an include loop?
                    self.add_options(buffer, self.result.config.get_option_include(value)?);
                }
            }
        }
        Ok(())
    }

    fn add_options_subcommand_args(&self, buffer: &mut Vec<String>, token: &str) {
        //self.result.sub_stack.iter().map (|sub| 
        // TODO
    }

    fn options_flagarg(&self, token: &str) -> Result<Vec<String>, TabryConfError> {
        let MachineStateMode::Flagarg { current_flag } = &self.result.state.mode else { unreachable!() };
        Ok(vec![])
    }
}
