use std::iter::Iterator;
use serde_derive::{Deserialize, Serialize};
use anyhow::Result;
use jsonwebtoken::{TokenData, decode, Validation, DecodingKey};
use lazy_static::lazy_static;
use std::sync::Arc;
use serde_json::json;
use tide::Request;
use http_types::headers::HeaderValue;
use tide::security::{CorsMiddleware, Origin};
use tide_acme::{AcmeConfig, TideRustlsExt};
use sonic_channel::*;

lazy_static! {
    static ref DB : Arc<rocksdb::DB> = {

        let prefix_extractor = rocksdb::SliceTransform::create_fixed_prefix(5);

        let mut opts = rocksdb::Options::default();
        opts.create_if_missing(true);
        opts.set_prefix_extractor(prefix_extractor);

        let configure = env_var_config();
        let db = rocksdb::DB::open(&opts, configure.db).unwrap();
        Arc::new(db)
    };
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: i64,          
    pub iat: i64,         
    pub iss: String,         
    pub sub: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EnvVarConfig {
  pub port: u16,
  pub origin: String,
  pub jwt_secret: String,
  pub db: String,
  pub secure: bool,
  pub certs: String,
  pub domain: String,
  pub sonic_server: String,
  pub sonic_password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Item {
    pub id: uuid::Uuid,
    pub collection: String,
    pub bucket: String,
    pub data: serde_json::Map<String, serde_json::Value>,
    pub indexes: Vec<String>,
    pub locale: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IndexForm {
    pub items: Vec<Item>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DeindexForm {
    pub ids: Vec<uuid::Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SearchForm {
    pub query: String,
    pub collection: String,
    pub bucket: String,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SuggestForm {
    pub query: String,
    pub collection: String,
    pub bucket: String,
    pub limit: Option<usize>,
}

fn replace(key: String, value: Vec<u8>) -> Result<()> {
    DB.put(key.clone(), value.clone())?;
    Ok(())
}

fn delete(key: String) -> Result<()> {
    DB.delete(key.clone())?;
    Ok(())
}

fn get_items() -> Result<Vec<Item>> {
    let prefix = "items".as_bytes();
    let i = DB.prefix_iterator(prefix);
    let res : Vec<Item> = i.map(|(_, v)| {
        let data: Item = rmp_serde::from_read_ref(&v).unwrap();
        data
    }).collect();
    Ok(res)
}

fn puts_items(items: Vec<Item>) -> Result<()> {
    for item in items {
        let key = format!("items_{}", item.id);
        let value = rmp_serde::to_vec_named(&item)?;
        replace(key, value)?;
    }
    Ok(())
}

fn get_item_by_id(id: uuid::Uuid) -> Result<Option<Item>> {
    let items = get_items()?;
    Ok(items.into_iter().filter(|item| item.id == id).last())
}

fn del_items(ids: Vec<uuid::Uuid>) -> Result<()> {
    for id in ids {
        let key = format!("items_{}", id);
        delete(key)?;
    }
    Ok(())
}

fn env_var_config() -> EnvVarConfig {
 
    let mut port : u16 = 8080;
    let mut secure = false;
    let mut origin = "*".to_string();
    let mut jwt_secret = "secret".to_string();
    let mut db: String = "db".to_string();
    let mut certs = "certs".to_string();
    let mut domain = "localhost".to_string();
    let mut sonic_server = "localhost:1491".to_string();
    let mut sonic_password = "SecretPassword".to_string();
    
    let _ : Vec<String> = go_flag::parse(|flags| {
        flags.add_flag("port", &mut port);
        flags.add_flag("origin", &mut origin);
        flags.add_flag("jwt_secret", &mut jwt_secret);
        flags.add_flag("secure", &mut secure);
        flags.add_flag("db", &mut db);
        flags.add_flag("domain", &mut domain);
        flags.add_flag("certs", &mut certs);
        flags.add_flag("sonic_conn", &mut sonic_server);
        flags.add_flag("sonic_password", &mut sonic_password);
    });

    EnvVarConfig{port, origin, jwt_secret, secure, domain, certs, db, sonic_server, sonic_password}
}

async fn jwt_verify(token: String) -> Result<Option<TokenData<Claims>>> {

    let configure = env_var_config();

    let mut parts = token.split(" ");
    let auth_type = parts.next().unwrap();
    if auth_type == "Bearer" {
        let token = parts.next().unwrap();
        let _ = match decode::<Claims>(&token,  &DecodingKey::from_secret(configure.jwt_secret.as_ref()), &Validation::default()) {
            Ok(c) => { return Ok(Some(c)); },
            Err(_) => { return Ok(None); }
        };
    } 
    Ok(None)
}

fn index_with_sonic(items: Vec<Item>) -> Result<()> {
    let configure = env_var_config();

    let channel = IngestChannel::start(configure.sonic_server, configure.sonic_password)?;
    for item in items {
        for (field, value) in item.clone().data {
            for index_field in item.indexes.clone() {
                if index_field == field && item.clone().locale == None {
                    channel.push(&item.collection, &item.bucket, &item.id.to_string(), &value.to_string())?;
                }
                else if index_field == field && item.clone().locale != None {
                    channel.push_with_locale(&item.collection, &item.bucket, &item.id.to_string(), &value.to_string(), &item.clone().locale.unwrap())?;
                }
            }
        }
    }
    Ok(())
}

fn deindex_with_sonic(ids: Vec<uuid::Uuid>) -> Result<()> {
    let configure = env_var_config();

    let channel = IngestChannel::start(configure.sonic_server, configure.sonic_password)?;
    for id in ids {
        match get_item_by_id(id)? {
            Some(item) => {
                channel.flusho(&item.collection, &item.bucket, &item.id.to_string())?;
            },
            None => {}
        }
    }
    Ok(())
}

fn search_with_sonic(sf: SearchForm) -> Result<Vec<Item>> {
    let configure = env_var_config();

    let channel = SearchChannel::start(configure.sonic_server, configure.sonic_password)?;

    let mut items = Vec::new();

    if sf.offset != None && sf.limit != None {
        let ids: Vec<String> = channel.query_with_limit_and_offset(&sf.collection, &sf.bucket, &sf.query, sf.limit.unwrap(), sf.offset.unwrap())?;
        for id_str in ids {
            let id = uuid::Uuid::parse_str(&id_str)?;
            let item = get_item_by_id(id)?;
            items.push(item.unwrap());
        }
    }
    else if sf.offset == None && sf.limit != None {
        let ids: Vec<String> = channel.query_with_limit(&sf.collection, &sf.bucket, &sf.query, sf.limit.unwrap())?;
        for id_str in ids {
            let id = uuid::Uuid::parse_str(&id_str)?;
            let item = get_item_by_id(id)?;
            items.push(item.unwrap());
        }
    }
    else {
        let ids: Vec<String> = channel.query(&sf.collection, &sf.bucket, &sf.query)?;
        for id_str in ids {
            let id = uuid::Uuid::parse_str(&id_str)?;
            let item = get_item_by_id(id)?;
            items.push(item.unwrap());
        }
    }

    Ok(items)
}

fn suggest_with_sonic(sf: SuggestForm) -> Result<Vec<Item>> {
    let configure = env_var_config();

    let channel = SearchChannel::start(configure.sonic_server, configure.sonic_password)?;

    let mut items = Vec::new();

    if sf.limit != None {
        let ids: Vec<String> = channel.suggest_with_limit(&sf.collection, &sf.bucket, &sf.query, sf.limit.unwrap())?;
        for id_str in ids {
            let id = uuid::Uuid::parse_str(&id_str)?;
            let item = get_item_by_id(id)?;
            items.push(item.unwrap());
        }
    }
    else {
        let ids: Vec<String> = channel.suggest(&sf.collection, &sf.bucket, &sf.query)?;
        for id_str in ids {
            let id = uuid::Uuid::parse_str(&id_str)?;
            let item = get_item_by_id(id)?;
            items.push(item.unwrap());
        }
    }

    Ok(items)
}

async fn index(mut req: Request<()>) -> tide::Result {
    let token_value = req.header("authorization");
    match token_value {
        Some(token_header) => {
            let token = token_header.last().to_string();
            let jwt_value = jwt_verify(token).await?;
            match jwt_value {
                Some(_) => {
                    let r =  req.body_string().await?;
                    let index_form : IndexForm = serde_json::from_str(&r)?;
                    let items = index_form.items;
                    puts_items(items.clone())?;
                    index_with_sonic(items.clone())?;
                    Ok(tide::Response::builder(200).header("content-type", "application/json").build())
                },
                None => { Ok(tide::Response::builder(401).header("content-type", "application/json").build()) }
            }
        },
        None => { Ok(tide::Response::builder(401).header("content-type", "application/json").build()) }
    }
}

async fn deindex(mut req: Request<()>) -> tide::Result {
    let token_value = req.header("authorization");
    match token_value {
        Some(token_header) => {
            let token = token_header.last().to_string();
            let jwt_value = jwt_verify(token).await?;
            match jwt_value {
                Some(_) => {
                    let r =  req.body_string().await?;
                    let deindex_form : DeindexForm = serde_json::from_str(&r)?;
                    let ids = deindex_form.ids;
                    del_items(ids.clone())?;
                    deindex_with_sonic(ids.clone())?;
                    Ok(tide::Response::builder(200).header("content-type", "application/json").build())
                },
                None => { Ok(tide::Response::builder(401).header("content-type", "application/json").build()) }
            }
        },
        None => { Ok(tide::Response::builder(401).header("content-type", "application/json").build()) }
    }
}

async fn search(mut req: Request<()>) -> tide::Result {
    let token_value = req.header("authorization");
    match token_value {
        Some(token_header) => {
            let token = token_header.last().to_string();
            let jwt_value = jwt_verify(token).await?;
            match jwt_value {
                Some(_) => {
                    let r =  req.body_string().await?;
                    let search_form : SearchForm = serde_json::from_str(&r)?;
                    let result = search_with_sonic(search_form)?;
                    Ok(tide::Response::builder(200).body(json!(result)).header("content-type", "application/json").build())
                },
                None => { Ok(tide::Response::builder(401).header("content-type", "application/json").build()) }
            }
        },
        None => { Ok(tide::Response::builder(401).header("content-type", "application/json").build()) }
    }
}

async fn suggest(mut req: Request<()>) -> tide::Result {
    let token_value = req.header("authorization");
    match token_value {
        Some(token_header) => {
            let token = token_header.last().to_string();
            let jwt_value = jwt_verify(token).await?;
            match jwt_value {
                Some(_) => {
                    let r =  req.body_string().await?;
                    let suggest_form : SuggestForm = serde_json::from_str(&r)?;
                    let result = suggest_with_sonic(suggest_form)?;
                    Ok(tide::Response::builder(200).body(json!(result)).header("content-type", "application/json").build())
                },
                None => { Ok(tide::Response::builder(401).header("content-type", "application/json").build()) }
            }
        },
        None => { Ok(tide::Response::builder(401).header("content-type", "application/json").build()) }
    }
}

#[async_std::main]
async fn main() -> tide::Result<()> {

    let configure = env_var_config();

    let cors = CorsMiddleware::new()
    .allow_methods("GET, POST, OPTIONS".parse::<HeaderValue>().unwrap())
    .allow_headers("authorization".parse::<HeaderValue>().unwrap())
    .allow_origin(Origin::from(configure.origin))
    .allow_credentials(false);
    
    let mut app = tide::new();
    app.with(driftwood::DevLogger);
    app.with(cors);
    app.at("/index").post(index);
    app.at("/deindex").post(deindex);
    app.at("/search").post(search);
    app.at("/suggest").post(suggest);

    let ip = format!("0.0.0.0:{}", configure.port);

    if configure.secure {
        app.listen(
            tide_rustls::TlsListener::build().addrs("0.0.0.0:443").acme(
                AcmeConfig::new()
                    .domains(vec![configure.domain])
                    .cache_dir(configure.certs)
                    .production(),
            ),
        )
        .await?;
    } else {
        app.listen(ip).await?;
    }

    Ok(())
}