use super::machine_state::MachineStateMode;
use super::result::TabryResult;

pub struct OptionsFinder {
    result: TabryResult,
}

impl OptionsFinder {
    pub fn new(result: TabryResult) -> Self {
        Self { result }
    }

    pub fn options(&self, token: &str) -> Vec<String> {
        match self.result.state.mode {
            MachineStateMode::Subcommand => self.options_subcommand(token),
            MachineStateMode::Flagarg { .. } => self.options_flagarg(token),
        }
    }

    fn options_subcommand(&self, token: &str) -> Vec<String> {
        let mut res = vec![];
        // TODO: required flags
        self.add_options_subcommand_subs(&mut res, token);
        self.add_options_subcommand_flags(&mut res, token);
        self.add_options_subcommand_args(&mut res, token);

        res
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

    fn add_options_subcommand_flags(&self, buffer: &mut Vec<String>, token: &str) {
        // TODO
    }

    fn add_options_subcommand_args(&self, buffer: &mut Vec<String>, token: &str) {
        //self.result.sub_stack.iter().map (|sub| 
        // TODO
    }

    fn options_flagarg(&self, token: &str) -> Vec<String> {
        let MachineStateMode::Flagarg { current_flag } = &self.result.state.mode else { unreachable!() };
        vec![]
    }
}
