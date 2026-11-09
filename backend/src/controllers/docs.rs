use crate::error::AppError;
use axum::Json;
use axum::extract::Path;
use serde::Serialize;

#[derive(Serialize)]
pub struct DocEntry {
    pub slug: String,
    pub title: String,
}

#[derive(Serialize)]
pub struct DocContent {
    pub slug: String,
    pub title: String,
    pub content: String,
}

static DOCS: &[(&str, &str, &str)] = &[
    (
        "backend",
        "Axum Backend",
        include_str!("../../../docs/backend.md"),
    ),
    (
        "frontend",
        "React Frontend",
        include_str!("../../../docs/frontend.md"),
    ),
    (
        "deployment",
        "Deployment",
        include_str!("../../../docs/deployment.md"),
    ),
];

pub async fn list_docs() -> Json<Vec<DocEntry>> {
    let entries = DOCS
        .iter()
        .map(|(slug, title, _)| DocEntry {
            slug: slug.to_string(),
            title: title.to_string(),
        })
        .collect();
    Json(entries)
}

pub async fn get_doc(Path(slug): Path<String>) -> Result<Json<DocContent>, AppError> {
    let (s, title, content) = DOCS
        .iter()
        .find(|(s, _, _)| *s == slug)
        .ok_or_else(|| AppError::NotFound(format!("Doc '{slug}' not found")))?;

    Ok(Json(DocContent {
        slug: s.to_string(),
        title: title.to_string(),
        content: content.to_string(),
    }))
}
