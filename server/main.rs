use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::Path;
use std::time::SystemTime;

use config::{Config, File as ConfigFile, FileFormat};
use anyhow::Result;
use querystring::querify;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server, StatusCode};
use tokio::{fs::File, io::AsyncReadExt};
use serde::{Deserialize, Serialize};

// use std::sync::RwLock;
// use include_dir::{include_dir, Dir};

// static STATIC_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/static");

type Query<'a> = querystring::QueryParams<'a>;

lazy_static::lazy_static! {
    static ref SETTINGS: Config = {
        Config::builder()
            .add_source(ConfigFile::from_str(include_str!("../static/default_conf.toml"), FileFormat::Toml))
            .build()
            .unwrap()
    };
}

macro_rules! NOT_FOUND_ERR {
    () => {
        Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("404 not found".into())?)
    };
}

macro_rules! INTERNAL_SERVER_ERR {
    () => {
        Ok(Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body("internal server error".into())?)
    };
}


#[derive(Debug, Serialize, Deserialize)]
struct FileMetadata{
    pub is_file: bool,
    pub is_folder: bool,
    pub is_symlink: bool,
    pub modified: Option<f64>,
    pub accessed: Option<f64>,
    pub created: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Fileinfo {
    pub name: String,
    pub urlpath: String,
    pub metadata: Option<FileMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FolderInfo {
    pub name: String,
    pub urlpath: String,
    pub children: Vec<Fileinfo>,
}

fn conv_systemtime_to_f64<E>(syst: Result<SystemTime, E>) -> Option<f64>{
    match syst{
        Ok(v) => v.duration_since(SystemTime::UNIX_EPOCH).map_or(None, |w| Some(w.as_secs_f64())),
        Err(_) => None,
    }
}

impl From<std::fs::Metadata> for FileMetadata {
    fn from(mtd: std::fs::Metadata) -> Self {
        Self { 
            is_file: mtd.is_file(), 
            is_folder: mtd.is_dir(), 
            is_symlink: mtd.is_symlink(),
            modified: conv_systemtime_to_f64(mtd.modified()),
            accessed: conv_systemtime_to_f64(mtd.accessed()), 
            created: conv_systemtime_to_f64(mtd.created()),
        }
    }
}

impl From<std::fs::DirEntry> for Fileinfo{
    fn from(entry: std::fs::DirEntry) -> Self {
        let metadata: Option<FileMetadata> = match entry.metadata() {
            Ok(v) => Some(v.into()),
            Err(_) => None,
        };
        Self {
            name: entry.file_name().to_str().unwrap().to_string(),
            urlpath: entry.path().to_str().unwrap().strip_prefix(".").unwrap_or_default().to_string(),
            metadata: metadata,
        }
    }
}

async fn file_handler(_req: &Request<Body>, path: &Path, _query: &Query<'_>) -> Result<Response<Body>>{
    Ok(Response::new({
        let mut f = File::open(path).await?;
        let mut buffer = Vec::new();
        f.read_to_end(&mut buffer).await?;
        Body::from(buffer)
    }))
}


async fn folder_handler(_req: &Request<Body>, path: &Path, _query: &Query<'_>) -> Result<Response<Body>> {
    let readdir =  match std::fs::read_dir(path) {
        Ok(v) => v,
        Err(_) => return NOT_FOUND_ERR!(),
    };
    let files: Vec<Fileinfo> = readdir
        .filter_map(|entry| -> Option<Fileinfo> {
            match entry {
                Ok(entry) => Some(entry.into()),
                Err(_) => None,
            }
        })
        .collect();
    let folderinfo = FolderInfo {
        name: path.file_name().unwrap_or_default().to_str().unwrap_or_default().to_string(),
        urlpath: path.to_str().unwrap().strip_prefix(".").unwrap_or_default().to_string(),
        children: files,
    };
    let body: Body = match serde_json::to_string(&folderinfo){
        Ok(v) => v.into(),
        Err(_) => return INTERNAL_SERVER_ERR!(),
    };
    Ok(Response::builder()
        .header("content-type", "application/json; charset=UTF-8")
        .body(body)?)
}

async fn main_handler(req: Request<Body>) -> Result<Response<Body>> {
    let path = req.uri().path().strip_prefix("/").unwrap();
    let path = Path::new(&SETTINGS.get_string("basepath")?).join(path);
    let path = path.as_path();
    let query = req.uri().query().unwrap_or_default();
    let query: Query = querify(query);
    println!("{:?} {:?}", path, query);

    if path.is_file() {
        file_handler(&req, path, &query).await
    } else if path.is_dir() {
        folder_handler(&req, path, &query).await
    } else {
        match path.try_exists() {
            // returns 404 if the path does not exist
            Ok(false) => NOT_FOUND_ERR!(),
            // I think it's impossible to be true
            Ok(true) | Err(_) => INTERNAL_SERVER_ERR!(),
        }
    }
}

#[tokio::main]
pub async fn main() -> Result<()> {
    let serv_fn = make_service_fn(|_conn| {
        async { Ok::<_, Infallible>(service_fn(main_handler)) }
    });

    let addr: SocketAddr = ([127, 0, 0, 1], 3000).into();
    let server = Server::bind(&addr).serve(serv_fn);

    println!("Listening on http://{}", addr);
    server.await?;

    Ok(())
}