use ::entities::technology::Column;
use ::entities::technology::{self, Model as TechnologyModel, Entity as Technology, ActiveModel as TechnologyActive};
use sea_orm::*;

pub async fn get( technology_id: i16,
    db: &DbConn
) ->  Result<TechnologyModel, DbErr> {
    let technologies = Technology::find_by_id(technology_id).all(db).await?;

    Ok(technologies.first().unwrap().clone())
}


pub async fn get_all(db: &DbConn) -> Result<Vec<TechnologyModel>, DbErr>  {
    Technology.select().order_by_asc(Column::Name).all(db).await
}

pub async  fn save(db: &DbConn, technology: TechnologyModel) -> Result<(String, i16), DbErr> {
    let technology_id: i16 = technology.id;

    let saved_id = if technology_id < 0  {
        let technology = TechnologyActive {
                name: Set(technology.name.to_owned()),
                ..Default::default()
            };

            let saved = Technology::insert(technology).exec(db).await?;

            saved.last_insert_id
        }
        else {
            let technology = TechnologyActive {
                id: Set(technology_id.to_owned()),
                name: Set(technology.name.to_owned())                
            };

            Technology::update(technology).exec(db).await?;

            technology_id
        };
        Ok((technology.name, saved_id))
}

pub async fn delete(db: &DbConn, technology_id: i16) -> Result<(), DbErr> {
    Technology::delete_by_id(technology_id).exec(db).await?;

    Ok(())
}

pub async fn delete_all(db: &DbConn) -> Result<(), DbErr> {
    Technology::delete_many().filter(technology::Column::Id.gte(0)).exec(db).await?;

    Ok(())
}
