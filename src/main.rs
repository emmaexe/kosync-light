mod api;
mod args;
mod store;

use args::Arguments;
use once_cell::sync::Lazy;
use serde::Serialize;
use serde_json::Number;
use std::time::{SystemTime, UNIX_EPOCH};
use store::Store;
use tiny_http::{Header, Method, Request, Response, Server};

static CONTENT_TYPE: Lazy<Header> =
    Lazy::new(|| Header::from_bytes("Content-Type", "application/json").unwrap());

fn response<T: Serialize>(data: &Option<T>, http_code: u32) -> Response<std::io::Cursor<Vec<u8>>> {
    if let Some(data) = data {
        return Response::from_string(serde_json::to_string(data).unwrap())
            .with_header(CONTENT_TYPE.clone())
            .with_status_code(http_code);
    } else {
        return Response::from_string("{}")
            .with_header(CONTENT_TYPE.clone())
            .with_status_code(http_code);
    }
}

fn handle_request(store: &mut Store, mut req: Request) {
    let url = req.url().to_string();
    let method = req.method().clone();
    let mut body = String::new();
    req.as_reader().read_to_string(&mut body).unwrap();

    if req.headers().iter().any(|header| {
        header.field.as_str() == "Accept"
            && header.value.as_str() == "application/vnd.koreader.v1+json"
    }) {
        if method == Method::Post && url == "/users/create" {
            if let Ok(body_data) = serde_json::from_str::<api::UserCreateReq>(&body) {
                if store.user_exists(&body_data.username) {
                    req.respond(response(
                        &Some(api::ErrorRes {
                            message: "Username is already registered.",
                            code: 2002,
                        }),
                        402,
                    )).unwrap_or_default();
                } else {
                    store.user_create(&body_data.username, &body_data.password)
                        .expect("Failed to create new user.");
                    req.respond(response(
                        &Some(api::UserCreateRes {
                            username: &body_data.username,
                        }),
                        201,
                    )).unwrap_or_default();
                }
            } else {
                req.respond(response(
                    &Some(api::ErrorRes {
                        message: "Could not parse JSON in body.",
                        code: 103,
                    }),
                    400,
                )).unwrap_or_default();
            }
        } else if method == Method::Get && url == "/users/auth" {
            let username = req
                .headers()
                .iter()
                .find(|header| header.field.as_str() == "x-auth-user")
                .and_then(|header| Some(header.value.to_string()));
            let password = req
                .headers()
                .iter()
                .find(|header| header.field.as_str() == "x-auth-key")
                .and_then(|header| Some(header.value.to_string()));
            match (username.as_ref(), password.as_ref()) {
                (Some(username), Some(password)) if store.user_auth(username, password) => {
                    req.respond(response(&Some(api::UserAuthRes { authorized: "OK" }), 200))
                        .unwrap_or_default();
                }
                _ => {
                    req.respond(response(
                        &Some(api::ErrorRes {
                            message: "Unauthorized",
                            code: 2001,
                        }),
                        401,
                    )).unwrap_or_default();
                }
            }
        } else if method == Method::Put && url == "/syncs/progress" {
            let username = req
                .headers()
                .iter()
                .find(|header| header.field.as_str() == "x-auth-user")
                .and_then(|header| Some(header.value.to_string()));
            let password = req
                .headers()
                .iter()
                .find(|header| header.field.as_str() == "x-auth-key")
                .and_then(|header| Some(header.value.to_string()));
            match (username.as_ref(), password.as_ref()) {
                (Some(username), Some(password)) if store.user_auth(username, password) => {
                    if let Ok(body_data) = serde_json::from_str::<api::ProgressPutReq>(&body) {
                        let time = Number::from(
                            SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs()
                        );
                        store.document_update(username, &body_data, &time)
                            .expect("Failed to update document.");
                        req.respond(response(
                            &Some(api::ProgressPutRes {
                                timestamp: time,
                                document: body_data.document,
                            }),
                            200,
                        )).unwrap_or_default();
                    } else {
                        req.respond(response(
                            &Some(api::ErrorRes {
                                message: "Could not parse JSON in body.",
                                code: 103,
                            }),
                            400,
                        )).unwrap_or_default();
                    }
                }
                _ => {
                    req.respond(response(
                        &Some(api::ErrorRes {
                            message: "Unauthorized",
                            code: 2001,
                        }),
                        401,
                    )).unwrap_or_default();
                }
            }
        } else if method == Method::Get && url.starts_with("/syncs/progress/") {
            if let Some(document) = url.strip_prefix("/syncs/progress/") {
                let username = req
                    .headers()
                    .iter()
                    .find(|header| header.field.as_str() == "x-auth-user")
                    .and_then(|header| Some(header.value.to_string()));
                let password = req
                    .headers()
                    .iter()
                    .find(|header| header.field.as_str() == "x-auth-key")
                    .and_then(|header| Some(header.value.to_string()));
                match (username.as_ref(), password.as_ref()) {
                    (Some(username), Some(password)) if store.user_auth(username, password) => {
                        req.respond(response(
                            &store
                                .document_read(username, document)
                                .expect("Failed to read document."),
                            200,
                        )).unwrap_or_default();
                    }
                    _ => {
                        req.respond(response(
                            &Some(api::ErrorRes {
                                message: "Unauthorized",
                                code: 2001,
                            }),
                            401,
                        )).unwrap_or_default();
                    }
                }
            } else {
                unreachable!("Failed to strip prefix from url???");
            }
        } else if method == Method::Get && url == "/healthcheck" {
            req.respond(response(&Some(api::HealthCheckRes { state: "OK" }), 200))
                .unwrap_or_default();
        } else {
            req.respond(response(
                &Some(api::ErrorRes {
                    message: "404 not found.",
                    code: 404,
                }),
                404,
            )).unwrap_or_default();
        }
    } else {
        req.respond(response(
            &Some(api::ErrorRes {
                message: "Invalid Accept header format.",
                code: 101,
            }),
            412,
        )).unwrap_or_default();
    }
}

fn main() {
    let arguments: Arguments = args::parse_args();
    let server = match Server::http(&arguments.address) {
        Ok(server) => server,
        Err(e) => {
            eprintln!("Failed to start kosync-light server:\n{}", e);
            std::process::exit(1);
        }
    };
    let mut store: Store = Store::new(&arguments).expect("Failed to create store directory.");

    println!("kosync-light v{}", env!("CARGO_PKG_VERSION"));
    println!("Data directory is {}", &arguments.data_path);
    if arguments.noauth {
        println!("Authentication will be ignored, noauth is enabled")
    }
    println!("Serving on {}", &arguments.address);

    for request in server.incoming_requests() {
        handle_request(&mut store, request);
    }
}
