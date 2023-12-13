use crate::routes::diary::FetchQuery;
use axum::extract::Query;
use axum::response::IntoResponse;

pub async fn fetch(Query(query): Query<FetchQuery>) -> impl IntoResponse {
    todo!()
}

pub async fn delete() -> impl IntoResponse {
    todo!()
}

pub async fn list() -> impl IntoResponse {
    todo!()
}
