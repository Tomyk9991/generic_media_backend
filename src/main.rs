use std::env;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use actix_web::middleware::{Condition, Logger};
use actix_web::web::Data;
use actix_web_httpauth::middleware::HttpAuthentication;
use crate::api::user::create_user;

use crate::model::states::app_state::AppState;

mod api;
mod model;
mod middleware;
mod database;
mod utils;
mod migrations;


#[actix_web::main]
#[allow(clippy::expect_used)]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let mut debug_mode = false;

    if env::args().skip(1).collect::<Vec<_>>().contains(&String::from("--debug")) {
        log::info!("Debug mode. Enabled CORS and \"User create service\"");
        debug_mode = true;
    }

    let app_state = AppState::from_env().await.unwrap_or_else(|e| {
        log::error!("{:?}", e);
        std::process::exit(1);
    });

    // create or do nothing, when created
    std::fs::create_dir_all(&app_state.data_directory)?;

    let (ip, port) = app_state.ip_port_tuple.clone();


    let server = HttpServer::new(move || {
        let logger = Logger::default();
        let cookie_middleware = HttpAuthentication::with_fn(middleware::cookie_validator::validator);

        let mut s = web::scope("/api")
            .app_data(Data::new(app_state.clone()))
            .wrap(Condition::new(debug_mode, Cors::permissive().supports_credentials()))
            .wrap(logger);

        if debug_mode {
            s = s.service(create_user);
        }

        App::new()
            .service(s
                .service(api::authentication::cookie_revalidate)
                .service(api::authentication::cookie_auth)
                .service(api::authentication::logout_cookie)
                .service(web::scope("/media")
                    .wrap(cookie_middleware.clone())
                    .service(api::media::list)
                    .service(api::media::upload)
                    .service(api::media::stories)
                    .service(api::media::story)
                    .service(api::media::upload_story)
                    .service(actix_files::Files::new("/", &app_state.data_directory))
                )
                .service(web::scope("/user")
                    .wrap(cookie_middleware.clone())
                    .service(api::user::get_friends)
                    .service(api::user::post_friendship)
                    .service(api::user::upload_avatar)
                    .service(api::user::put_user_information)
                    .service(api::user::list)
                    .service(api::user::put_list)
                    .service(api::user::full_profile_information)
                    .service(api::user::avatar)
                )
                .service(web::scope("")
                    .wrap(cookie_middleware)
                    .service(api::changelog::changelog)
                    .service(api::changelog::changelog_version)
                    .service(api::user::who_am_i)
                )
            )
    })
        .bind((ip.clone(), port))?
        .run();

    println!("Server running at http://{}:{}", ip, port);
    server.await
}