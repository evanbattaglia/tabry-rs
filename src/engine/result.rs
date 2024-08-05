use crate::core::{
    config::TabryConf,
    types::TabryConcreteSub,
};
use super::machine_state::MachineState;


/// Encapsulates a TabryConfig and a TabryMachineState state, and provides
/// functionality relating to this state.
pub struct TabryResult {
    pub config: TabryConf,
    pub state: MachineState,

    // TODO: tried to make it a reference but got into a named lifetime mess... not sure why
    // when it only references things in TabryConf
    pub sub_stack: Vec<TabryConcreteSub>,
}

impl TabryResult {
    pub fn new(config: TabryConf, state: MachineState) -> Self {
        let sub_stack = config.dig_subs(&state.subcommand_stack).unwrap().into_iter().cloned().collect();
        TabryResult { config, state, sub_stack }
    }

    pub fn current_sub(&self) -> &TabryConcreteSub {
        self.sub_stack.last().unwrap()
    }
}
