use crate::DatabaseHandler;

pub enum RestrictedFeature {
    All,
    Command(String),
}
impl From<String> for RestrictedFeature {
    fn from(value: String) -> Self {
        match &value.to_lowercase()[..] {
            "all" => RestrictedFeature::All,
            x => RestrictedFeature::Command(x.to_owned()),
        }
    }
}
impl Into<String> for RestrictedFeature {
    fn into(self) -> String {
        match self {
            RestrictedFeature::All => "all".to_owned(),
            RestrictedFeature::Command(command) => command,
        }
    }
}

pub enum RestrictionType {
    Allow,
    Block,
    Other(String),
}
impl From<String> for RestrictionType {
    fn from(value: String) -> Self {
        match &value.to_lowercase()[..] {
            "allow" => RestrictionType::Allow,
            "block" => RestrictionType::Block,
            other => RestrictionType::Other(other.to_owned()),
        }
    }
}
impl Into<String> for RestrictionType {
    fn into(self) -> String {
        match self {
            RestrictionType::Allow => "allow".to_owned(),
            RestrictionType::Block => "block".to_owned(),
            RestrictionType::Other(other) => other,
        }
    }
}

pub enum RestrictionScope {
    Channel,
    User,
    Role,
    Other(String),
}
impl From<String> for RestrictionScope {
    fn from(value: String) -> Self {
        match &value.to_lowercase()[..] {
            "channel" => RestrictionScope::Channel,
            "role" => RestrictionScope::Role,
            "user" => RestrictionScope::User,
            other => RestrictionScope::Other(other.to_owned()),
        }
    }
}
impl Into<String> for RestrictionScope {
    fn into(self) -> String {
        match self {
            RestrictionScope::Channel => "channel".to_owned(),
            RestrictionScope::Role => "role".to_owned(),
            RestrictionScope::User => "user".to_owned(),
            RestrictionScope::Other(other) => other,
        }
    }
}

/// Primary key: (guild_id, command_name, scope, id)
/// Essentially cannot have duplicate entry which is both 'allow' and 'block'
/// Code validation to check that passed IDs are valid for the scope (i.e., supplied id is a role id
/// if scope is role)
#[derive(sqlx::FromRow, Debug)]
pub struct CommandRestrictionRow {
    pub guild_id: i64,
    pub command_name: String,
    pub r#type: String,
    pub scope: String,
    pub id: i64,
}

pub struct CommandRestriction {
    pub guild_id: u64,
    pub restricted_feature: RestrictedFeature,
    pub restriction_type: RestrictionType,
    pub scope: RestrictionScope,
    pub id: u64,
}
impl CommandRestriction {
    pub async fn get_guild_restrictions(handler: &DatabaseHandler, guild_id: u64) -> anyhow::Result<Vec<Self>> {
        let query = "SELECT * FROM command_restrictions WHERE guild_id = $1";

        Ok(sqlx::query_as::<_, CommandRestrictionRow>(query)
            .bind(guild_id as i64)
            .fetch_all(&handler.pool)
            .await?
            .iter()
            .map(|x| CommandRestriction::from(x))
            .collect::<Vec<_>>())
    }

    pub async fn set_guild_restriction(&self) -> anyhow::Result<()> {
        todo!()
    }
}
impl From<&CommandRestrictionRow> for CommandRestriction {
    fn from(value: &CommandRestrictionRow) -> Self {
        CommandRestriction {
            guild_id: value.guild_id as u64,
            restricted_feature: RestrictedFeature::from(value.command_name.clone()),
            restriction_type: RestrictionType::from(value.r#type.clone()),
            scope: RestrictionScope::from(value.scope.clone()),
            id: value.id as u64,
        }
    }
}
impl Into<CommandRestrictionRow> for CommandRestriction {
    fn into(self) -> CommandRestrictionRow {
        CommandRestrictionRow {
            guild_id: self.guild_id as i64,
            command_name: self.restricted_feature.into(),
            r#type: self.restriction_type.into(),
            scope: self.scope.into(),
            id: self.id as i64,
        }
    }
}
