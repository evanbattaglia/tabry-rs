use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TabryOpt {
    #[serde(rename = "file")]
    File,
    #[serde(rename = "dir")]
    Dir,
    #[serde(rename = "const")]
    Const { value: String },
    #[serde(rename = "delegate")]
    Delegate { value: String },
    #[serde(rename = "shell")]
    Shell { value: String },
    #[serde(rename = "include")]
    Include { value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TabryArg {
    TabryIncludeArg { include: String },
    TabryConcreteArg(TabryConcreteArg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabryConcreteArg {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub options: Vec<TabryOpt>,
    #[serde(default)]
    pub optional: bool,
    #[serde(default)]
    pub varargs: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabryConcreteFlag {
    pub name: String,
    #[serde(default)]
    pub aliases: Vec<String>,
    #[serde(default)]
    pub options: Vec<TabryOpt>,
    pub description: Option<String>,
    // TODO: could break up into flagarg and regular flag
    #[serde(default)]
    pub arg: bool,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TabryFlag {
    TabryIncludeFlag { include: String },
    TabryConcreteFlag(TabryConcreteFlag),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabryConcreteSub {
    pub name: Option<String>,
    #[serde(default)]
    pub aliases: Vec<String>,
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
    TabryIncludeSub { include: String },
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
