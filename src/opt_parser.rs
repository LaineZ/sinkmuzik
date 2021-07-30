use std::env;

use anyhow::{Result, bail};

const USAGE_MSG: &str = "
USAGE:
    roggen [action: convert, preview] [music directory]
";

pub enum Action {
    Convert,
    Preview
}

pub struct CmdlineOptions {
    pub path: String,
    pub action: Action, 
}

impl CmdlineOptions {
    pub fn new() -> Result<Self> {
        let args: Vec<String> = env::args().collect();

        if args.len() <= 2 {
            bail!(format!("Insufficient number of arguments is provided!{}", USAGE_MSG));
        }

        let act;
        match args[1].to_lowercase().as_str() {
            "convert" => act = Some(Action::Convert),
            "preview" => act = Some(Action::Preview),
            _ => { act = None }
        }

        if let Some(action) = act {
            Ok(Self {
                path: args[2].clone(),
                action
            })
        } else {
            bail!(format!("Unknown option: {}{}", args[1], USAGE_MSG))
        }
    }
}