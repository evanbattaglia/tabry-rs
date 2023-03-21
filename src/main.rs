use serde::{Serialize, Deserialize}; // 1.0.124
use serde_json; // 1.0.64

//

//enum Option {
//    OptionFile(),
//    OptionConst
//}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
enum TabryOpt {
  #[serde(rename="file")]
  File,
  #[serde(rename="file")]
  Dir,
  #[serde(rename="const")]
  Const { value: String },
  #[serde(rename="const")]
  Shell { value: String },
  #[serde(rename="include")]
  Include { value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum TabryArg {
    TabryIncludeArg {
        include: String
    },
    TabryConcreteArg {
        name: Option<String>,
        #[serde(default)]
        options: Vec<TabryOpt>,
        #[serde(default)]
        optional: bool,
        #[serde(default)]
        varargs: bool,
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum TabryFlag {
    TabryIncludeFlag {
        include: String
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
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
enum TabrySub {
    TabryIncludeArg {
        include: String
    },
    TabryConcreteSub {
        name: Option<String>,
        description: Option<String>,
        #[serde(default)]
        args: Vec<TabryArg>,
        #[serde(default)]
        flags: Vec<TabryFlag>,
        #[serde(default)]
        subs: Vec<TabrySub>
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TabryConf {
    cmd: String,
    main: TabrySub,
}

fn main() {
    let conf_json = r#"{
        "cmd": "vehicles",
        "main": {
          "description": "Build and control vehicles",
          "flags": [
            {
              "name": "verbose",
              "aliases": [
                "v"
              ],
              "description": "Give more details in output"
            }
          ],
          "subs": [
            {
              "name": "build",
              "args": [
                {
                  "name": "vehicle-types",
                  "options": [
                    {
                      "type": "include",
                      "value": "vehicle-type-arg"
                    }
                  ],
                  "varargs": true
                }
              ]
            },
            {
              "name": "list-vehicles"
            },
            {
              "name": "move",
              "subs": [
                {
                  "name": "go",
                  "aliases": [
                    "g"
                  ],
                  "args": [
                    {
                      "include": "vehicle-type-arg"
                    }
                  ],
                  "flags": [
                    {
                      "include": "vehicle-type-arg"
                    }
                  ],
                  "subs": [
                    {
                      "include": "vehicle-type-arg"
                    }
                  ]
                },
                {
                  "name": "stop",
                  "aliases": [
                    "s"
                  ],
                  "args": [
                    {
                      "include": "vehicle-type-arg"
                    }
                  ],
                  "flags": [
                    {
                      "include": "vehicle-type-arg"
                    }
                  ],
                  "subs": [
                    {
                      "include": "vehicle-type-arg"
                    }
                  ]
                },
                {
                  "name": "crash",
                  "args": [
                    {
                      "include": "vehicle-type-arg"
                    },
                    {
                      "name": "crash-into-vehicle",
                      "optional": true,
                      "options": [
                        {
                          "type": "include",
                          "value": "vehicle-type-arg"
                        }
                      ],
                      "description": "Crash into another vehicle, default is to crash into a fire hydrant"
                    }
                  ],
                  "flags": [
                    {
                      "include": "vehicle-type-arg"
                    },
                    {
                      "name": "speed",
                      "arg": true,
                      "options": [
                        {
                          "type": "shell",
                          "value": "echo fast && echo slow"
                        }
                      ]
                    },
                    {
                      "name": "dry-run",
                      "description": "Don't actually crash, just simulate it"
                    },
                    {
                      "name": "output-to-file",
                      "aliases": [
                        "f"
                      ],
                      "arg": true,
                      "options": [
                        {
                          "type": "file"
                        },
                        {
                          "type": "const",
                          "value": "-"
                        }
                      ]
                    },
                    {
                      "name": "output-to-directory",
                      "aliases": [
                        "dir",
                        "d"
                      ],
                      "arg": true,
                      "options": [
                        {
                          "type": "dir"
                        }
                      ]
                    }
                  ],
                  "subs": [
                    {
                      "include": "vehicle-type-arg"
                    }
                  ]
                },
                {
                  "name": "freeway-crash",
                  "aliases": [
                    "pileup",
                    "p"
                  ],
                  "args": [
                    {
                      "include": "vehicle-type-arg"
                    },
                    {
                      "name": "crash-into-vehicles",
                      "optional": true,
                      "options": [
                        {
                          "type": "include",
                          "value": "vehicle-type-arg"
                        }
                      ],
                      "varargs": true,
                      "title": "vehicle to crash into",
                      "description": "List of vehicles to crash into. Optional, leave out for a '1 car pileup' -- just crashing into center divider"
                    }
                  ],
                  "flags": [
                    {
                      "include": "vehicle-type-arg"
                    }
                  ],
                  "subs": [
                    {
                      "include": "vehicle-type-arg"
                    }
                  ],
                  "description": "Crash on the freeway (AKA a 'pile up')"
                }
              ]
            },
            {
              "name": "sub-with-sub-or-arg",
              "args": [
                {
                  "options": [
                    {
                      "type": "const",
                      "value": "x"
                    },
                    {
                      "type": "const",
                      "value": "y"
                    },
                    {
                      "type": "const",
                      "value": "z"
                    }
                  ]
                }
              ],
              "subs": [
                {
                  "name": "subsub"
                }
              ]
            },
            {
              "name": "sub-with-sub-or-opt-arg",
              "subs": [
                {
                  "name": "subsub"
                }
              ],
              "args": [
                {
                  "optional": true
                }
              ]
            },
            {
              "name": "sub-with-mandatory-flag",
              "args": [
                {
                  "optional": true,
                  "options": [
                    {
                      "type": "const",
                      "value": "a"
                    },
                    {
                      "type": "const",
                      "value": "b"
                    },
                    {
                      "type": "const",
                      "value": "c"
                    }
                  ]
                }
              ],
              "flags": [
                {
                  "name": "mandatory",
                  "required": true,
                  "arg": true
                },
                {
                  "name": "verbose",
                  "aliases": [
                    "v"
                  ],
                  "arg": true
                }
              ]
            }
          ]
        }
    }"#;
        // ,
        // "arg_includes": {
        //   "vehicle-type-arg": {
        //     "args": [
        //       {
        //         "name": "vehicle-type",
        //         "options": [
        //           {
        //             "type": "include",
        //             "value": "vehicle-type"
        //           }
        //         ]
        //       }
        //     ]
        //   }
        // },
        // "option_includes": {
        //   "vehicle-type": [
        //     {
        //       "type": "const",
        //       "value": "car"
        //     },
        //     {
        //       "type": "const",
        //       "value": "bike"
        //     }
        //   ]
        // }
    let conf: TabryConf = serde_json::from_str(conf_json).unwrap();

}
