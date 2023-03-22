use serde::{Deserialize, Serialize};
use std::collections::HashMap; // 1.0.124

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TabryOpt {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "dir")]
    Dir,
    #[serde(rename = "const")]
    Const { value: String },
    #[serde(rename = "shell")]
    Shell { value: String },
    #[serde(rename = "include")]
    Include { value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TabryArg {
    TabryIncludeArg {
        include: String,
    },
    TabryConcreteArg {
        name: Option<String>,
        #[serde(default)]
        options: Vec<TabryOpt>,
        #[serde(default)]
        optional: bool,
        #[serde(default)]
        varargs: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TabryFlag {
    TabryIncludeFlag {
        include: String,
    },
    TabryConcreteFlag {
        name: String,
        #[serde(default)]
        aliases: Vec<String>,
        #[serde(default)]
        options: Vec<TabryOpt>,
        // TODO: could break up into flagarg and regular flag
        #[serde(default)]
        arg: bool,
        #[serde(default)]
        required: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabryConcreteSub {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub args: Vec<TabryArg>,
    #[serde(default)]
    pub flags: Vec<TabryFlag>,
    #[serde(default)]
    pub subs: Vec<TabrySub>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TabrySub {
    TabryIncludeArg { include: String },
    TabryConcreteSub(TabryConcreteSub),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabryArgInclude {
    #[serde(default)]
    pub args: Vec<TabryArg>,
    #[serde(default)]
    pub flags: Vec<TabryFlag>,
    #[serde(default)]
    pub subs: Vec<TabrySub>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabryConf {
    pub cmd: String,
    pub main: TabryConcreteSub,
    #[serde(default)]
    pub arg_includes: HashMap<String, TabryArgInclude>,
    #[serde(default)]
    pub option_includes: HashMap<String, Vec<TabryOpt>>,
}
