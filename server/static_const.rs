use crate::template::Template;
use config::{Config, File as ConfigFile, FileFormat};
use include_dir::{include_dir, Dir};
use std::collections::HashMap;
use tera::{Tera, Context};

pub static STATIC_PATH: &str = "/static_contents";
pub static EMBED_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/embedded");

lazy_static::lazy_static! {
    pub static ref CONFIG: Config = {
        Config::builder()
            .add_source(ConfigFile::from_str(
                EMBED_DIR.get_file("default_conf.toml").unwrap().contents_utf8().unwrap(),
                FileFormat::Toml))
            .build()
            .unwrap()
    };

    pub static ref TEMPLATES_RAW: HashMap<String, Template> = {
        let raw_conf = CONFIG.get_table("templates").unwrap();
        let templates = raw_conf.into_iter().filter_map(|v| {
            match Template::from(v){
                Some(tmp) => Some((tmp.name.clone(), tmp)),
                None => None
            }
        });
        HashMap::from_iter(templates)
    };

    pub static ref TEMPLATES: Tera = {
        let mut tera = Tera::default();
        println!("{:?}", TEMPLATES_RAW.keys());
        for (k, v) in TEMPLATES_RAW.iter(){
            tera.add_raw_template(k, v.get_content()).expect("Error in template syntax");
        }
        tera
    };

    pub static ref TEMPLATE_CONTEXT: Context = {
        let mut ctx = Context::default();
        ctx.insert("static_path", STATIC_PATH);
        ctx.insert("timezone", &CONFIG.get_string("timezone").expect("invalid config"));
        ctx.insert("time_format", &CONFIG.get_string("time_format").expect("invalid config"));
        ctx
    };
}
