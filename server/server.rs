use crate::static_var::EMBED_DIR;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;

use anyhow::Result;
use hyper::http::HeaderValue;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use querystring::querify;
use tera::Context;
use tokio::{fs::File, io::AsyncReadExt};

use crate::finfo::FolderInfo;
use crate::static_var::*;

type Query<'a> = querystring::QueryParams<'a>;
type HandlerResponse = Result<Response<Body>>;

macro_rules! NOT_FOUND_ERR {
    () => {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("page not found".into())?)
    };
}

macro_rules! BAD_REQUEST_ERR {
    () => {
        Ok(Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("bad request".into())?)
    };
}

macro_rules! INTERNAL_SERVER_ERR {
    () => {
        Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("internal server error".into())?)
    };
}

fn getvalue_from_query<'a>(q: &Query<'a>, key: &str) -> Option<&'a str> {
    q.iter()
        .find_map(|&v| if v.0 == key { Some(v.1) } else { None })
}

async fn file_handler(_req: &Request<Body>, path: &Path, _query: &Query<'_>) -> HandlerResponse {
    let mut f = File::open(path).await?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer).await?;
    Ok(Response::new(Body::from(buffer)))
}

async fn folder_handler(_req: &Request<Body>, path: &Path, query: &Query<'_>) -> HandlerResponse {
    let folderinfo = match FolderInfo::new(path) {
        Ok(v) => v,
        Err(_) => return NOT_FOUND_ERR!(),
    };
    let default_type = CONFIG.get_string("default_type").unwrap();
    let return_type = getvalue_from_query(query, "type").unwrap_or(default_type.as_str());

    let template = match TEMPLATES_RAW.get(return_type) {
        Some(v) => v,
        None => return BAD_REQUEST_ERR!(),
    };

    let mut ctx = Context::default();
    ctx.insert("info", &folderinfo);
    ctx.insert("static_path", STATIC_PATH);

    match template.render(&ctx) {
        Ok((content_type, body)) => Ok(Response::builder()
            .header("content-type", content_type)
            .body(body.into())?),
        Err(_) => INTERNAL_SERVER_ERR!(),
    }
}

async fn static_handler(_req: &Request<Body>, path: &str, _query: &Query<'_>) -> HandlerResponse {
    let path = path.strip_prefix(STATIC_PATH).unwrap_or(path);
    let path = path.strip_prefix("/").unwrap_or(path);
    let path = Path::new("static").join(path);
    let path = path.as_path();

    let file = match EMBED_DIR.get_file(path) {
        Some(f) => f,
        None => return NOT_FOUND_ERR!(),
    };
    let body = file.contents();
    let mut response = Response::new(Body::from(body));
    response.headers_mut().insert("cache-control", HeaderValue::from_static("private, max-age=360000"));
    Ok(response)
}

async fn main_handler(req: Request<Body>) -> HandlerResponse {
    let uripath = req.uri().path();
    let path = uripath.strip_prefix("/").unwrap();
    let path = Path::new(&CONFIG.get_string("basepath")?).join(path);
    let path = path.as_path();

    let query = req.uri().query().unwrap_or_default();
    let query: Query = querify(query);
    println!("{:} {:?}", path.display(), query);

    if path.is_dir() {
        folder_handler(&req, path, &query).await
    } else if uripath.starts_with(STATIC_PATH) {
        static_handler(&req, uripath, &query).await
    } else if path.is_file() {
        file_handler(&req, path, &query).await
    } else {
        match path.try_exists() {
            // returns 404 if the path does not exist
            Ok(false) => NOT_FOUND_ERR!(),
            // I think it's impossible to be true
            Ok(true) | Err(_) => INTERNAL_SERVER_ERR!(),
        }
    }
}

pub async fn run_server() -> Result<()> {
    let serv_fn = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(main_handler)) });

    let addr = CONFIG.get_string("address").unwrap();
    let addr: SocketAddr = addr.parse().expect("Invalid socket address");
    let server = Server::bind(&addr).serve(serv_fn);

    println!("Listening on http://{}", addr);
    server.await?;
    Ok(())
}
