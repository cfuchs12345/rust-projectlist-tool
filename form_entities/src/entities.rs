use entities::project::ProjectAndDependencies;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ClientWithBusinessArea {
    pub id: String,
    pub name: String,
    pub businessarea_id: i16
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectWithRelatedEntites {
    pub id: String,
    pub summary_de: String,
    pub summary_en: String,
    pub description_de: String,
    pub description_en: String,
    pub client_ids: String,
    pub businessarea_ids: String,
    pub person_ids: String,
    pub role_ids: String,
    pub technology_ids: String,
    pub duration: String,
    pub from: String,
    pub to: String,
}
impl ProjectWithRelatedEntites {
    pub fn to_project_and_dependencies(&self) -> ProjectAndDependencies {
        let link = &self;

        ProjectAndDependencies {            
            project: entities::project::Model {
                id: self.id.parse().unwrap_or_default(),
                summary_de: link.summary_de.clone(),
                summary_en:link.summary_en.clone(),
                description_de: link.description_de.clone(),
                description_en: link.description_en.clone(),
                from: link.from.clone(),
                to: link.to.clone(),
                duration: link.duration.clone()
            },
            clients: split_to_int(link.client_ids.clone()),
            businessareas: split_to_int(link.businessarea_ids.clone()),
            roles: split_to_int(link.role_ids.clone()),
            persons: split_to_int(link.person_ids.clone()),
            technologies: split_to_int(link.technology_ids.clone())
        }
    }
}

fn split_to_int(id_list: String) -> Vec<i16> {
    if id_list.trim().len() == 0 {
        return Vec::new();
    }
    id_list
        .split(",")
        .map(|s| -> i16 { s.trim().parse().unwrap_or_default()})
        .collect()
}