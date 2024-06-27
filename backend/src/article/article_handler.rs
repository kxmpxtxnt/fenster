use anyhow::Result;
use axum::extract::{Path, State};
use axum::Json;

use crate::AppInject;
use crate::article::{article_entity, article_entity::Article};
use crate::article::article_entity::CreateArticle;
use crate::fenster_error::FensterError;
use crate::fenster_error::FensterError::{Conflict, NotFound, Unauthorized};
use crate::user::user_entity;

pub async fn get_article(
    State(AppInject { postgres_pool, .. }): State<AppInject>,
    Path(slug): Path<String>,
) -> Result<Json<Article>, FensterError> {
    if !article_entity::exists(&slug, &postgres_pool).await? {
        return Err(NotFound(format!("Article with given slug ({}) does not exist.", slug)));
    }

    let article = article_entity::fetch(&slug, &postgres_pool).await?;
    Ok(Json(article))
}

pub async fn create_article(
    State(AppInject { postgres_pool, .. }): State<AppInject>,
    Json(create): Json<CreateArticle>,
) -> Result<Json<Article>, FensterError> {
    if article_entity::exists(&create.slug.as_str(), &postgres_pool).await? {
        return Err(Conflict(format!("Article with given slug ({}) already exists.", create.slug)));
    }

    if !user_entity::exists_id(&create.author.as_str(), &postgres_pool).await? {
        return Err(NotFound(format!("Author with given id ({}) does not exist.", create.author)));
    }

    let user = user_entity::fetch(&create.author.as_str(), &postgres_pool).await?;

    if !user.author {
        return Err(Unauthorized(format!("User with given id ({}) is not a author.", user.id)));
    }

    let article = Article {
        slug: create.slug,
        title: create.title,
        content: create.content,
        author: user,
        published: create.published,
    };

    article.store(&postgres_pool).await?;

    Ok(Json(article))
}