use std::fmt::Debug;

use anyhow::Result;
use log::error;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::fenster_error::{error, FensterError, POSTGRES_ERROR};
use crate::fenster_error::FensterError::Internal;
use crate::user;
use crate::user::user_entity::User;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Article {
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) author: User,
    pub(crate) published: bool,
}

#[derive(Deserialize)]
pub struct CreateArticle {
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) content: String,
    pub(crate) author: String,
    pub(crate) published: bool,
}

impl Article {
    pub async fn store(&self, pool: &PgPool) -> Result<(), FensterError> {
        sqlx::query!(
            "INSERT INTO fenster.public.articles
            (article_slug, article_title, article_content, article_author, article_published, editing_date)
            VALUES($1, $2, $3, $4, $5,  NOW())",
            &self.slug, &self.title, &self.content, &self.author.id, &self.published)
            .execute(pool)
            .await
            .inspect_err(|err| {
                error!("Error while saving article with article_slug ({}). - {}", self.slug, err)
            })
            .map_err(|_| Internal(error(POSTGRES_ERROR, 1)))?;

        Ok(())
    }
}

pub async fn exists(slug: &str, pool: &PgPool) -> Result<bool, FensterError> {
    let result = sqlx::query!(
        "SELECT EXISTS (SELECT article_title) FROM fenster.public.articles WHERE article_slug=$1", slug)
        .fetch_one(pool)
        .await
        .inspect_err(|err| {
            error!("Error while finding article with article_slug ({}). - {}", slug, err)
        })
        .map_err(|_| Internal(error(POSTGRES_ERROR, 2)))?;


    Ok(result.exists.unwrap_or(false))
}

pub async fn fetch(slug: &str, pool: &PgPool) -> Result<Article, FensterError> {
    let article_result = sqlx::query!(
        "SELECT article_slug, article_title, article_content, article_author, article_published
         FROM fenster.public.articles WHERE article_slug=$1", slug)
        .fetch_one(pool)
        .await
        .inspect_err(|err| {
            error!("Error while fetching article with article_slug ({}). - {}", slug, err)
        })
        .map_err(|_| Internal(error(POSTGRES_ERROR, 3)))?;

    let user_result = user::user_entity::fetch(article_result.article_author.as_str(), pool).await?;

    Ok(Article {
        slug: article_result.article_slug,
        title: article_result.article_title,
        content: article_result.article_content,
        author: user_result,
        published: article_result.article_published,
    })
}