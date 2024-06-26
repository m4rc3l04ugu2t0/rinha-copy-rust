use std::{env, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use persistence::PostgresRepository;
use serde::{Deserialize, Serialize};
use time::Date;
use uuid::Uuid;

mod persistence;

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Clone, Serialize, sqlx::FromRow)]
pub struct Person {
    pub id: Uuid,
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>,
}

macro_rules! new_string_type {
    ($type:ident, max_length = $max_len:expr, error = $error_msg:expr) => {
        #[derive(Deserialize, Clone)]
        #[serde(try_from = "String")]
        pub struct $type(String);
        impl TryFrom<String> for $type {
            type Error = &'static str;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                if value.len() > $max_len {
                    Err($error_msg)
                } else {
                    Ok($type(value))
                }
            }
        }
        impl From<$type> for String {
            fn from(value: $type) -> Self {
                value.0
            }
        }
    };
}

new_string_type!(PersonName, max_length = 100, error = "name is too big");
new_string_type!(Nick, max_length = 32, error = "nick is too big");
new_string_type!(Tech, max_length = 32, error = "tech is too big too");

#[derive(Clone, Deserialize)]
pub struct NewPerson {
    #[serde(rename = "nome")]
    pub name: PersonName,
    #[serde(rename = "apelido")]
    pub nick: Nick,
    #[serde(rename = "nascimento", with = "date_format")]
    pub birth_date: Date,
    pub stack: Option<Vec<Tech>>,
}

type AppState = Arc<PostgresRepository>;

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(9999);

    let url = env::var("DATABASE_URL")
        .unwrap_or(String::from("postgress://rinha:rinha@localhost:5432/rinha"));

    let repo = PostgresRepository::connect(url).await;

    let app_state = Arc::new(repo);

    // build our application with a single route
    let app = Router::new()
        .route("/people", get(search_people))
        .route("/people/:id", get(find_person))
        .route("/people", post(create_person))
        .route("/people-account", get(people_account))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct PersonSearchQuery {
    #[serde(rename = "t")]
    query: String,
}

async fn search_people(
    State(people): State<AppState>,
    Query(PersonSearchQuery { query }): Query<PersonSearchQuery>,
) -> impl IntoResponse {
    match people.search_people(query).await {
        Ok(people) => Ok(Json(people)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn find_person(
    State(people): State<AppState>,
    Path(person_id): Path<Uuid>,
) -> impl IntoResponse {
    match people.find_person(person_id).await {
        Ok(Some(person)) => Ok(Json(person)),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn create_person(
    State(people): State<AppState>,
    Json(new_person): Json<NewPerson>,
) -> impl IntoResponse {
    match people.create_person(new_person).await {
        Ok(person) => Ok((StatusCode::CREATED, Json(person))),
        Err(sqlx::Error::Database(err)) if err.is_unique_violation() => {
            Err(StatusCode::UNPROCESSABLE_ENTITY)
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn people_account(State(people): State<AppState>) -> impl IntoResponse {
    match people.count_people().await {
        Ok(count) => Ok(Json(count)),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}
