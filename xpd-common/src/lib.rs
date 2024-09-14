#![deny(clippy::all, clippy::pedantic, clippy::nursery)]

use std::{
    borrow::Cow,
    fmt::{Debug, Display, Formatter},
    str::FromStr,
};

use simpleinterpolation::Interpolation;
use twilight_cache_inmemory::ResourceType;
use twilight_gateway::EventTypeFlags;
use twilight_model::{
    gateway::Intents,
    guild::Member,
    id::{
        marker::{ChannelMarker, GuildMarker, RoleMarker, UserMarker},
        Id,
    },
    user::User,
    util::ImageHash,
};

pub trait DisplayName {
    #[must_use]
    fn display_name(&self) -> &str;
}

impl DisplayName for User {
    fn display_name(&self) -> &str {
        self.global_name.as_ref().unwrap_or(&self.name)
    }
}

impl DisplayName for Member {
    fn display_name(&self) -> &str {
        self.nick
            .as_deref()
            .unwrap_or_else(|| self.user.display_name())
    }
}

impl DisplayName for MemberDisplayInfo {
    fn display_name(&self) -> &str {
        self.nick.as_ref().map_or_else(
            || {
                self.global_name
                    .as_ref()
                    .map_or(self.name.as_str(), |global| global.as_str())
            },
            |nick| nick.as_str(),
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemberDisplayInfo {
    pub id: Id<UserMarker>,
    pub name: String,
    pub global_name: Option<String>,
    pub nick: Option<String>,
    pub avatar: Option<ImageHash>,
    pub local_avatar: Option<ImageHash>,
    pub bot: bool,
}

impl From<User> for MemberDisplayInfo {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            name: value.name,
            global_name: value.global_name,
            nick: None,
            avatar: value.avatar,
            local_avatar: None,
            bot: value.bot,
        }
    }
}

impl From<Member> for MemberDisplayInfo {
    fn from(value: Member) -> Self {
        Self {
            id: value.user.id,
            name: value.user.name,
            global_name: value.user.global_name,
            nick: value.nick,
            avatar: value.user.avatar,
            local_avatar: value.avatar,
            bot: value.user.bot,
        }
    }
}

impl MemberDisplayInfo {
    #[must_use]
    pub fn with_nick(self, nick: Option<String>) -> Self {
        Self { nick, ..self }
    }
}

/// Get environment variable and parse it, panicking on failure
/// # Panics
/// If the environment variable cannot be found or parsed
#[must_use]
pub fn parse_var<T>(key: &str) -> T
where
    T: FromStr,
    T::Err: Display,
{
    get_var(key)
        .parse()
        .unwrap_or_else(|e| panic!("{key} could not be parsed: {e}"))
}

/// Get environment variable and parse it, panicking on failure
/// # Panics
/// If the environment variable cannot be found or parsed
#[must_use]
pub fn get_var(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|e| panic!("Expected {key} in environment: {e}"))
}

pub const TEMPLATE_VARIABLES: [&str; 2] = ["user_mention", "level"];
pub const DEFAULT_MAX_XP_PER_MESSAGE: i16 = 25;
pub const DEFAULT_MIN_XP_PER_MESSAGE: i16 = 15;
pub const DEFAULT_MESSAGE_COOLDOWN: i16 = 60;

#[derive(Default, Debug)]
pub struct GuildConfig {
    pub one_at_a_time: Option<bool>,
    pub level_up_message: Option<Interpolation>,
    pub level_up_channel: Option<Id<ChannelMarker>>,
    pub ping_on_level_up: Option<bool>,
    pub min_xp_per_message: Option<i16>,
    pub max_xp_per_message: Option<i16>,
    pub cooldown: Option<i16>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct UserStatus {
    pub id: Id<UserMarker>,
    pub guild: Id<GuildMarker>,
    pub xp: i64,
}

#[derive(Debug)]
pub struct RoleReward {
    pub id: Id<RoleMarker>,
    pub requirement: i64,
}

#[must_use]
#[inline]
pub fn sort_rewards(a: &RoleReward, b: &RoleReward) -> std::cmp::Ordering {
    a.requirement.cmp(&b.requirement)
}

#[inline]
const fn tribool(data: Option<bool>, default: Option<bool>) -> &'static str {
    match (data, default) {
        (None, None) => "unset",
        (Some(true), _) | (None, Some(true)) => "true",
        (Some(false), _) | (None, Some(false)) => "false",
    }
}

fn opt_code_str(data: Option<&str>) -> Cow<str> {
    data.map_or(Cow::Borrowed("unset"), |v| Cow::Owned(format!("`{v}`")))
}

fn opt_mention_str<T>(data: Option<Id<T>>, mention_kind: char) -> Cow<'static, str> {
    data.map_or(Cow::Borrowed("unset"), |v| {
        Cow::Owned(format!("`<{mention_kind}{v}>`"))
    })
}

/// Convert a discord message ID to a seconds value of when it was sent relative to the discord epoch
#[must_use]
pub fn snowflake_to_timestamp<T>(id: Id<T>) -> i64 {
    // this is safe, because dividing an u64 by 1000 ensures it is a valid i64
    ((id.get() >> 22) / 1000).try_into().unwrap_or(0)
}

impl Display for GuildConfig {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "One reward role at a time: {}",
            tribool(self.one_at_a_time, Some(false))
        )?;
        writeln!(
            f,
            "Level-up message: {}",
            opt_code_str(
                self.level_up_message
                    .as_ref()
                    .map(Interpolation::input_value)
                    .as_deref()
            )
        )?;
        writeln!(
            f,
            "Level-up channel: {}",
            opt_mention_str(self.level_up_channel, '#')
        )?;
        writeln!(
            f,
            "Maximum XP per message: {}",
            self.max_xp_per_message
                .unwrap_or(DEFAULT_MAX_XP_PER_MESSAGE)
        )?;
        writeln!(
            f,
            "Minimum XP per message: {}",
            self.min_xp_per_message
                .unwrap_or(DEFAULT_MIN_XP_PER_MESSAGE)
        )?;
        write!(
            f,
            "Cooldown (seconds): {}",
            self.cooldown.unwrap_or(DEFAULT_MESSAGE_COOLDOWN)
        )?;
        Ok(())
    }
}

pub trait RequiredDiscordResources {
    fn required_intents() -> Intents;
    fn required_events() -> EventTypeFlags;
    fn required_cache_types() -> ResourceType;
}
