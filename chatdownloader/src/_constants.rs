/*
File of constants used throughout the project.
*/
use lazy_static::lazy_static;

pub const VED_CH_ID: &str = "85498365";

pub const USER_AGENT: &str = concat!(
    "neuro-chat-elo/0.1 ",
    env!("CARGO_PKG_NAME"),
    "/",
    env!("CARGO_PKG_VERSION"),
    " (https://vanorsigma.github.io/neuro-chat-elo)"
);

lazy_static! {
    pub static ref SEVEN_TV_URL: String = format!("https://7tv.io/v3/users/twitch/{}", VED_CH_ID);
}