use std::fmt::Debug;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::entities::user;
use crate::entities::user::User;
use crate::handlers::FensterError;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Article {
    pub(crate) slug: String,
    pub(crate) title: String,
    pub(crate) content: String,
    pub(crate) author: User,
    pub(crate) published: bool,
}

impl Article {
    pub async fn store(&self, pool: &PgPool) -> Result<bool, FensterError> {
        let result = sqlx::query!(
            "INSERT INTO fenster.public.articles
            (article_slug, article_title, article_content, article_author, article_published, editing_date)
            VALUES($1, $2, $3, $4, $5,  NOW())",
            &self.slug, &self.title, &self.content, &self.author.id, &self.published)
            .execute(pool)
            .await.map_err(|_| FensterError::Internal)?;

        Ok(result.rows_affected() != 0)
    }
}

pub async fn exists(slug: &str, pool: &PgPool) -> Result<bool, FensterError> {
    let result = sqlx::query!(
        "SELECT EXISTS (SELECT article_title) FROM fenster.public.articles WHERE article_slug=$1", slug)
        .fetch_one(pool)
        .await.map_err(|_| FensterError::Internal)?;


    Ok(result.exists.unwrap_or(false))
}

pub async fn fetch(slug: &str, pool: &PgPool) -> Result<Article, FensterError> {
    let article = sqlx::query!(
        "SELECT article_slug, article_title, article_content, article_author, article_published
         FROM fenster.public.articles WHERE article_slug=$1", slug)
        .fetch_one(pool)
        .await.map_err(|_| FensterError::Internal)?;


    let user = user::fetch(article.article_author.as_str(), pool).await?;

    Ok(Article {
        slug: article.article_slug,
        title: article.article_title,
        content: article.article_content,
        author: user,
        published: article.article_published,
    })
}