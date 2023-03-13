use ::entities::client::Column;
use ::entities::client::{self, ActiveModel as ClientActive, Entity as Client, Model as ClientModel};

use sea_orm::*;

pub async fn get( client_id: i16,
    db: &DbConn
) ->  Result<ClientModel, DbErr> {
    let clients = Client::find_by_id(client_id)
    .all(db).await?;

    Ok(clients.iter().next().unwrap().clone())
}




pub async fn get_all(db: &DbConn) -> Result<Vec<ClientModel>, DbErr> {
    let clients = Client::find()
        .order_by_asc(Column::Name)
        .all(db)
        .await;

    clients
}

pub async fn save(db: &DbConn, client: ClientModel) -> Result<(String, i16), DbErr> {
    let client_id: i16 = client.id;

    let saved_client_id = if client_id < 0  {
        let client = ClientActive {
                name: Set(client.name.to_owned()),
                ..Default::default()
            };

            let result = Client::insert(client).exec(db).await?;

            result.last_insert_id
        }
        else {
            let client = ClientActive {
                id: Set(client_id.to_owned()),
                name: Set(client.name.to_owned()),
                ..Default::default()
            };

            let result = Client::update(client).exec(db).await?;

            result.id // should remain the project_id during update
        };
    Ok((client.name, saved_client_id))
}

pub async fn delete(db: &DbConn, client_id: i16) -> Result<(), DbErr> {
    Client::delete_by_id(client_id).exec(db).await?;
    Ok(())
}

pub async fn delete_all(db: &DbConn) -> Result<(), DbErr> {
    Client::delete_many().filter(client::Column::Id.gte(0)).exec(db).await?;
    
    Ok(())
}
