use std::collections::HashMap;

use assyst_common::config::{CONFIG, PATREON_REFRESH_LOCATION};
use serde::{Deserialize, Serialize};

use crate::assyst::ThreadSafeAssyst;

pub const REFRESH_ROUTE: &str = "https://www.patreon.com/api/oauth2/token";
pub const ROUTE: &str = "https://api.patreon.com/api/oauth2/v2/campaigns/4568373/members?include=user,currently_entitled_tiers&fields%5Buser%5D=social_connections,full_name&fields%5Bmember%5D=is_follower,last_charge_date,last_charge_status,lifetime_support_cents,currently_entitled_amount_cents,patron_status&page%5Bsize%5D=10000";

pub const TIER_4_AMOUNT: usize = 2000;
pub const TIER_3_AMOUNT: usize = 1000;
pub const TIER_2_AMOUNT: usize = 500;
pub const TIER_1_AMOUNT: usize = 300;

#[derive(Debug, Clone, PartialEq)]
pub enum PatronTier {
    Tier4 = 4,
    Tier3 = 3,
    Tier2 = 2,
    Tier1 = 1,
    Tier0 = 0,
}

impl From<PatronTier> for u64 {
    fn from(val: PatronTier) -> Self {
        match val {
            PatronTier::Tier0 => 0,
            PatronTier::Tier1 => 1,
            PatronTier::Tier2 => 2,
            PatronTier::Tier3 => 3,
            PatronTier::Tier4 => 4,
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Response {
    data: Vec<Datum>,
    included: Vec<Included>,
    meta: Meta,
}

#[derive(Serialize, Deserialize)]
pub struct Datum {
    attributes: DatumAttributes,
    id: String,
    relationships: Relationships,
    #[serde(rename = "type")]
    datum_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct DatumAttributes {
    currently_entitled_amount_cents: usize,
    is_follower: bool,
    last_charge_date: Option<String>,
    last_charge_status: Option<String>,
    lifetime_support_cents: usize,
    patron_status: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Relationships {
    currently_entitled_tiers: CurrentlyEntitledTiers,
    user: User,
}

#[derive(Serialize, Deserialize)]
pub struct CurrentlyEntitledTiers {
    data: Vec<Dat>,
}

#[derive(Serialize, Deserialize)]
pub struct Dat {
    id: String,
    #[serde(rename = "type")]
    dat_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct User {
    data: Dat,
    links: Links,
}

#[derive(Serialize, Deserialize)]
pub struct Links {
    related: String,
}

#[derive(Serialize, Deserialize)]
pub struct Included {
    attributes: IncludedAttributes,
    id: String,
    #[serde(rename = "type")]
    included_type: String,
}

#[derive(Serialize, Deserialize)]
pub struct IncludedAttributes {
    full_name: Option<String>,
    social_connections: Option<SocialConnections>,
}

#[derive(Serialize, Deserialize)]
pub struct SocialConnections {
    deviantart: Option<serde_json::Value>,
    discord: Option<Discord>,
    facebook: Option<serde_json::Value>,
    google: Option<serde_json::Value>,
    instagram: Option<serde_json::Value>,
    reddit: Option<serde_json::Value>,
    spotify: Option<serde_json::Value>,
    twitch: Option<serde_json::Value>,
    twitter: Option<serde_json::Value>,
    youtube: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct Discord {
    url: Option<serde_json::Value>,
    user_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct Meta {
    pagination: Pagination,
}

#[derive(Serialize, Deserialize)]
pub struct Pagination {
    total: i64,
}

#[derive(Debug, Clone)]
pub struct Patron {
    pub user_id: u64,
    pub tier: PatronTier,
    pub _admin: bool,
}

#[derive(Deserialize, Debug)]
pub struct PatronRefreshResponse {
    access_token: String,
    refresh_token: String,
}

async fn get_patreon_access_token(assyst: ThreadSafeAssyst) -> anyhow::Result<String> {
    let response = assyst
        .reqwest_client
        .post(REFRESH_ROUTE)
        .query(&vec![
            &("refresh_token", &CONFIG.authentication.patreon_refresh),
            &("client_secret", &CONFIG.authentication.patreon_client_secret),
            &("client_id", &CONFIG.authentication.patreon_client_id),
            &("grant_type", &"refresh_token".to_owned()),
        ])
        .send()
        .await?
        .json::<PatronRefreshResponse>()
        .await?;

    std::fs::write(PATREON_REFRESH_LOCATION, response.refresh_token.clone())?;

    Ok(response.access_token)
}

/// I am not proud of this code, but at the same time, I am not proud of Patreon for making such a
/// terrible API
pub async fn get_patrons(assyst: ThreadSafeAssyst) -> anyhow::Result<Vec<Patron>> {
    let mut patrons: Vec<Patron> = vec![];

    let access_token = get_patreon_access_token(assyst.clone()).await?;

    let response = assyst.reqwest_client.get(ROUTE).header(reqwest::header::AUTHORIZATION, &format!("Bearer {access_token}")).send().await?.json::<Response>().await?;

    let mut entitled_tiers: HashMap<String, PatronTier> = HashMap::new();
    let mut discord_connections: HashMap<String, u64> = HashMap::new();

    for d in response.data {
        if let Some(status) = d.attributes.patron_status
            && status == "active_patron"
        {
            let tier = get_tier_from_pledge(d.attributes.currently_entitled_amount_cents);
            entitled_tiers.insert(d.relationships.user.data.id.clone(), tier);
        }
    }

    for i in response.included {
        let id = i.id.clone();
        let discord = i.attributes.social_connections.as_ref().map(|s| s.discord.as_ref().map(|d| d.user_id.clone()));

        if let Some(Some(d)) = discord {
            discord_connections.insert(id, d.parse::<u64>().unwrap());
        };
    }

    for e in entitled_tiers {
        let patron_id = e.0;
        let tier = e.1;

        let discord = discord_connections.get(&patron_id);
        if let Some(d) = discord {
            patrons.push(Patron { user_id: *d, tier, _admin: false });
        };
    }

    Ok(patrons)
}

fn get_tier_from_pledge(pledge: usize) -> PatronTier {
    if pledge >= TIER_4_AMOUNT {
        PatronTier::Tier4
    } else if pledge >= TIER_3_AMOUNT {
        PatronTier::Tier3
    } else if pledge >= TIER_2_AMOUNT {
        PatronTier::Tier2
    } else if pledge >= TIER_1_AMOUNT {
        PatronTier::Tier1
    } else {
        PatronTier::Tier0
    }
}
