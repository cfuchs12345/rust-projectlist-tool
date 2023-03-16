

use serde::{Deserialize, Serialize};

use crate::project::ProjectTuple;


#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct ProjectList {
    pub list: Vec<ProjectTuple>
}