use std::sync::Arc;
use actix_server::Server;
use actix_cors::Cors;
use actix_web::{http::header, middleware::Logger, web, App, HttpServer};
use super::{PortData, PortError};
use self::routes::*;

pub mod routes;
pub mod entities;

pub fn start_server<S>(address: S, data: PortData) -> Result<Server, PortError>
where
    S: AsRef<str>
{
    let server = HttpServer::new( move || {
        App::new()
            .wrap(
                Cors::new()
                    .allowed_methods(vec!["GET", "POST", "PUT"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .wrap(Logger::default())
            .data( Arc::new(data.clone()) )
        // .service( web::resource("/").route(web::get().to( || {
        //     HttpResponse::Found()
        //         .header( "LOCATION", "/static/index.html")
        //         .finish()
        // })))
            .service(
                web::scope("/api/cluster")
                    // .service( web::resource("/echo").to_async(echo))
                    .service(web::resource("/nodes").to_async(all_nodes_route))
                    .service(
                        web::resource("/nodes/{uid}")
                            .route(web::get().to_async(node_route))
                            .route(web::post().to_async(join_cluster_route))
                            .route(web::delete().to_async(leave_cluster_route)),
                    )
                    .service(web::resource("/state").route(web::get().to_async(state_route)))
                    .service(web::resource("/entries").route(web::post().to_async(append_entries_route)))
                    .service(web::resource("/snapshots").route(web::post().to_async(install_snapshot_route)))
                    .service(web::resource("/vote").route(web::post().to_async(vote_route)))
            )
        // static resources
        // .service( fs::Files::new("/static/", "static/"))
    })
        .bind( address.as_ref() )?
        .start();

    Ok(server)
}