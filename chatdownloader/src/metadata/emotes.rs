/*
This module enables a top emotes leaderboard. User chat performances can also represent emotes, so thier avatar and display name are assigned based on the 7tv emotes.
*/
use log::{debug, info};
use std::collections::HashMap;

use crate::_constants::SEVEN_TV_URL;
use crate::_types::clptypes::{MetadataTypes, SevenTVEmote};
use crate::_types::twitchtypes::{ChatMessageFragment, Comment};
use crate::metadata::metadatatrait::AbstractMetadata;
use crate::twitch_utils::TwitchAPIWrapper;

pub struct Emotes {
    seventv_emotes: HashMap<String, SevenTVEmote>,
}

impl Emotes {
    fn get_7tv_emotes_in_fragment(&self, fragment: &ChatMessageFragment) -> Vec<SevenTVEmote> {
        let mut result: Vec<SevenTVEmote> = Vec::new();
        for word in fragment.text.split(' ') {
            if let Some(emote) = self.seventv_emotes.get(word) {
                result.push(emote.clone());
            }
        }
        debug!("Found {} number of 7TV emotes in {}", result.len(), fragment.text);
        result
    }
}

impl AbstractMetadata for Emotes {
    async fn new(_twitch: &TwitchAPIWrapper) -> Self {
        info!("Getting the 7TV channel emotes");
        let response = reqwest::get(SEVEN_TV_URL.clone()).await;
        if response.is_err() {
            info!("Cannot get 7tv emotes");
            return Self {
                seventv_emotes: HashMap::new(),
            };
        }

        let resp_body: serde_json::Value = response.unwrap().json().await.unwrap();
        let mut ret_val = HashMap::new();
        if let Some(raw_emotes) = resp_body["emote_set"]["emotes"].as_array() {
            for raw_emote in raw_emotes {
                let host_url = raw_emote["data"]["host"]["url"].as_str().unwrap();
                let filename = raw_emote["data"]["host"]["files"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|emote| emote["name"].as_str().unwrap().ends_with(".webp"))
                    .max_by_key(|emote| emote["width"].as_i64().unwrap())
                    .unwrap();
                let emote = SevenTVEmote {
                    name: raw_emote["name"].as_str().unwrap().to_owned(),
                    emote_url: format!("https://{}/{}", host_url, filename["name"]),
                };
                ret_val.insert(emote.name.clone(), emote);
            }
        } else {
            info!("Cannot access the required keys to get the emotes");
        }

        debug!("Got {} 7tv emotes", ret_val.len());
        Self {
            seventv_emotes: ret_val,
        }
    }

    fn get_name(&self) -> String {
        "emote".to_string()
    }

    fn get_default_value(&self) -> MetadataTypes {
        MetadataTypes::Bool(false)
    }

    fn get_metadata(
        &self,
        comment: Comment,
        _sequence_no: u32,
    ) -> (String, HashMap<String, MetadataTypes>) {
        let metadata: HashMap<String, MetadataTypes> = comment.message.fragments.iter()
            .flat_map(|fragment| self.get_7tv_emotes_in_fragment(fragment))
            .map(|emote| (emote.name.clone(), MetadataTypes::BasicInfo((emote.name.clone(), emote.emote_url.clone()))))
            .collect();
        (self.get_name(), metadata)
    }
}
