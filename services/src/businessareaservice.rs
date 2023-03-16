
use entities::businessarea::Column;
use ::entities::businessarea::{Model as BusinessAreaModel, Entity as BusinessArea, ActiveModel as BusinessAreaActive};
use sea_orm::*;

pub async fn get( businessarea_id: i16,
    db: &DbConn
) ->  Result<BusinessAreaModel, DbErr> {
    let businessareas = BusinessArea::find_by_id(businessarea_id).all(db).await?;

    Ok(businessareas.first().unwrap().clone())
}

pub async fn get_all(db: &DbConn) -> Result<Vec<BusinessAreaModel>, DbErr>  {
    BusinessArea.select().order_by_asc(Column::NameDe).all(db).await
}

pub async  fn save(db: &DbConn, businessarea: BusinessAreaModel) -> Result<(String, i16), DbErr> {
    let businessarea_id: i16 = businessarea.id;

    let saved_id = if businessarea_id < 0  {
        let businessarea = BusinessAreaActive {
                name_de: Set(businessarea.name_de.to_owned()),
                name_en: Set(businessarea.name_en.to_owned()),
                ..Default::default()
            };

            let saved = BusinessArea::insert(businessarea).exec(db).await?;

            saved.last_insert_id
        }
        else {
            let businessarea = BusinessAreaActive {
                id: Set(businessarea_id.to_owned()),
                name_de: Set(businessarea.name_de.to_owned()),
                name_en: Set(businessarea.name_en.to_owned())
            };

            BusinessArea::update(businessarea).exec(db).await?;

            businessarea_id
        };
    Ok((businessarea.name_de, saved_id))
}

pub async fn delete(db: &DbConn, businessarea_id: i16) -> Result<(), DbErr> {
    BusinessArea::delete_by_id(businessarea_id).exec(db).await?;

    Ok(())
}

pub async fn delete_all(db: &DbConn) -> Result<(), DbErr> {
    BusinessArea::delete_many().filter(entities::businessarea::Column::Id.gte(0)).exec(db).await?;

    Ok(())
}

