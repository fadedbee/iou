#[macro_use] extern crate rocket;
use std::env;

use rocket::{Rocket, Build, futures};
use rocket::fairing::{self, AdHoc};
use rocket::response::status::Created;
use rocket::serde::{Serialize, Deserialize, json::Json};

use rocket_db_pools::{sqlx, Database, Connection};

use futures::{stream::TryStreamExt, future::TryFutureExt};

#[derive(Database)]
#[database("sqlx")]
struct Db(sqlx::SqlitePool);

#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match Db::fetch(&rocket) {
        Some(db) => match sqlx::migrate!("db/migrations").run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        }
        None => Err(rocket),
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(stage())
}

pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket.attach(Db::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
            .mount("/", routes![index])
    })
}