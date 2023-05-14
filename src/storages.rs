use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use anyhow::Context;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Role {
    pub system: String,
}

pub type Roles = HashMap<String, Role>;

const SAVE_FILE_PATH: &str = "storage/roles.yaml";

pub fn get_roles() -> Result<Roles, anyhow::Error> {
    let mut file = File::open(SAVE_FILE_PATH).context("Cannot find file 'storage/roles'")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    serde_yaml::from_str(contents.as_str()).context("Cannot deserialize file 'storage/roles'")
}

pub fn rewrite_file(roles: &Roles) -> Result<(), anyhow::Error> {
    let mut file = File::create(SAVE_FILE_PATH).context("Cannot create file 'storage/roles'")?;
    let yaml = serde_yaml::to_string(&roles).context("Cannot serialize file 'storage/roles'")?;
    file.write_all(yaml.as_bytes())
        .context("Cannot write file 'storage/roles'")?;
    Ok(())
}
