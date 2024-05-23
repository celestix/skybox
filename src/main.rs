use actix_web::http::header::{
    ContentDisposition, DispositionParam, DispositionType, CONTENT_DISPOSITION,
};
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use entity::file;
use log::LevelFilter;
use migration::{Migrator, MigratorTrait};
use sea_orm::{ActiveModelTrait, Database, DatabaseConnection, EntityTrait, Set};
use serde::Deserialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::{fs::File, io};

mod utils;

#[get("/")]
async fn root() -> impl Responder {
    HttpResponse::Ok().body("Pong!")
}

#[get("/info/{file_id}")]
async fn info(path: web::Path<String>, data: web::Data<DatabaseConnection>) -> impl Responder {
    let file_id: String = path.into_inner();
    let conn = data.get_ref();
    let file_info = file::Entity::find_by_id(&file_id)
        .one(conn)
        .await
        .expect("failed to fetch file info");

    if file_info.is_none() {
        log::info!("\"{}\" file not found on the server.", file_id);
        return HttpResponse::NotFound().body("file not found");
    }

    let file_info = file_info.unwrap();
    HttpResponse::Ok().json(file_info)
}

#[get("/get/{file_id}")]
async fn get(
    req: HttpRequest,
    path: web::Path<String>,
    data: web::Data<DatabaseConnection>,
) -> impl Responder {
    let file_id: String = path.into_inner();
    let conn = data.get_ref();
    let file_info = file::Entity::find_by_id(&file_id)
        .one(conn)
        .await
        .expect("failed to fetch file info");

    if file_info.is_none() {
        log::info!("\"{}\" file not found on the server.", file_id);
        return HttpResponse::NotFound().body("file not found");
    }

    let f = actix_files::NamedFile::open_async(format!("db/{}", file_id))
        .await
        .unwrap()
        .set_content_disposition(ContentDisposition {
            disposition: DispositionType::Attachment,
            parameters: vec![DispositionParam::Filename(file_info.unwrap().name)],
        });
    f.into_response(&req)
}

#[derive(Debug, Deserialize)]
pub struct SaveFileQuery {
    name: Option<String>,
}

#[post("/save")]
async fn save(
    req: HttpRequest,
    query: web::Query<SaveFileQuery>,
    b: web::Bytes,
    token: web::Data<String>,
    data: web::Data<DatabaseConnection>,
) -> impl Responder {
    if !utils::auth::check_auth(token.to_string(), &req) {
        return HttpResponse::Unauthorized().body("unauthorized");
    }
    let sfq = query.into_inner();
    let file_name: String;
    if sfq.name.is_none() {
        let hv = req.headers().get(CONTENT_DISPOSITION);
        if hv.is_none() {
            return HttpResponse::BadRequest().body("file name not provided");
        }
        let cd = ContentDisposition::from_raw(hv.unwrap()).unwrap();
        let cd_file_name = cd.get_filename();
        if cd_file_name.is_none() {
            return HttpResponse::BadRequest().body("file name not provided");
        }
        file_name = cd_file_name.unwrap().to_string();
    } else {
        file_name = sfq.name.unwrap();
    }
    if file_name.is_empty() {
        return HttpResponse::BadRequest().body("file name is empty");
    }
    let file_id = utils::hash::generate();
    let file_path = format!("db/{}", &file_id);
    let mut f = File::create(&file_path).await.unwrap();
    let mut chunk = b.chunks(32 * 1024);
    let mut fsize: u64 = 0;
    while let Some(mut c) = chunk.next() {
        match io::copy(&mut c, &mut f).await {
            Ok(siz) => {
                fsize = siz;
            }
            Err(err) => {
                log::error!("failed to copy file: {:?}", err);
                let del = tokio::fs::remove_file(&file_path).await;
                if let Err(err) = del {
                    log::error!("failed to delete file: {:?}", err);
                }
                return HttpResponse::BadRequest().body("failed to save file");
            }
        }
    }
    if fsize == 0 {
        let del = tokio::fs::remove_file(&file_path).await;
        if let Err(err) = del {
            log::error!("failed to delete file: {:?}", err);
        }
        return HttpResponse::BadRequest().body("file is empty");
    }
    let conn = data.get_ref();
    let mut created: Option<i64> = None;
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(t) => {
            created = Some(t.as_secs() as i64);
        }
        Err(err) => {
            log::error!("failed to get current time: {:?}", err);
        }
    }
    file::ActiveModel {
        id: Set(file_id.clone()),
        name: Set(file_name),
        size: Set(fsize as i64),
        created: Set(created),
        ..Default::default()
    }
    .insert(conn)
    .await
    .expect("failed to add store file.");
    HttpResponse::Ok().body(file_id)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("SkyBox: A blazingly fast file server.");
    env_logger::builder()
    .filter_level(LevelFilter::Info)
    .filter_module("sqlx::query", LevelFilter::Off)
    .filter_module("actix_server", LevelFilter::Off)
    .init();
    log::info!("Starting...");
    log::info!("Loading env vars...");
    dotenv::dotenv().ok();
    let config = utils::config::Config::parse();
    log::info!("Connecting to database...");
    let db = Database::connect(config.DATABASE_URI)
        .await
        .expect("failed to connect to db");
    log::info!("Running database migration script...");
    Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");
    log::info!("Listening on PORT {:?}", config.PORT);
    HttpServer::new(move || {
        App::new()
            .service(root)
            .service(save)
            .service(info)
            .service(get)
            .app_data(web::Data::new(db.clone()))
            .app_data(web::Data::new(config.PRIVATE_TOKEN.clone()))
            .app_data(web::PayloadConfig::new(config.MAX_FILE_SIZE))
            .wrap(Logger::new("%a %r %s %Dms"))
    })
    .bind(("127.0.0.1", config.PORT))?
    .run()
    .await
}
