use ::entities::person::Column;
use ::entities::person::{self, Model as PersonModel, Entity as Person, ActiveModel as PersonActive};
use sea_orm::*;


pub async fn get( persion_id: i16,
    db: &DbConn
) ->  Result<PersonModel, DbErr> {
    let persons = Person::find_by_id(persion_id).all(db).await?;

    Ok(persons.first().unwrap().clone())
}

pub async fn get_all(db: &DbConn) -> Result<Vec<PersonModel>, DbErr> {
    Person.select().order_by_asc(Column::Name).all(db).await
}

pub async  fn save(db: &DbConn, person: PersonModel) -> Result<(String, i16), DbErr> {
    let person_id: i16 = person.id;

    let saved_id = if person_id <= 0  {
        let person = PersonActive {
                name: Set(person.name.to_owned()),
                ..Default::default()
            };

            let saved = Person::insert(person).exec(db).await?;

            saved.last_insert_id
        }
        else {
            let person = PersonActive {
                id: Set(person_id.to_owned()),
                name: Set(person.name.to_owned())
            };

            Person::update(person).exec(db).await?;

            person_id
        };

    Ok((person.name, saved_id))
}

pub async fn delete(db: &DbConn, person_id: i16) -> Result<(), DbErr> {
    Person::delete_by_id(person_id).exec(db).await?;

    Ok(())
}

pub async fn delete_all(db: &DbConn) -> Result<(), DbErr> {
    Person::delete_many().filter(person::Column::Id.gte(0)).exec(db).await?;

    Ok(())
}
