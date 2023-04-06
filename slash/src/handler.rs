use axum::{body::Bytes, extract::State, http::HeaderMap, response::IntoResponse, Json};
use twilight_model::{
    application::interaction::{Interaction, InteractionType},
    channel::message::MessageFlags,
    http::interaction::{InteractionResponse, InteractionResponseType},
};
use twilight_util::builder::InteractionResponseDataBuilder;

use crate::AppState;

pub async fn handle(
    headers: HeaderMap,
    State(state): State<AppState>,
    body: Bytes,
) -> Result<axum::Json<InteractionResponse>, IncomingInteractionError> {
    let body = body.to_vec();
    crate::discord_sig_validation::validate_discord_sig(&headers, &body, &state.pubkey)?;
    let interaction: Interaction = serde_json::from_slice(&body)?;
    if matches!(interaction.kind, InteractionType::Ping) {
        return Ok(Json(InteractionResponse {
            kind: InteractionResponseType::Pong,
            data: None,
        }));
    }
    tokio::spawn(async move {
        let interaction_token = interaction.token.clone();
        let interaction_id = interaction.id;
        let resp = match crate::processor::process(interaction, state.clone()).await {
            Ok(val) => val,
            Err(e) => {
                error!("{e}");
                InteractionResponse {
                    kind: InteractionResponseType::ChannelMessageWithSource,
                    data: Some(
                        InteractionResponseDataBuilder::new()
                            .flags(MessageFlags::EPHEMERAL)
                            .content(e.to_string())
                            .build(),
                    ),
                }
            }
        };
        if let Err(e) = state
            .client
            .interaction(state.my_id)
            .create_response(interaction_id, &interaction_token, &resp)
            .await
        {
            warn!("{e:#?}");
        };
    });
    Ok(Json(InteractionResponse {
        kind: InteractionResponseType::DeferredChannelMessageWithSource,
        data: None,
    }))
}

#[derive(thiserror::Error, Debug)]
pub enum IncomingInteractionError {
    #[error("Signature validation error: {0}")]
    Validation(#[from] crate::discord_sig_validation::SignatureValidationError),
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
}

impl IntoResponse for IncomingInteractionError {
    fn into_response(self) -> axum::response::Response {
        error!("{self}");
        axum::response::Response::builder()
            .body(axum::body::boxed(axum::body::Full::from(self.to_string())))
            .unwrap()
    }
}
