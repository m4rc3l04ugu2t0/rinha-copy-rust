use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use time::Date;
use tokio::sync::RwLock;
use uuid::Uuid;

time::serde::format_description!(date_format, Date, "[year]-[month]-[day]");

#[derive(Clone, Serialize)]
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

#[derive(Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct PersonName(String);

impl TryFrom<String> for PersonName {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > 100 {
            return Err("Name is to big");
        } else {
            return Ok(PersonName(value));
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct Nick(String);
impl TryFrom<String> for Nick {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > 32 {
            return Err("nick is to big");
        } else {
            return Ok(Nick(value));
        }
    }
}

#[derive(Deserialize, Clone)]
#[serde(try_from = "String")]
pub struct Tech(String);
impl TryFrom<String> for Tech {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() > 32 {
            return Err("tech is to big");
        } else {
            return Ok(Tech(value));
        }
    }
}

impl From<Tech> for String {
    fn from(value: Tech) -> Self {
        value.0
    }
}

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

type AppState = Arc<RwLock<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .ok()
        .and_then(|port| port.parse::<u16>().ok())
        .unwrap_or(9999);

    let people: HashMap<Uuid, Person> = HashMap::new();

    let app_state: AppState = Arc::new(RwLock::new(people));

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
async fn search_people() -> impl IntoResponse {
    (StatusCode::OK, "Search for people")
}

async fn find_person(
    State(people): State<AppState>,
    Path(person_id): Path<Uuid>,
) -> impl IntoResponse {
    match people.read().await.get(&person_id) {
        Some(person) => Ok(Json(person.clone())),
        None => Err(StatusCode::NOT_FOUND),
    }
}

async fn create_person(
    State(people): State<AppState>,
    Json(new_person): Json<NewPerson>,
) -> impl IntoResponse {
    let id = Uuid::now_v7();
    let person = Person {
        id,
        name: new_person.name.0,
        birth_date: new_person.birth_date,
        nick: new_person.nick.0,
        stack: new_person
            .stack
            .map(|stack| stack.into_iter().map(String::from).collect()),
    };
    people.write().await.insert(id, person.clone());
    (StatusCode::OK, Json(person))
}

async fn people_account(State(people): State<AppState>) -> impl IntoResponse {
    let count = people.read().await.len();
    (StatusCode::OK, Json(count))
}
