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
    name: Option<String>,
    description: Option<String>,
    #[serde(default)]
    args: Vec<TabryArg>,
    #[serde(default)]
    flags: Vec<TabryFlag>,
    #[serde(default)]
    subs: Vec<TabrySub>,
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
    args: Vec<TabryArg>,
    #[serde(default)]
    flags: Vec<TabryFlag>,
    #[serde(default)]
    subs: Vec<TabrySub>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabryConf {
    cmd: String,
    main: TabryConcreteSub,
    #[serde(default)]
    arg_includes: HashMap<String, TabryArgInclude>,
    #[serde(default)]
    option_includes: HashMap<String, Vec<TabryOpt>>,
}
