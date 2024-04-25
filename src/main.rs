use std::{
    collections::HashMap, intrinsics::mir::Len, sync::{Arc, Mutex}
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use time::{macros::date, Date};
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

#[derive(Clone, Deserialize)]
pub struct NewPerson {
    #[serde(rename = "nome")]
    pub name: String,
    #[serde(rename = "apelido")]
    pub nick: String,
    #[serde(rename = "nascimento")]
    pub birth_date: Date,
    pub stack: Option<Vec<String>>,
}

type AppState = Arc<Mutex<HashMap<Uuid, Person>>>;

#[tokio::main]
async fn main() {
    let mut people: HashMap<Uuid, Person> = HashMap::new();

    let person = Person {
        id: Uuid::now_v7(),
        name: String::from("Marcelo"),
        nick: String::from("m4rc3l0"),
        birth_date: date!(1986 - 03 - 31),
        stack: None,
    };

    println!("{}", person.id);

    people.insert(person.id, person);
    // HashMap::insert(&mut people, person.id, person);

    let app_state: AppState = Arc::new(Mutex::new(people));

    // build our application with a single route
    let app = Router::new()
        .route("/people", get(search_people))
        .route("/people/:id", get(find_person))
        .route("/people", post(create_person))
        .route("/people-account", get(people_account))
        .with_state(app_state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn search_people() -> impl IntoResponse {
    return (StatusCode::OK, "Search for people");
}

async fn find_person(
    State(people): State<AppState>,
    Path(person_id): Path<Uuid>,
) -> impl IntoResponse {
    match people.lock().await.get(&person_id) {
        Some(person) => Ok(Json(person.clone())),
        None => Err(StatusCode::NOT_FOUND),
    };
}

async fn create_person(
    Json(new_person): Json<NewPerson>,
    State(people): State<AppState>,
) -> impl IntoResponse {
    let id = Uuid::now_v7();
    let person = Person {
        id,
        name: new_person.name,
        birth_date: new_person.birth_date,
        nick: new_person.nick,
        stack: new_person.stack,
    };
    people.lock().await.insert(id, person.clone());
    return (StatusCode::OK, Json(person));
}

async fn people_account(State(people): State<AppState>) -> impl IntoResponse {
    let count = people.lock().await.len();
    return (StatusCode::OK, Json(count));
}
