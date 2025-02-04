use super::{bad_request, HtmlTemplate, ResultHtml};
use crate::{AppState, State};
use askama::Template;

#[derive(Template)]
#[template(path = "joke/index.html")]
pub struct IndexTemplate {
    joke: String,
}

pub async fn index(State(state): State<AppState>) -> ResultHtml<IndexTemplate> {
    let joke = get_joke(state).await.map_err(bad_request)?;
    let template = IndexTemplate { joke };

    Ok(HtmlTemplate(template))
}

#[derive(Template)]
#[template(path = "joke/show.html")]
pub struct ShowTemplate {
    joke: String,
}

pub async fn renew(State(state): State<AppState>) -> ResultHtml<ShowTemplate> {
    let joke = get_joke(state).await.map_err(bad_request)?;
    let template = ShowTemplate { joke };

    Ok(HtmlTemplate(template))
}

async fn get_joke(state: AppState) -> anyhow::Result<String> {
    let request = state
        .api_client
        .get("https://v2.jokeapi.dev/joke/Any?format=txt&safe-mode");
    let response = request.send().await?;
    let joke = response.text().await?;

    Ok(joke)
}
