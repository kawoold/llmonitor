use axum::response::Json;

use crate::proxy::types::ModelOption;

pub async fn get_models() -> Json<Vec<ModelOption>> {
    Json(vec![
        ModelOption {
            provider: "anthropic",
            model: "claude-opus-4-7",
            display_name: "Claude Opus 4.7",
        },
        ModelOption {
            provider: "anthropic",
            model: "claude-sonnet-4-6",
            display_name: "Claude Sonnet 4.6",
        },
        ModelOption {
            provider: "anthropic",
            model: "claude-haiku-4-5-20251001",
            display_name: "Claude Haiku 4.5",
        },
    ])
}
