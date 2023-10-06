use actix_web::{
    web,
    App,
    HttpServer,
    middleware,
};
use actix_files::NamedFile;
use actix_web::{HttpRequest, Result};
use std::path::PathBuf;

use log::warn;
use tera::Tera;

mod templating;


const PORT: u16 = 8080;

async fn public(req: HttpRequest, state: web::Data<State>) -> Result<NamedFile> {
    let path: PathBuf = req.match_info().query("filename").parse().unwrap();
    Ok(NamedFile::open(state.get_from_public(path))?)
}

pub struct State {
    public_dir: PathBuf,
}

impl State {
    pub fn new(public_dir: PathBuf) -> Self {
        Self { public_dir }
    }

    pub fn get_from_public(&self, path: PathBuf) -> PathBuf {
        let p = self.public_dir.join(path);
        dbg!(&p);
        p
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let environment = std::env::var("environment").unwrap_or(String::from("development"));
    
    let cwd = std::env::current_dir().unwrap();
    if !cwd.join("templates").is_dir() {
        panic!("Templates dir not found");
    }
    let public_dir = cwd.join("public");
    if !public_dir.is_dir() {
        panic!("Public dir not found");
    }

    let sock = if environment.to_lowercase().contains("prod")  {
        Some(std::env::var("SOCK").expect("Sock not set"))
    } else {
        None
    };


    let server = HttpServer::new(move || {
        let tera = Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).expect(
            "Tera templating failed"
        );

        use templating::routes::*;

        App::new()
            .app_data(web::Data::new(tera))
            .app_data(web::Data::new(State::new(public_dir.clone())))
            .wrap(middleware::Logger::default())
            .service(index)
            .service(about)
            .route("/{filename:.*}", web::get().to(public))
            .service(web::scope("").wrap(templating::error_handlers()))
    });

    // Starts the server depending on the platform, then attaches 1 worker and runs it.
    (
        if environment.to_lowercase().contains("prod")  {
            println!("Server started - listening on unix socket");
            server.bind_uds(sock.expect("Sock previously checked for prod"))?
        } else {
            let address = ("127.0.0.1", PORT);
            warn!("Running in development mode");
            println!("Server started - listening on http://{}:{}", address.0, address.1);
            server.bind(address)?
        }
    )
        .workers(1)
        .run().await
}
