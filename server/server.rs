use crate::getconfig;
use anyhow::Result;
use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server, StatusCode};
use std::convert::Infallible;
use std::fs::{metadata, DirEntry, Metadata};
use std::path::Path;
use std::ptr::metadata;
use std::time::SystemTime;

enum FileMetadata {
    Picture {
        size: u64,
        modified: Option<SystemTime>,
        created: Option<SystemTime>,
    },
    File {
        size: u64,
        modified: Option<SystemTime>,
        created: Option<SystemTime>,
    },
    Folder {
        modified: Option<SystemTime>,
        created: Option<SystemTime>,
    },
    None,
}

impl FileMetadata {
    pub fn getMetadata(entry: DirEntry) -> Option<FileMetadata> {
        if let Ok(ft) = entry.file_type() {
            if ft.is_file() {
                entry
                    .metadata()
                    .map(|metadata| FileMetadata::File {
                        size: metadata.len(),
                        modified: metadata.modified().ok(),
                        created: metadata.created().ok(),
                    })
                    .ok()
            } else if ft.is_dir() {
                entry
                    .metadata()
                    .map(|metadata| FileMetadata::Folder {
                        modified: metadata.modified().ok(),
                        created: metadata.created().ok(),
                    })
                    .ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}

struct Fileinfo {
    pub name: String,
    pub filepath: String,
    pub urlpath: String,
    pub metadata: Option<FileMetadata>,
}

struct DirectoryInfo {
    pub name: String,
    pub dirpath: String,
    pub urlpath: String,
    pub files: Vec<Fileinfo>,
}

struct DirectoryHandler {
    basepath: String,
}

impl DirectoryHandler {
    pub fn getinfo(&self, strpath: String) -> Vec<DirectoryInfo> {
        let filepath = Path::new(&self.basepath).join(strpath);
        if filepath.is_dir() {
            if let Ok(readdir) = std::fs::read_dir(filepath.as_path()) {
                let files: Vec<Fileinfo> = readdir
                    .filter_map(|entry| -> Option<Fileinfo> {
                        match entry {
                            Ok(entry) => Some(Fileinfo {
                                name: entry.file_name().to_str().unwrap().to_string(),
                                filepath: entry.path().to_str().unwrap().to_string(),
                                urlpath: entry.path().to_str().unwrap().to_string(),
                                metadata: FileMetadata::getMetadata(entry),
                            }),
                            Err(_) => None,
                        }
                    })
                    .collect();
                return files;
            }
        }
        vec![]
        // std::fs::read_dir(filepath.to_str())?
    }
}

async fn handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    let mut response = Response::new(Body::empty());

    match req.method() {
        &Method::GET => {}
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
            return Ok(response);
        }
    }
    Ok(Response::new("Hello, World".into()))
}

pub async fn runserver(args: &getconfig::Args) -> Result<()> {
    let dh = DirectoryHandler {
        basepath: args.basedir.clone(),
    };

    let server = Server::bind(&args.listen).serve(make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(handler))
    }));

    if let Err(e) = server.await {
        eprintln!("server error: {}", e);
        Err(anyhow::Error::new(e))
    } else {
        Ok(())
    }
}
