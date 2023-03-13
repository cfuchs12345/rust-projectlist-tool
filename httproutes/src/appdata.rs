use sea_orm::DatabaseConnection;
use tera::Tera;
use config::Config;


#[derive(Debug, Clone)]
pub struct AppData {
    pub app_data_templates: Tera,
    pub app_data_conn: DatabaseConnection,
    pub app_data_config: Config,
}