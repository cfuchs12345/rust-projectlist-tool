use std::{collections::HashMap, error::Error};

use actix_web::{web, get, post, HttpRequest, HttpResponse, http::header};
use http::StatusCode;
use reqwest::header::HeaderValue;
use tera::Context;
use pdf;
use log;
use uuid::Uuid;
use ::entities::{businessarea::Model as BusinessArea, client::Model as Client, role::Model as Role, technology::Model as Technology, person::Model as Person, projectlist::ProjectList};
use ::services::{businessareaservice, clientservice, roleservice, technologyservice, personservice, projectservice};
use ::form_entities::entities::{ProjectWithRelatedEntites};

use crate::{appdata::AppData};

#[get("/")]
pub(crate) async fn index(_data: web::Data<AppData>, _req: HttpRequest) -> Result<HttpResponse, http::Error> {
    Ok(HttpResponse::Found()
    .append_header(("location", "/listproject"))
    .finish())
}

#[get("/generate_pdf_projectlist")]
pub async fn generate_pdf_projectlist(data: web::Data<AppData>, req: HttpRequest) -> HttpResponse  {
    let params = split_query_string(req.query_string());
    let language = params.get("language").unwrap_or(&"de").to_owned();

    let projects = projectservice::get_all(&data.app_data_conn).await.unwrap_or_default(); // vec of tuple with all project related information
    
    let uuid = Uuid::new_v4();

    let target_file_name = format!("project_list_{}.pdf", uuid);

    let response = 
    match pdf::generate_projectlist::generate_pdf(&data.app_data_config, projects, target_file_name.clone(), language) {
        Ok(_r) => {
            let file = actix_files::NamedFile::open_async(target_file_name).await.expect("PDF file could not be loaded");

           
            let mut response = file.into_response(&req);
            
            response.headers_mut().insert(header::CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename='projectlist.pdf'"));

            response
        }
        Err(e) =>  { HttpResponse::InternalServerError().body(e.to_string()) }
    };
    response
}

#[get("/listproject")]
pub(crate) async fn list_project(data: web::Data<AppData>, _req: HttpRequest) -> Result<HttpResponse, http::Error>  {
    list_projects_internal(data, None, None).await
}

#[get("/editproject/{project_id}")]
pub(crate) async fn edit_project(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let project_id = path.into_inner();

   list_projects_internal(data, Some(project_id), None).await
}

#[get("/pushprojectlist")]
pub(crate) async fn push_projectlist_to_receiver(data: web::Data<AppData>)  -> Result<HttpResponse, http::Error>  {    
    let targeturl = data.app_data_config.get_string("projectlist_receiver_rest_service");

    let found_project_tuples = projectservice::get_all(&data.app_data_conn).await;

    let mut ctx =   Context::new();

    if found_project_tuples.is_ok() {
        let project_list = ProjectList {
            list: found_project_tuples.unwrap()
        };
        // Create a new HTTP client
        let client = reqwest::Client::new();

        let api_key = data.app_data_config.get_string("projectlist_receiver_api_key");

        match api_key {
            Ok(key) => {
                // Send the POST request with the JSON object as the request body
                let response = client 
                    .post(targeturl.unwrap())
                    .header("X-API-KEY", key)
                    .json(&project_list)
                    .send()
                    .await;
                
                match response {
                    Ok(resp) => {
                        match resp.status() {
                            StatusCode::OK => ctx.insert("message", "Successfully pushed projectlist to configured target service"),
                            StatusCode::UNAUTHORIZED => ctx.insert("error", "Authentication failed. API key invalid"),
                            StatusCode::INTERNAL_SERVER_ERROR => ctx.insert("error", "Target service produced an internal server errors while processing the data"),
                            y => ctx.insert("error", &format!("Could not push projectlist to targetserver. Response status was {:?}", y))
                        }
                    },
                    _ => {
                        ctx.insert("error", "Could not push projectlist to configured target service");
                    }
                }
            },
            _e => {
                ctx.insert("error", "API key not set in .env file");
            }
    
        }
    }
    else {    
        ctx.insert("warning", "No projects found");
    }
    list_projects_internal(data, None, Some(ctx)).await
}

// common method for edit and list - only the project id is set or not as input
async fn list_projects_internal(data: web::Data<AppData>, selected_project_id: Option<i16>, existing_ctx: Option<Context>) -> Result<HttpResponse, http::Error>  {
    let mut ctx = match existing_ctx {
        Some(existing_ctx) => existing_ctx,
        None => Context::new()
    };

    match selected_project_id {
        Some(project_id)    => {
            let found_project_tuples = projectservice::get(project_id, &data.app_data_conn).await.unwrap_or_default();
            
            if found_project_tuples.len() == 1 {
                match found_project_tuples.first() {
                    Some(project_tuple) => {
                        ctx.insert("input_id", &project_tuple.0.id);

                        let selected_ids:Vec<i16> = project_tuple.1.iter().map( |m| -> i16 { m.id}).collect();
                        ctx.insert("selected_clients", &selected_ids);

                        let selected_ids:Vec<i16> = project_tuple.2.iter().map( |m| -> i16 { m.id}).collect();
                        ctx.insert("selected_businessareas", &selected_ids);

                        let selected_ids:Vec<i16> = project_tuple.3.iter().map( |m| -> i16 { m.id}).collect();
                        ctx.insert("selected_roles", &selected_ids);

                        let selected_ids:Vec<i16> = project_tuple.4.iter().map( |m| -> i16 { m.id}).collect();
                        ctx.insert("selected_persons", &selected_ids);

                        let selected_ids:Vec<i16> = project_tuple.5.iter().map( |m| -> i16 { m.id}).collect();
                        ctx.insert("selected_technologies", &selected_ids);

                        ctx.insert("input_summary_de", &project_tuple.0.summary_de);                        
                        ctx.insert("input_summary_en", &project_tuple.0.summary_en);                        
                        ctx.insert("input_description_de", &project_tuple.0.description_de);
                        ctx.insert("input_description_en", &project_tuple.0.description_en);
                        ctx.insert("input_from", &project_tuple.0.from);
                        ctx.insert("input_to", &project_tuple.0.to);
                        ctx.insert("input_duration", &project_tuple.0.duration);                         
                    },
                    None => {
                        log::warn!("Project with id {} not found (result was None) - setting default/empty values for input", project_id);
                        ctx.insert("warning", & format!("project with id {} not found", project_id));
                        
                        default_input_project(&mut ctx);
                    }
                }
            }
            else {
                log::warn!("Project with id {} not found or len != 1 - setting default/empty values for input", project_id);
                ctx.insert("warning", & format!("project with id {} not found", project_id));
                
                default_input_project(&mut ctx);
            }            
        },
        None => {
            log::debug!("list project mode - setting default/empty values for input");
            default_input_project(&mut ctx);
        }
    }
    let projects = projectservice::get_all(&data.app_data_conn).await.unwrap_or_default(); // vec of tuple with all project related information
    let clients = clientservice::get_all(&data.app_data_conn).await.unwrap_or_default();
    let businessareas = businessareaservice::get_all(&data.app_data_conn).await.unwrap_or_default();
    let roles = roleservice::get_all(&data.app_data_conn).await.unwrap_or_default();
    let persons = personservice::get_all(&data.app_data_conn).await.unwrap_or_default();
    let technologies = technologyservice::get_all(&data.app_data_conn).await.unwrap_or_default();

    
    ctx.insert("name", "listproject.html");
    ctx.insert("clients", &clients); // needed for form i
    ctx.insert("businessareas", &businessareas); // needed for form i
    ctx.insert("roles", &roles); // needed for form input
    ctx.insert("persons", &persons); // needed for form input
    ctx.insert("projects", &projects); // needed for form input
    ctx.insert("technologies", &technologies); // needed for form input

    let push_url_optional = data.app_data_config.get_string("projectlist_receiver_rest_service");
    match push_url_optional {
        Ok(push_url) => {
            if !push_url.is_empty() {
                ctx.insert("send_activated", &true);
            }
        },
        e => {
            log::error!("Error while getting push url from config {:?}", e);
        }
    }

    
    let rendered = match data.app_data_templates.render("tera/project/listproject.html.tera", &ctx) {
        Ok(t) => t,
        Err(e) => {
           handle_error(e)
        }
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

fn default_input_project( ctx: &mut Context)  {
    let empty_vec:Vec<i16> = Vec::new();
    let empty_string = String::from("");
    ctx.insert("selected_persons", &empty_vec);
    ctx.insert("selected_clients", &empty_vec);
    ctx.insert("selected_businessareas", &empty_vec);
    ctx.insert("selected_roles", &empty_vec);                     
    ctx.insert("selected_technologies", &empty_vec);


    ctx.insert("input_id", &-1);
    ctx.insert("input_person_ids", &empty_vec);                  
    ctx.insert("input_client_ids", &empty_vec);         
    ctx.insert("input_businessarea_ids", &empty_vec);         
    ctx.insert("input_role_ids", &empty_vec);                     
    ctx.insert("input_technologies_ids", &empty_vec);                  

    ctx.insert("input_summary_de", &empty_string);                    
    ctx.insert("input_summary_en", &empty_string);                    
    ctx.insert("input_description_de", &empty_string);
    ctx.insert("input_description_en", &empty_string);
    ctx.insert("input_from", &empty_string);
    ctx.insert("input_to", &empty_string);
    ctx.insert("input_duration", &empty_string);
}


#[post("/createorupdateproject")]
pub(crate) async fn createorupdate_project(data: web::Data<AppData>, form_data: web::Form<ProjectWithRelatedEntites>) -> Result<HttpResponse, http::Error>  {
    projectservice::save(&data.app_data_conn, form_data.to_project_and_dependencies()).await.unwrap();

    Ok(HttpResponse::Found()
    .append_header(("location", "/listproject"))
    .finish())
}

#[get("/deleteproject/{project_id}")]
pub(crate) async fn delete_project(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let project_id = path.into_inner();

    projectservice::delete(&data.app_data_conn, project_id).await.unwrap();
    
    Ok(HttpResponse::Ok().content_type("text/html").body("deleted"))
}

#[get("/listperson")]
pub(crate) async fn list_person(data: web::Data<AppData>, _req: HttpRequest) -> Result<HttpResponse, http::Error>  {
    list_person_internal(data, None).await
}

#[get("/editperson/{person_id}")]
pub(crate) async fn edit_person(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let person_id = path.into_inner();

    list_person_internal(data, Some(person_id)).await
}

async fn list_person_internal(data: web::Data<AppData>, selected_person_id: Option<i16>) -> Result<HttpResponse, http::Error>  {
    let mut ctx = Context::new();

    match selected_person_id {
        Some(person_id)    => {
            let found_person = personservice::get(person_id, &data.app_data_conn).await;
            match found_person {
                Ok(person) => {
                    ctx.insert("input_id", &person.id);
                    ctx.insert("input_name", &person.name);
                },
                e => {
                    log::error!("Error: {:?}", e);
                    ctx.insert("input_id", &-1);
                    ctx.insert("input_name", &"");
                }
            }
        },
        None => {
            ctx.insert("input_id", &-1);
            ctx.insert("input_name", &"");
        }
    }

    let persons = personservice::get_all(&data.app_data_conn).await;
    match persons {
        Ok(persons) =>  ctx.insert("persons", &persons),
        e => log::error!("Error while loading persons: {:?}", e)
    }

    ctx.insert("name", "listperson.html");
   
    
    let rendered = match data.app_data_templates.render("tera/person/listperson.html.tera", &ctx) {
        Ok(t) => t,
        Err(e) => handle_error(e)
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

#[post("/createorupdateperson")]
pub(crate) async fn createorupdate_person(data: web::Data<AppData>, form_data: web::Form<Person>) -> Result<HttpResponse, http::Error>  {
    personservice::save(&data.app_data_conn, form_data.into_inner()).await.unwrap();

    Ok(HttpResponse::Found()
    .append_header(("location", "/listperson"))
    .finish())
}

#[get("/deleteperson/{person_id}")]
pub(crate) async fn delete_person(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let person_id = path.into_inner();

    personservice::delete(&data.app_data_conn, person_id).await.unwrap_or_default();
    
    Ok(HttpResponse::Ok().content_type("text/html").body("deleted"))
}



#[get("/listrole")]
pub(crate) async fn list_role(data: web::Data<AppData>, _req: HttpRequest) -> Result<HttpResponse, http::Error>  {
    list_role_internal(data, None).await
}

#[get("/editrole/{role_id}")]
pub(crate) async fn edit_role(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let role_id = path.into_inner();

   list_role_internal(data, Some(role_id)).await
}

async fn list_role_internal(data: web::Data<AppData>, selected_role_id: Option<i16>) -> Result<HttpResponse, http::Error>  {
    let mut ctx = Context::new();

    match selected_role_id {
        Some(role_id)    => {
            let found_role = roleservice::get(role_id, &data.app_data_conn).await;
            match found_role {
                Ok(role) => {
                    ctx.insert("input_id", &role.id);
                    ctx.insert("input_name_de", &role.name_de);
                    ctx.insert("input_name_en", &role.name_en);
                },
                e => {
                    log::error!("Error: {:?}", e);
                    ctx.insert("input_id", &-1);
                    ctx.insert("input_name_de", &"");
                    ctx.insert("input_name_en", &"");
                }
            }
        },
        None => {
            ctx.insert("input_id", &-1);
            ctx.insert("input_name_de", &"");
            ctx.insert("input_name_en", &"");
        }
    }
    let roles = roleservice::get_all(&data.app_data_conn).await;

    match roles {
        Ok(roles) => ctx.insert("roles", &roles),
        e => log::error!("Error while loading roles: {:?}", e)
    }
    
    
    ctx.insert("name", "listrole.html");
    
    
    let rendered = match data.app_data_templates.render("tera/role/listrole.html.tera", &ctx) {
        Ok(t) => t,
        Err(e) => handle_error(e)
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

#[post("/createorupdaterole")]
pub(crate) async fn createorupdate_role(data: web::Data<AppData>, form_data: web::Form<Role>) -> Result<HttpResponse, http::Error>  {
    roleservice::save(&data.app_data_conn, form_data.into_inner()).await.unwrap();

    Ok(HttpResponse::Found()
    .append_header(("location", "/listrole"))
    .finish())
}

#[get("/deleterole/{role_id}")]
pub(crate) async fn delete_role(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let role_id = path.into_inner();

    roleservice::delete(&data.app_data_conn, role_id).await.unwrap_or_default();
    
    Ok(HttpResponse::Ok().content_type("text/html").body("deleted"))
}

#[get("/listtechnology")]
pub(crate) async fn list_technology(data: web::Data<AppData>, _req: HttpRequest) -> Result<HttpResponse, http::Error>  {
    list_technology_internal(data, None).await
}

#[get("/edittechnology/{technology_id}")]
pub(crate) async fn edit_technology(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let technology_id = path.into_inner();

   list_technology_internal(data, Some(technology_id)).await
}

async fn list_technology_internal(data: web::Data<AppData>, selected_technology_id: Option<i16>) -> Result<HttpResponse, http::Error>  {
    let mut ctx = Context::new();

    match selected_technology_id {
        Some(technology_id)    => {
            let found_technology = technologyservice::get(technology_id, &data.app_data_conn).await;
            match found_technology {
                Ok(technology) => {
                    ctx.insert("input_id", &technology.id);
                    ctx.insert("input_name", &technology.name);
                },
                e => {
                    log::error!("Error: {:?}", e);
                    ctx.insert("input_id", &-1);
                    ctx.insert("input_name", &"");
                }
            }
        },
        None => {
            ctx.insert("input_id", &-1);
            ctx.insert("input_name", &"");
        }
    }



    let technologies = technologyservice::get_all(&data.app_data_conn).await;
    

    match technologies {
        Ok(technologies) => ctx.insert("technologies", &technologies),
        e => log::error!("Error while loading technologies: {:?}", e)
    }


    ctx.insert("name", "listtechnology.html");
    
    
    let rendered = match data.app_data_templates.render("tera/technology/listtechnology.html.tera", &ctx) {
        Ok(t) => t,
        Err(e) => handle_error(e)
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}


#[post("/createorupdatetechnology")]
pub(crate) async fn createorupdate_technology(data: web::Data<AppData>, form_data: web::Form<Technology>) -> Result<HttpResponse, http::Error>  {
    technologyservice::save(&data.app_data_conn, form_data.into_inner()).await.unwrap();

    Ok(HttpResponse::Found()
    .append_header(("location", "/listtechnology"))
    .finish())
}

#[get("/deletetechnology/{technology_id}")]
pub(crate) async fn delete_technology(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let technology_id = path.into_inner();

    technologyservice::delete(&data.app_data_conn, technology_id).await.unwrap_or_default();
    
    Ok(HttpResponse::Ok().content_type("text/html").body("deleted"))
}


#[get("/listbusinessarea")]
pub(crate) async fn list_businessarea(data: web::Data<AppData>, _req: HttpRequest) -> Result<HttpResponse, http::Error>  {
    list_businessarea_internal(data, None).await
}

#[get("/editbusinessarea/{businessarea_id}")]
pub(crate) async fn edit_businessarea(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let businessarea_id = path.into_inner();

    list_businessarea_internal(data, Some(businessarea_id)).await
}

async fn list_businessarea_internal(data: web::Data<AppData>, selected_businessarea_id: Option<i16>) -> Result<HttpResponse, http::Error>  {
    let mut ctx = Context::new();

    match selected_businessarea_id {
        Some(businessarea_id)    => {
            let found_businessarea = businessareaservice::get(businessarea_id, &data.app_data_conn).await;
            match found_businessarea {
                Ok(businessarea) => {
                    ctx.insert("input_id", &businessarea.id);
                    ctx.insert("input_name_de", &businessarea.name_de);
                    ctx.insert("input_name_en", &businessarea.name_en);
                },
                e => {
                    log::error!("Error: {:?}", e);
                    ctx.insert("input_id", &-1);
                    ctx.insert("input_name_de", &"");
                    ctx.insert("input_name_en", &"");
                }
            }
        },
        None => {
            ctx.insert("input_id", &-1);
            ctx.insert("input_name_de", &"");
            ctx.insert("input_name_en", &"");
        }
    }



    let business_areas = businessareaservice::get_all(&data.app_data_conn).await;

    match business_areas {
        Ok(business_areas) => ctx.insert("business_areas", &business_areas),
        e => log::error!("Error while loading businessareas: {:?}", e)
    }

    ctx.insert("name", "listbusinessarea.html");
    
    
    let rendered = match data.app_data_templates.render("tera/businessarea/listbusinessarea.html.tera", &ctx) {
        Ok(t) => t,
        Err(e) => handle_error(e)
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

#[post("/createorupdatebusinessarea")]
pub(crate) async fn createorupdate_businessarea(data: web::Data<AppData>, form_data: web::Form<BusinessArea>) -> Result<HttpResponse, http::Error>  {
    businessareaservice::save(&data.app_data_conn, form_data.into_inner()).await.unwrap();

    Ok(HttpResponse::Found()
    .append_header(("location", "/listbusinessarea"))
    .finish())
}

#[get("/deletebusinessarea/{businessarea_id}")]
pub(crate) async fn delete_businessarea(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let businessarea_id = path.into_inner();

    businessareaservice::delete(&data.app_data_conn, businessarea_id).await.unwrap_or_default();
    
    Ok(HttpResponse::Ok().content_type("text/html").body("deleted"))
}


#[get("/listclient")]
pub(crate) async fn list_client(data: web::Data<AppData>, _req: HttpRequest) -> Result<HttpResponse, http::Error>  {
    list_client_internal(data, None).await
}

#[get("/editclient/{client_id}")]
pub(crate) async fn edit_client(data: web::Data<AppData>, path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let client_id = path.into_inner();
    list_client_internal(data, Some(client_id)).await
}

 async fn list_client_internal(data: web::Data<AppData>, selected_client_id: Option<i16>) -> Result<HttpResponse, http::Error>  {
    let mut ctx = Context::new();

    match selected_client_id {
        Some(client_id)    => {
            let found_client = clientservice::get(client_id, &data.app_data_conn).await;
            match found_client {
                Ok(client) => {
                    ctx.insert("input_id", &client.id);
                    ctx.insert("input_name", &client.name);
                },
                e => {
                    log::error!("Error: {:?}", e);
                    ctx.insert("input_id", &-1);
                    ctx.insert("input_name", &"");
                }
            }
        },
        None => {
            ctx.insert("input_id", &-1);
            ctx.insert("input_name", &"");
        }
    }

    let clients = clientservice::get_all(&data.app_data_conn).await; 
    
    match clients {
        Ok(clients) => ctx.insert("clients", &clients),
        e => log::error!("Error while loading clients: {:?}", e)
    }

    let rendered = match data.app_data_templates.render("tera/client/listclient.html.tera", &ctx) {
        Ok(t) => t,
        Err(e) =>handle_error(e)
    };
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}

#[post("/createorupdateclient")]
pub(crate) async fn createorupdate_client(data: web::Data<AppData>, form_data: web::Form<Client>) -> Result<HttpResponse, http::Error>  {
    clientservice::save(&data.app_data_conn, form_data.into_inner()).await.unwrap_or_default();
    
    Ok(HttpResponse::Found()
    .append_header(("location", "/listclient"))
    .finish())
}

#[get("/deleteclient/{client_id}")]
pub(crate) async fn delete_client(data: web::Data<AppData>,path: web::Path<i16>) -> Result<HttpResponse, http::Error>  {
    let client_id = path.into_inner();
    println!("Id: {}", client_id);
    clientservice::delete(&data.app_data_conn, client_id).await.unwrap_or_default();
    
    Ok(HttpResponse::Ok().content_type("text/html").body("deleted"))
}



fn split_query_string(string: &str) -> HashMap<&str, &str> {
    if string.is_empty() || ! string.contains('=') {
        return HashMap::new();
    }
    string.split(',').map(|s| s.split_at(s.find('=').unwrap())).map(|(key, val)| (key, &val[1..])).collect()
}

fn handle_error(e: tera::Error ) -> String {
    log::error!("Error: {}", e);
    let mut cause = e.source();
    while let Some(e) = cause {
        log::error!("Reason: {}", e);
        cause = e.source();
    }
    "could not render page".to_string()
}