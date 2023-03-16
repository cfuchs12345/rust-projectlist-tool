use ::entities::role::Column;
use ::entities::role::{self, Model as RoleModel, Entity as Role, ActiveModel as RoleActive};
use sea_orm::*;

pub async fn get( role_id: i16,
    db: &DbConn
) ->  Result<RoleModel, DbErr> {
    let roles = Role::find_by_id(role_id).all(db).await?;

    Ok(roles.first().unwrap().clone())
}

pub async fn get_all(db: &DbConn) -> Result<Vec<RoleModel>, DbErr> {
    Role.select().order_by_asc(Column::NameDe).all(db).await
}

pub async  fn save(db: &DbConn, role: RoleModel) -> Result<(String, i16), DbErr> {
    let role_id: i16 = role.id;

    let saved_id = if role_id < 0  {
        let role = RoleActive {
                name_de: Set(role.name_de.to_owned()),
                name_en: Set(role.name_en.to_owned()),
                ..Default::default()
            };

            let saved = Role::insert(role).exec(db).await?;

            saved.last_insert_id
        }
        else {
            let role = RoleActive {
                id: Set(role_id.to_owned()),
                name_de: Set(role.name_de.to_owned()),
                name_en: Set(role.name_en.to_owned())
            };

            Role::update(role).exec(db).await?;

            role_id
        };
        Ok((role.name_de, saved_id))
}

pub async fn delete(db: &DbConn, role_id: i16) -> Result<(), DbErr> {
    Role::delete_by_id(role_id).exec(db).await?;

    Ok(())
}

pub async fn delete_all(db: &DbConn) -> Result<(), DbErr> {
    Role::delete_many().filter(role::Column::Id.gte(0)).exec(db).await?;

    Ok(())
}
