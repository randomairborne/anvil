use simpleinterpolation::Interpolation;
use twilight_model::{
    channel::{message::MessageFlags, ChannelType},
    guild::Permissions,
    id::{marker::GuildMarker, Id},
};
use xpd_common::{
    GuildConfig, DEFAULT_MAX_XP_PER_MESSAGE, DEFAULT_MIN_XP_PER_MESSAGE, TEMPLATE_VARIABLES,
};
use xpd_database::{Database, UpdateGuildConfig};

use crate::{
    cmd_defs::{
        config::{ConfigCommandLevels, ConfigCommandRewards},
        ConfigCommand,
    },
    Error, SlashState, XpdSlashResponse,
};

pub async fn process_config(
    command: ConfigCommand,
    guild: Id<GuildMarker>,
    state: SlashState,
) -> Result<XpdSlashResponse, Error> {
    match command {
        ConfigCommand::Reset(_) => reset_config(state, guild).await,
        ConfigCommand::Get(_) => state
            .query_guild_config(guild)
            .await
            .map(|v| v.unwrap_or_default().to_string())
            .map_err(Into::into),
        ConfigCommand::Rewards(r) => process_rewards_config(state, guild, r).await,
        ConfigCommand::Levels(l) => process_levels_config(state, guild, l).await,
        ConfigCommand::PermsCheckup(_) => process_perm_checkup(state, guild).await,
    }
    .map(|s| XpdSlashResponse::with_embed_text(s).flags(MessageFlags::EPHEMERAL))
}

async fn process_rewards_config(
    state: SlashState,
    guild_id: Id<GuildMarker>,
    options: ConfigCommandRewards,
) -> Result<String, Error> {
    let new_cfg = UpdateGuildConfig::new().one_at_a_time(options.one_at_a_time);
    let config = state
        .query_update_guild_config(guild_id, new_cfg, validate_config)
        .await?;
    state.update_config(guild_id, config).await;
    Ok("Updated rewards config!".to_string())
}

async fn process_levels_config(
    state: SlashState,
    guild_id: Id<GuildMarker>,
    options: ConfigCommandLevels,
) -> Result<String, Error> {
    if let Some(interp_template) = options.level_up_message.as_ref() {
        if interp_template.len() > 512 {
            return Err(Error::LevelUpMessageTooLong);
        }
        let interp = Interpolation::new(interp_template.clone())?;
        for item in interp.variables_used() {
            if !TEMPLATE_VARIABLES.contains(&item) {
                return Err(Error::UnknownInterpolationVariable(item.to_string()));
            }
        }
    }

    if options
        .level_up_channel
        .as_ref()
        .is_some_and(|v| !matches!(v.kind, ChannelType::GuildText))
    {
        return Err(Error::LevelUpChannelMustBeText);
    }

    let max_xp_per_message = safecast_to_i16(options.max_xp_per_message)?;
    let min_xp_per_message = safecast_to_i16(options.min_xp_per_message)?;
    let message_cooldown = safecast_to_i16(options.message_cooldown)?;
    
    let new_cfg = UpdateGuildConfig {
        level_up_message: options.level_up_message,
        level_up_channel: options.level_up_channel.map(|v| v.id),
        ping_users: options.ping_users,
        max_xp_per_message,
        min_xp_per_message,
        message_cooldown,
        one_at_a_time: None,
    };
    let config = state.query_update_guild_config(guild_id, new_cfg, validate_config).await?;
    let msg = config.to_string();
    state.update_config(guild_id, config).await;

    Ok(msg)
}

fn safecast_to_i16(ou16: Option<i64>) -> Result<Option<i16>, Error> {
    ou16.map(TryInto::try_into).transpose().map_err(Into::into)
}

async fn reset_config(state: SlashState, guild_id: Id<GuildMarker>) -> Result<String, Error> {
    state.query_delete_guild_config(guild_id).await?;
    state.update_config(guild_id, GuildConfig::default()).await;
    Ok("Reset guild reward config, but NOT rewards themselves!".to_string())
}

fn validate_config(config: &GuildConfig) -> Result<(), GuildConfigErrorReport> {
    let max_xp_per_msg = config
        .max_xp_per_message
        .unwrap_or(DEFAULT_MAX_XP_PER_MESSAGE);
    let min_xp_per_msg = config
        .min_xp_per_message
        .unwrap_or(DEFAULT_MIN_XP_PER_MESSAGE);
    if max_xp_per_msg < min_xp_per_msg {
        return Err(GuildConfigErrorReport::MinXpIsMoreThanMax {
            min: min_xp_per_msg,
            max: max_xp_per_msg,
        });
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum GuildConfigErrorReport {
    #[error("The selected minimum XP value of {min} is more than the selected maximum of {max}")]
    MinXpIsMoreThanMax { min: i16, max: i16 },
}

impl From<GuildConfigErrorReport> for xpd_database::Error {
    fn from(value: GuildConfigErrorReport) -> Self {
        Self::Validate(value.to_string())
    }
}

async fn process_perm_checkup(
    state: SlashState,
    guild_id: Id<GuildMarker>,
) -> Result<String, Error> {
    let config = state
        .query_guild_config(guild_id)
        .await?
        .unwrap_or_default();
    let can_msg_in_level_up = if let Some(level_up) = config.level_up_channel {
        let perms = state
            .cache
            .permissions()
            .in_channel(state.my_id.cast(), level_up)?;
        Some(perms.contains(Permissions::SEND_MESSAGES))
    } else {
        None
    };
    let can_assign_roles = config;
    // TODO: Finish this
    Ok("Perm checkup is not implemented yet".to_string())
}
