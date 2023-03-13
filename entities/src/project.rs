use serde::{Deserialize, Serialize};
use sea_orm::entity::prelude::*;

use crate::common;


#[derive(Clone, Hash, Debug, PartialEq, Eq, DeriveEntityModel, Deserialize, Serialize, Default)]
#[sea_orm(table_name = "project")]
pub struct Model {
    #[serde(default)]
    #[sea_orm(primary_key)]
    pub id: i16,
    #[sea_orm(column_type = "Text")]
    pub summary_de: String,
    #[sea_orm(column_type = "Text")]
    pub summary_en: String,
    #[sea_orm(column_type = "Text")]
    pub description_de: String,
    #[sea_orm(column_type = "Text")]
    pub description_en: String,
    #[sea_orm(column_type = "Text")]
    pub duration: String,
    #[sea_orm(column_type = "Text")]
    pub from: String,
    #[sea_orm(column_type = "Text")]
    pub to: String
}


#[derive(Default, Hash, Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct ProjectAndDependencies {
    pub project: Model,
    pub clients: Vec<i16>,
    pub businessareas: Vec<i16>,
    pub roles: Vec<i16>,
    pub persons: Vec<i16>,
    pub technologies: Vec<i16>
}

impl common::DBEntity for Model {
    fn get_id(&self) -> &i16 {
        &self.id
    }
}


#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
}



impl Related<super::client::Entity> for Entity {
    fn to() -> RelationDef {
        super::project_client::Relation::Client.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::project_client::Relation::Project.def().rev())
    }
}

impl Related<super::businessarea::Entity> for Entity {
    fn to() -> RelationDef {
        super::project_businessarea::Relation::Businessarea.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::project_businessarea::Relation::Project.def().rev())
    }
}

impl Related<super::person::Entity> for Entity {
    fn to() -> RelationDef {
        super::project_person::Relation::Person.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::project_person::Relation::Project.def().rev())
    }
}

impl Related<super::role::Entity> for Entity {
    fn to() -> RelationDef {
        super::project_role::Relation::Role.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::project_role::Relation::Project.def().rev())
    }
}

impl Related<super::technology::Entity> for Entity {
    fn to() -> RelationDef {
        super::project_technology::Relation::Technology.def()
    }

    fn via() -> Option<RelationDef> {
        Some(super::project_technology::Relation::Project.def().rev())
    }
}



#[async_trait::async_trait]
impl ActiveModelBehavior for ActiveModel {
    async fn before_save<C>(self, _db: &C, _insert: bool) -> Result<Self, DbErr>
    where
        C: ConnectionTrait,
    {
        Ok(self)
    }
}