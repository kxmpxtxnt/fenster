use anyhow::Result;
use axum::extract::{Path, State};
use axum::Json;
use serde::Deserialize;
use crate::AppInject;

use crate::entities::{article, user};
use crate::entities::article::Article;
use crate::handlers::FensterError;

pub async fn get_article(
    State(AppInject { postgres_pool, .. }): State<AppInject>,
    Path(slug): Path<String>,
) -> Result<Json<Article>, FensterError> {
    article::exists(&slug, &postgres_pool).await.map_err(|_| FensterError::NotFound)?;

    match article::fetch(&slug, &postgres_pool).await {
        Ok(article) => Ok(Json(article)),
        Err(_) => Err(FensterError::Internal)
    }
}

#[derive(Deserialize)]
pub struct CreateArticle {
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) content: String,
    pub(crate) author: String,
    pub(crate) published: bool,
}

pub async fn create_article(
    State(AppInject { postgres_pool, .. }): State<AppInject>,
    Json(create): Json<CreateArticle>,
) -> Result<Json<Article>, FensterError> {
    article::exists(&create.slug, &postgres_pool).await.map(|_| FensterError::Conflict)?;

    user::exists(create.author.as_str(), &postgres_pool).await.map_err(|_| FensterError::NotFound)?;

    let user = user::fetch(&create.author.as_str(), &postgres_pool)
        .await.map_err(|_| FensterError::Internal)?;

    let article = Article {
        slug: create.slug,
        title: create.title,
        content: create.content,
        author: user,
        published: create.published,
    };

    article.store(&postgres_pool).await.map_err(|_| FensterError::Internal)?;

    Ok(Json(article))
}