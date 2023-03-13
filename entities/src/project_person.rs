use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "project_person"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub project_id: i16,
    pub person_id: i16,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    ProjectId,
    PersonId,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    ProjectId,
    PersonId,
}

impl PrimaryKeyTrait for PrimaryKey {
    type ValueType = (i16, i16);

    fn auto_increment() -> bool {
        false
    }
}

impl ColumnTrait for Column {
    type EntityName = Entity;

    fn def(&self) -> ColumnDef {
        match self {
            Self::ProjectId => ColumnType::Integer.def(),
            Self::PersonId => ColumnType::Integer.def(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}


#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Project,
    Person,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Project => Entity::belongs_to(super::project::Entity)
                .from(Column::ProjectId)
                .to(super::project::Column::Id)
                .into(),
            Self::Person => Entity::belongs_to(super::person::Entity)
                .from(Column::PersonId)
                .to(super::person::Column::Id)
                .into(),
        }
    }
}