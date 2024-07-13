use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ChatPerformance {
    pub id: String,
    pub perf_type: PerformanceType,
    pub username: String,
    pub avatar: String,
    pub metrics: HashMap<String, f32>,
    pub metadata: HashMap<String, MetadataTypes>,
}

#[derive(Deserialize, Serialize, PartialEq, Debug, Clone, Copy)]
pub enum PerformanceType {
    User,
    Emote,
    Unknown,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct BadgeInformation {
    pub description: String,
    pub image_url: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub enum MetadataTypes {
    Bool(bool),
    BadgeList(Vec<BadgeInformation>),
    BasicInfo(String, String, PerformanceType),
}

impl MetadataTypes {
    pub fn get_badge_list(&self) -> Option<&Vec<BadgeInformation>> {
        match self {
            MetadataTypes::BadgeList(badge_list) => Some(badge_list),
            _ => None,
        }
    }
    pub fn get_bool(&self) -> Option<&bool> {
        match self {
            MetadataTypes::Bool(b) => Some(b),
            _ => None,
        }
    }
    pub fn get_basic_info(&self) -> Option<(String, String, PerformanceType)> {
        match self {
            MetadataTypes::BasicInfo(username, avatar, perf_type) => {
                Some((username.to_string(), avatar.to_string(), *perf_type))
            }
            _ => None,
        }
    }
}
