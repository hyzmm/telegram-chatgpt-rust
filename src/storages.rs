use std::collections::HashMap;
use std::fs::File;
use std::io::Read;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Role {
    pub system: String,
}

pub type Roles = HashMap<String, Role>;

pub fn get_roles() -> Result<Roles, anyhow::Error> {
    let mut file = File::open("storage/roles.yaml").context("Cannot find file 'storage/roles'")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    serde_yaml::from_str(contents.as_str()).context("Cannot deserialize file 'storage/roles'")
}
