use crate::routes::diary::AuthForm;
use axum::response::IntoResponse;
use axum::Form;

pub async fn login(Form(form): Form<AuthForm>) -> impl IntoResponse {
    todo!()
}
