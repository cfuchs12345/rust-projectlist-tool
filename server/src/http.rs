use actix_web::{middleware::Logger, web, App, HttpServer, HttpRequest, Result};
use actix_files as fs;
use tera::Tera;
use core::panic;
use std::env;

use crate::database;

use ::httproutes::appdata::AppData;
use ::httproutes::routes;
use ::httproutes::config;


#[actix_web::main]
pub async fn start() -> std::io::Result<()> {
    let config = config::get_config();
    let conn = database::connect().await.unwrap();
    let bind_address = config.get_string("bind_address").unwrap();

    database::migrate(&conn).await;
     
     let app_data_templates = match  Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };

    let app_data = AppData { app_data_templates: app_data_templates, app_data_conn: conn, app_data_config: config };

    HttpServer::new(move || {
        App::new()
            
            .wrap(Logger::default())
            .app_data(web::Data::new(app_data.clone()))
            .configure(init)
    })
    .bind(bind_address)?
    .run()
    .await
}

async fn fav_icon(_req: HttpRequest) -> Result<fs::NamedFile> {
    
    let path_found = fs::NamedFile::open("./server/static/images/favicon.ico");
    match path_found {
        Ok(f) => return Ok(f),
        Err(e) => panic!("file not found {}", e)
    }
}


fn init(cfg: &mut web::ServiceConfig) {
    cfg.route("/favicon.ico", web::get().to(fav_icon));
    cfg.service(fs::Files::new("/static", "./server/static"));


    cfg.service(routes::index);
    
    cfg.service(routes::generate_pdf_projectlist);
    cfg.service(routes::push_projectlist_to_receiver);

    cfg.service(routes::list_project);
    cfg.service(routes::createorupdate_project);
    cfg.service(routes::edit_project);
    cfg.service(routes::delete_project);

    cfg.service(routes::list_person);
    cfg.service(routes::createorupdate_person);
    cfg.service(routes::edit_person);
    cfg.service(routes::delete_person);

    cfg.service(routes::list_role);
    cfg.service(routes::createorupdate_role);
    cfg.service(routes::edit_role);
    cfg.service(routes::delete_role);

    cfg.service(routes::list_technology);
    cfg.service(routes::createorupdate_technology);
    cfg.service(routes::edit_technology);
    cfg.service(routes::delete_technology);

    cfg.service(routes::list_client);
    cfg.service(routes::createorupdate_client);
    cfg.service(routes::edit_client);
    cfg.service(routes::delete_client);

    cfg.service(routes::list_businessarea);
    cfg.service(routes::createorupdate_businessarea);
    cfg.service(routes::edit_businessarea);
    cfg.service(routes::delete_businessarea);
}