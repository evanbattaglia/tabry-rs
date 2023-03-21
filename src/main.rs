use serde::{Serialize, Deserialize}; // 1.0.124
use serde_json; // 1.0.64

//

//enum Option {
//    OptionFile(),
//    OptionConst
//}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum Opt {
  #[serde(rename="file")]
  File,
  #[serde(rename="const")]
  Const { value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Arg {
    aliases: Option<Vec<String>>,
    name: String,
    // #[serde(flatten)]
    options: Option<Vec<Opt>>
}


// #[derive(Debug, Clone, Serialize, Deserialize)]
// struct TabryConf {
//     cmd: String,
//     main: TabryCommand,


// }

fn main() {
   let arg_json = r#"{
        "name": "output-to-file",
        "aliases": ["foo"],
        "options": [
          {
            "type": "file"
          },
          {
              "type": "const",
              "value": "foo"
          }
        ]
   }"#;
   let arg: Arg = serde_json::from_str(arg_json).unwrap();
   println!("{}, {:?}", arg.name, arg.options.unwrap()[1]);
}
