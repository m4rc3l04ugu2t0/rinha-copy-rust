use axum::{
    http::StatusCode, response::IntoResponse, routing::{get, post}, Router
};

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/people", get(search_people))
        .route("/people/:id", get(find_person))
        .route("/people", post(create_person))
        .route("/people-account", get(people_account));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn search_people() -> impl IntoResponse {
    return (StatusCode::OK, "Search for people");
}

async fn find_person() -> impl IntoResponse {
    return (StatusCode::OK, "Search for people");
}

async fn create_person() -> impl IntoResponse {
    return (StatusCode::OK, "Create people");
}

async fn people_account() -> impl IntoResponse {
    return (StatusCode::OK, "People account");
}
