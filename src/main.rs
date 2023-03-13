use dotenvy;


pub fn main() {
    dotenvy::dotenv().ok(); 

    env_logger::init();

    let result = server::http::start();

    if let Some(err) = result.err() {
        println!("Error: {err}");
    }
}