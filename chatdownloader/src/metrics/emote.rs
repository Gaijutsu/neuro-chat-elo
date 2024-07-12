/*
The emote metric
*/

use std::collections::HashMap;

use log::{debug, info};

use crate::_constants::SEVEN_TV_URL;
use crate::_types::clptypes::SevenTVEmote;
use crate::_types::twitchtypes::{ChatMessageFragment, Comment};

use super::metrictrait::AbstractMetric;

const WEIGHT_EMOTES: f32 = 0.02;

#[derive(Default, Debug, Clone)]
pub struct Emote {
    seventv_emotes: HashMap<String, SevenTVEmote>,
}

impl Emote {
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

impl AbstractMetric for Emote {
    async fn new() -> Self {
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

    fn can_parallelize(&self) -> bool {
        false
    }

    fn get_name(&self) -> String {
        String::from("emote")
    }

    fn get_metric(
        &mut self,
        comment: Comment,
        _sequence_no: u32,
    ) -> (String, HashMap<String, f32>) {
        let mut emotes: HashMap<String, usize> = HashMap::new();
        let metric = comment
            .message
            .fragments
            .iter()
            .map(|fragment| {
            let mut emote_count = 0;
            for emote in self.get_7tv_emotes_in_fragment(fragment) {
                *emotes.entry(emote.name).or_insert(0) += 1;
                emote_count += 1;
            }
            (fragment.emoticon.is_some() as u16 as f32 + emote_count as f32) * WEIGHT_EMOTES
            })
            .sum();

        let mut result: HashMap<String, f32> = emotes.iter()
            .map(|(emote, count)| (emote.clone(), *count as f32))
            .collect();
        result.insert(comment.commenter._id, metric);

        (self.get_name(), result)
    }
}