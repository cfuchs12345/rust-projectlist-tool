use sea_orm::entity::prelude::*;

#[derive(Copy, Clone, Default, Debug, DeriveEntity)]
pub struct Entity;

impl EntityName for Entity {
    fn table_name(&self) -> &str {
        "project_role"
    }
}

#[derive(Clone, Debug, PartialEq, Eq, DeriveModel, DeriveActiveModel)]
pub struct Model {
    pub project_id: i16,
    pub role_id: i16,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveColumn)]
pub enum Column {
    ProjectId,
    RoleId,
}

#[derive(Copy, Clone, Debug, EnumIter, DerivePrimaryKey)]
pub enum PrimaryKey {
    ProjectId,
    RoleId,
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
            Self::RoleId => ColumnType::Integer.def(),
        }
    }
}

impl ActiveModelBehavior for ActiveModel {}


#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Relation {
    Project,
    Role,
}

impl RelationTrait for Relation {
    fn def(&self) -> RelationDef {
        match self {
            Self::Project => Entity::belongs_to(super::project::Entity)
                .from(Column::ProjectId)
                .to(super::project::Column::Id)
                .into(),
            Self::Role => Entity::belongs_to(super::role::Entity)
                .from(Column::RoleId)
                .to(super::role::Column::Id)
                .into(),
        }
    }
}