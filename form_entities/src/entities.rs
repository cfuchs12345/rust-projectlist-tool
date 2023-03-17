use entities::project::ProjectAndDependencies;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ClientWithBusinessArea {
    pub id: String,
    pub name: String,
    pub businessarea_id: i16
}
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectFormInput {
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
impl ProjectFormInput {
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
    if id_list.is_empty() {
        return Vec::new();
    }
    id_list
        .split(',')
        .map(|s| -> i16 { s.trim().parse().unwrap()})
        .collect()
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_to_int_normal() {
        assert_eq!(split_to_int("1,2,3,4,5".to_string()), vec![1,2,3,4,5]);
    }

    #[test]
    fn test_split_to_int_trim() {
        assert_eq!(split_to_int("1, 2, 3, 4 ,5 ".to_string()), vec![1,2,3,4,5]);
    }

    #[test]
    #[should_panic]
    fn test_split_to_int_panic_when_no_digit() { // with chars in it
        assert_eq!(split_to_int("1, a, 3, 4 ,5 ".to_string()), vec![1,2,3,4,5]);
    }

    #[test]
    fn test_projectwithrelatedentites_to_project_and_dependencies() {
        let testee = ProjectFormInput { // like it would be filled from an input form
            id: "0".to_string(),
            description_de: "desc de".to_string(),
            description_en: "desc en".to_string(),
            summary_de: "summary de".to_string(),
            summary_en: "summary en".to_string(),
            from: "2023-03".to_string(),
            to: "2023-04".to_string(),
            duration: "not set".to_string(),
            businessarea_ids: "1,2".to_string(),
            client_ids: "3,4".to_string(),
            person_ids: "123".to_string(),
            role_ids: "3456,789".to_string(),
            technology_ids: "9876".to_string()

        };
        let expected = ProjectAndDependencies  {
            project: entities::project::Model {
                id: 0,
                description_de: "desc de".to_string(),
                description_en: "desc en".to_string(),
                summary_de: "summary de".to_string(),
                summary_en: "summary en".to_string(),
                from: "2023-03".to_string(),
                to: "2023-04".to_string(),
                duration: "not set".to_string()
            },
            businessareas: vec![1, 2],
            clients:  vec![3, 4],
            persons: vec![123],
            roles: vec![3456,789],
            technologies: vec![9876]           
        };
        assert_eq!(testee.to_project_and_dependencies(), expected);
    }
}

