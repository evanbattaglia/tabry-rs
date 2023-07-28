use std::fmt;
use std::collections::HashMap;
use serde::ser::{Serialize, SerializeStruct};

#[derive(Debug, Default, PartialEq)]
pub enum MachineStateMode {
    #[default]
    Subcommand,
    Flagarg {
        current_flag: String,
    },
}

impl From<&MachineStateMode> for String {
    fn from(mode: &MachineStateMode) -> Self {
        match mode {
            MachineStateMode::Subcommand => "Subcommand".to_string(),
            MachineStateMode::Flagarg { current_flag } => format!("Flagarg({})", current_flag),
        }
    }
}

#[derive(Default)]
pub struct MachineState {
    pub mode: MachineStateMode,
    pub subcommand_stack: Vec<String>,
    pub flags: HashMap<String, bool>,
    pub flag_args: HashMap<String, String>,
    pub args: Vec<String>,
    pub help: bool,
    pub dashdash: bool,
}

impl Serialize for MachineState {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        let mut n_fields = 7;
        // TODO this seems manual (have to check twice) and rest of this function
        // feels very repetitive
        if let MachineStateMode::Flagarg { .. } = self.mode {
            n_fields += 1;
        }

        let mut state = serializer.serialize_struct("MachineState", n_fields)?;

        if let MachineStateMode::Flagarg { current_flag } = &self.mode {
            state.serialize_field("mode", "flagarg")?;
            state.serialize_field("current_flag", current_flag)?;
        } else {
            state.serialize_field("mode", "subcommand")?;
        }

        state.serialize_field("flags", &self.flags)?;
        state.serialize_field("flag_args", &self.flag_args)?;
        state.serialize_field("args", &self.args)?;
        state.serialize_field("help", &self.help)?;
        state.serialize_field("dashdash", &self.dashdash)?;
        state.serialize_field("subs", &self.subcommand_stack)?;
        state.end()
    }
}

impl fmt::Debug for MachineState {
    // TODO not really fond of this method either...
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = & mut f.debug_struct("MachineState");
        // TODO: macro to add to all if not default
        // except it doesn't work for vec... or bool, if serde_json is used...
        // there might even be another way to just not show defaults
        if !(self.mode == Default::default()) {
            res = res.field("mode", &self.mode);
        }
        if !self.subcommand_stack.is_empty() {
            res = res.field("subcommand_stack", &self.subcommand_stack);
        }
        if !(self.flags == Default::default()) {
            res = res.field("flags", &self.flags);
        }
        if !(self.flag_args == Default::default()) {
            res = res.field("flag_args", &self.flag_args);
        }
        if !self.args.is_empty() {
            res = res.field("args", &self.args);
        }
        if self.help {
            res = res.field("help", &self.help);
        }
        if self.dashdash {
            res = res.field("dashdash", &self.dashdash);
        }
        res.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use assert_json_diff::assert_json_eq;

    #[test]
    fn test_json_representation() {
        let state = MachineState {
            help: true,
            flag_args: HashMap::from([ ("foo".to_owned(), "bar".to_owned()) ]),
            args: vec!["myarg".to_owned()],
            ..Default::default()
        };
        let actual = serde_json::value::to_value(state);
        let expected = serde_json::from_str::<serde_json::Value>(r#"
          {
              "subs": [],
              "help": true,
              "dashdash": false,
              "flag_args": {"foo":"bar"},
              "flags": {},
              "mode": "subcommand",
              "args": ["myarg"]
          }
        "#);
        assert_json_eq!(actual.unwrap(), expected.unwrap());
    }
}
