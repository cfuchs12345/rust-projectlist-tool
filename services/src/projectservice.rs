use ::entities::project::Column;
use entities::project::ProjectAndDependencies;
use ::entities::project::{ActiveModel as ProjectActive, Entity as Project, Model as ProjectModel};
use ::entities::project_client::{ActiveModel as ProjectClientActive, Entity as ProjectClient};
use ::entities::project_businessarea::{ActiveModel as ProjectBusinessareaActive, Entity as ProjectBusinessarea};
use ::entities::project_person::{ActiveModel as ProjectPersonActive, Entity as ProjectPerson};
use ::entities::project_role::{ActiveModel as ProjectRoleActive, Entity as ProjectRole};
use ::entities::project_technology::{
    ActiveModel as ProjectTechnologyActive, Entity as ProjectTechnology,
};
use ::entities::client::{Entity as Client, Model as ClientModel};
use ::entities::businessarea::{Entity as Businessarea, Model as BusinessareaModel};
use ::entities::person::{Entity as Person, Model as PersonModel};
use ::entities::role::{Entity as Role, Model as RoleModel};
use ::entities::technology::{Entity as Technology, Model as TechnologyModel};
use ::entities::{project_client, project_person, project_role, project_technology, project_businessarea};


use sea_orm::*;


pub async fn get(
    project_id: i16,
    db: &DbConn,
) -> Result<
Vec<(
    ProjectModel,
    Vec<ClientModel>,
    Vec<BusinessareaModel>,
    Vec<RoleModel>,
    Vec<PersonModel>,
    Vec<TechnologyModel>,
)>,
DbErr,
>{
    let projects = Project::find_by_id(project_id).all(db).await?;
   
    let clients = projects
        .load_many_to_many(Client, ProjectClient, db)
        .await
        ?;

    let businessareas = projects
        .load_many_to_many(Businessarea, ProjectBusinessarea, db)
        .await
        ?;

    let roles = projects
        .load_many_to_many(Role, ProjectRole, db)
        .await
        ?;
    let persons = projects
        .load_many_to_many(Person, ProjectPerson, db)
        .await
        ?;
    let technologies = projects
        .load_many_to_many(Technology, ProjectTechnology, db)
        .await
        ?;

        let mut clients_it = clients.iter();
        let mut businessareas_it = businessareas.iter();
        let mut roles_it = roles.iter();
        let mut persons_it = persons.iter();
        let mut technologies_it = technologies.iter();
    
        Ok(projects
            .iter()
            .map(
                |p| -> (
                    ProjectModel,
                    Vec<ClientModel>,
                    Vec<BusinessareaModel>,
                    Vec<RoleModel>,
                    Vec<PersonModel>,
                    Vec<TechnologyModel>,
                ) {
                    let clients = clients_it.next().unwrap().to_owned();
                    let businessareas = businessareas_it.next().unwrap().to_owned();
                    let roles = roles_it.next().unwrap().to_owned();
                    let persons = persons_it.next().unwrap().to_owned();
                    let technologies = technologies_it.next().unwrap().to_owned();
    
                    (p.to_owned(), clients, businessareas, roles, persons, technologies)
                },
            )
            .collect())
}

pub async fn get_all(
    db: &DbConn,
) -> Result<
    Vec<(
        ProjectModel,
        Vec<ClientModel>,
        Vec<BusinessareaModel>,
        Vec<RoleModel>,
        Vec<PersonModel>,
        Vec<TechnologyModel>,
    )>,
    DbErr,
> {
    let projects = Project::find()
        .order_by_desc(Column::To)
        .all(db)
        .await
        ?;

    let clients = projects
        .load_many_to_many(Client, ProjectClient, db)
        .await
        ?;

    let businessareas = projects
        .load_many_to_many(Businessarea, ProjectBusinessarea, db)
        .await
        ?;
    
    let roles = projects
        .load_many_to_many(Role, ProjectRole, db)
        .await
        ?;
    let persons = projects
        .load_many_to_many(Person, ProjectPerson, db)
        .await
        ?;
    let technologies = projects
        .load_many_to_many(Technology, ProjectTechnology, db)
        .await
        ?;

    let mut clients_it = clients.iter();
    let mut businessareas_it = businessareas.iter();
    let mut roles_it = roles.iter();
    let mut persons_it = persons.iter();
    let mut technologies_it = technologies.iter();

    Ok(projects
        .iter()
        .map(
            |p| -> (
                ProjectModel,
                Vec<ClientModel>,
                Vec<BusinessareaModel>,
                Vec<RoleModel>,
                Vec<PersonModel>,
                Vec<TechnologyModel>,
            ) {
                let clients = clients_it.next().unwrap().to_owned();
                let businessareas = businessareas_it.next().unwrap().to_owned();
                let roles = roles_it.next().unwrap().to_owned();
                let persons = persons_it.next().unwrap().to_owned();
                let technologies = technologies_it.next().unwrap().to_owned();

                (p.to_owned(), clients, businessareas, roles, persons, technologies)
            },
        )
        .collect())
}


