

use serde::{Deserialize, Serialize};

use crate::project::Model as Project;
use crate::client::Model as Client;
use crate::role::Model as Role;
use crate::person::Model as Person;
use crate::technology::Model as Technology;
use crate::businessarea::Model as Businessarea;


#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize, Default)]
pub struct ProjectList {
    pub list: Vec<(Project, Vec<Client>,Vec<Businessarea>, Vec<Role>, Vec<Person>, Vec<Technology>)>
}