use nih_plug::prelude::*;

pub const NAME: &str = "Cantrip Compressor";
pub const VENDOR: &str = "flathill404";
pub const URL: &str = env!("CARGO_PKG_HOMEPAGE");
pub const EMAIL: &str = "38638577+flathill404@users.noreply.github.com";
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const CLAP_ID: &str = "com.flathill404.grimoire.cantrip_compressor";
pub const CLAP_DESCRIPTION: Option<&str> = Some("Simple Compressor");
pub const CLAP_MANUAL_URL: Option<&str> = Some(URL);
pub const CLAP_SUPPORT_URL: Option<&str> = None;
pub const CLAP_FEATURES: &[ClapFeature] = &[
    ClapFeature::AudioEffect,
    ClapFeature::Compressor,
    ClapFeature::Stereo,
];

pub const VST3_CLASS_ID: [u8; 16] = *b"hCmpVdKz609ecZKi";
pub const VST3_SUBCATEGORIES: &[Vst3SubCategory] =
    &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