pub async fn save(db: &DbConn, project_and_dependencies: ProjectAndDependencies) -> Result<(), DbErr> {
    let project_id: i16 = project_and_dependencies.project.id; // sicne the db sequence starts with 0, we need -1 as undentifier for new form data

    let client_ids = project_and_dependencies.clients;
    let businessarea_ids = project_and_dependencies.businessareas;
    let role_ids = project_and_dependencies.roles;
    let person_ids = project_and_dependencies.persons;
    let technology_ids = project_and_dependencies.technologies;
    
    

    let saved_project_id = if project_id < 0  {
        let project = ProjectActive {
                description_de: Set(project_and_dependencies.project.description_de.to_owned()),
                summary_de: Set(project_and_dependencies.project.summary_de.to_owned()),
                description_en: Set(project_and_dependencies.project.description_en.to_owned()),
                summary_en: Set(project_and_dependencies.project.summary_en.to_owned()),
                duration: Set(project_and_dependencies.project.duration.to_owned()),
                from: Set(project_and_dependencies.project.from.to_owned()),
                to: Set(project_and_dependencies.project.to.to_owned()),
                ..Default::default()
            };

            let result = Project::insert(project).exec(db).await?;

            result.last_insert_id
        }
        else {
            let project = ProjectActive {
                id: Set(project_id.to_owned()),
                description_de: Set(project_and_dependencies.project.description_de.to_owned()),
                summary_de: Set(project_and_dependencies.project.summary_de.to_owned()),
                description_en: Set(project_and_dependencies.project.description_en.to_owned()),
                summary_en: Set(project_and_dependencies.project.summary_en.to_owned()),
                duration: Set(project_and_dependencies.project.duration.to_owned()),
                from: Set(project_and_dependencies.project.from.to_owned()),
                to: Set(project_and_dependencies.project.to.to_owned()),
                ..Default::default()
            };

            let result = Project::update(project).exec(db).await?;

            result.id // should remain the project_id during update
        };

   
    if project_id > -1 {
        delete_relationships(db, project_id).await?;
    }

    let mut project_roles: Vec<ProjectRoleActive> = Vec::new();
    let mut project_persons: Vec<ProjectPersonActive> = Vec::new();
    let mut project_technologies: Vec<ProjectTechnologyActive> = Vec::new();

    if client_ids.len() > 0 {
        let project_client = ProjectClientActive {
            project_id: Set(saved_project_id),
            client_id: Set(client_ids.get(0).unwrap_or(&0).to_owned()),
        };
        ProjectClient::insert(project_client).exec(db).await?;
    }

    if businessarea_ids.len() > 0 {
        let project_businessarea = ProjectBusinessareaActive {
            project_id: Set(saved_project_id),
            businessarea_id: Set(businessarea_ids.get(0).unwrap_or(&0).to_owned()),
        };
        ProjectBusinessarea::insert(project_businessarea).exec(db).await?;
    }




    if role_ids.len() > 0 {
        role_ids
            .iter()
            .map(|role_id| {
                let p = ProjectRoleActive {
                    project_id: Set(saved_project_id),
                    role_id: Set(role_id.to_owned()),
                };

                project_roles.push(p);
            })
            .count();
        ProjectRole::insert_many(project_roles).exec(db).await?;
    }

    if person_ids.len() > 0 {
        person_ids
            .iter()
            .map(|person_id| {
                let p = ProjectPersonActive {
                    project_id: Set(saved_project_id),
                    person_id: Set(person_id.to_owned()),
                };
                project_persons.push(p);
            })
            .count();

        ProjectPerson::insert_many(project_persons).exec(db).await?;
    }

    if technology_ids.len() > 0 {
        technology_ids
            .iter()
            .map(|technology_id| {
                let p = ProjectTechnologyActive {
                    project_id: Set(saved_project_id),
                    technology_id: Set(technology_id.to_owned()),
                };
                project_technologies.push(p);
            })
            .count();

        ProjectTechnology::insert_many(project_technologies)
            .exec(db)
            .await?;
    }

    Ok(())
}




pub async fn delete(db: &DbConn, project_id: i16) -> Result<(), DbErr> {
    Project::delete_by_id(project_id).exec(db).await?;

    delete_relationships(db, project_id).await?;

    Ok(())
}

pub async fn delete_relationships(db: &DbConn, project_id: i16) -> Result<(), DbErr>  {
    
    ProjectClient::delete_many()
        .filter(project_client::Column::ProjectId.eq(project_id))
        .exec(db)
        .await?;
    
    ProjectBusinessarea::delete_many()
        .filter(project_businessarea::Column::ProjectId.eq(project_id))
        .exec(db)
        .await?;

    ProjectRole::delete_many()
        .filter(project_role::Column::ProjectId.eq(project_id))
        .exec(db)
        .await?;

    ProjectPerson::delete_many()
        .filter(project_person::Column::ProjectId.eq(project_id))
        .exec(db)
        .await?;

    ProjectTechnology::delete_many()
        .filter(project_technology::Column::ProjectId.eq(project_id))
        .exec(db)
        .await?;

    Ok(())
}