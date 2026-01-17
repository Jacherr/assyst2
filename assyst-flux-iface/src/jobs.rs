use std::collections::HashMap;
use std::time::Duration;

use assyst_common::util::string_from_likely_utf8;
use serde::Deserialize;
use serde_json::from_str;

use super::FluxHandler;
use super::flux_request::FluxRequest;

#[derive(Deserialize)]
pub struct ImageInfo {
    pub file_size_bytes: u64,
    pub mime_type: String,
    pub dimensions: String,
    pub frame_count: Option<u64>,
    pub repeat: Option<String>,
    pub comments: Vec<String>,
}

#[derive(Deserialize)]
pub struct VideoInfo {
    pub file_size_bytes: u64,
    pub mime_type: String,
    pub dimensions: String,
    pub duration_ms: u64,
    pub frame_count: u64,
    pub fps: f64,
}

#[derive(Deserialize)]
pub enum MediaInfo {
    Image(ImageInfo),
    Video(VideoInfo),
}

pub type FluxResult = anyhow::Result<Vec<u8>>;

impl FluxHandler {
    pub async fn ahshit(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "ah-shit");

        self.run_flux(request, limits.time).await
    }

    pub async fn aprilfools(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "april-fools");

        self.run_flux(request, limits.time).await
    }

    pub async fn back_tattoo(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "back-tattoo");

        self.run_flux(request, limits.time).await
    }

    pub async fn billboard(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "billboard");

        self.run_flux(request, limits.time).await
    }

    pub async fn bloom(
        &self,
        media: Vec<u8>,
        radius: Option<u64>,
        sharpness: Option<u64>,
        brightness: Option<u64>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        if let Some(r) = radius {
            options.insert("radius".to_owned(), r.to_string());
        };
        if let Some(s) = sharpness {
            options.insert("sharpness".to_owned(), s.to_string());
        };
        if let Some(b) = brightness {
            options.insert("brightness".to_owned(), b.to_string());
        };

        request.operation("bloom".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn blur(&self, media: Vec<u8>, power: Option<f32>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        if let Some(p) = power {
            options.insert("strength".to_owned(), p.to_string());
        };

        request.operation("blur".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn book(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "book");

        self.run_flux(request, limits.time).await
    }

    pub async fn caption(
        &self,
        media: Vec<u8>,
        text: String,
        bottom: bool,
        black: bool,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        options.insert("text".to_owned(), text);
        if bottom {
            options.insert("bottom".to_owned(), "1".to_owned());
        }
        if black {
            options.insert("black".to_owned(), "1".to_owned());
        }

        request.operation("caption".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn circuitboard(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "circuitboard");

        self.run_flux(request, limits.time).await
    }

    pub async fn deepfry(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "deepfry");

        self.run_flux(request, limits.time).await
    }

    pub async fn drip(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "drip");

        self.run_flux(request, limits.time).await
    }

    pub async fn femurbreaker(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "femurbreaker");

        self.run_flux(request, limits.time).await
    }

    pub async fn fisheye(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "fisheye");

        self.run_flux(request, limits.time).await
    }

    pub async fn flag(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "flag");

        self.run_flux(request, limits.time).await
    }

    pub async fn flag2(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "flag2");

        self.run_flux(request, limits.time).await
    }

    pub async fn flip(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "flip");

        self.run_flux(request, limits.time).await
    }

    pub async fn flop(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "flop");

        self.run_flux(request, limits.time).await
    }

    pub async fn fortune_cookie(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "fortune-cookie");

        self.run_flux(request, limits.time).await
    }

    pub async fn frame_shift(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "frame-shift");

        self.run_flux(request, limits.time).await
    }

    pub async fn frames(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "frames");

        self.run_flux(request, limits.time).await
    }

    pub async fn ghost(&self, media: Vec<u8>, depth: Option<u64>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        if let Some(d) = depth {
            options.insert("depth".to_owned(), d.to_string());
        };

        request.operation("ghost".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn gif(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "gif");

        self.run_flux(request, limits.time).await
    }

    pub async fn gif_magik(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "gif-magik");

        self.run_flux(request, limits.time).await
    }

    pub async fn globe(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "globe");

        self.run_flux(request, limits.time).await
    }

    pub async fn grayscale(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "grayscale");

        self.run_flux(request, limits.time).await
    }

    pub async fn heart_locket(&self, media: Vec<u8>, text: String, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        options.insert("text".to_owned(), text);

        request.operation("heart-locket".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn image_info(&self, media: Vec<u8>) -> anyhow::Result<MediaInfo> {
        let mut request = FluxRequest::default();
        request.input(media);
        request.info();

        let out = self.run_flux(request, Duration::MAX).await?;
        Ok(from_str::<MediaInfo>(&string_from_likely_utf8(out))?)
    }

    pub async fn invert(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "invert");

        self.run_flux(request, limits.time).await
    }

    pub async fn jpeg(&self, media: Vec<u8>, quality: Option<u64>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        if let Some(q) = quality {
            options.insert("quality".to_owned(), q.to_string());
        }

        request.operation("jpeg".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn magik(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "magik");

        self.run_flux(request, limits.time).await
    }

    pub async fn meme(
        &self,
        media: Vec<u8>,
        top: Option<String>,
        bottom: Option<String>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        if let Some(t) = top
            && !t.is_empty()
        {
            options.insert("top".to_owned(), t);
        }
        if let Some(b) = bottom
            && !b.is_empty()
        {
            options.insert("bottom".to_owned(), b);
        }

        request.operation("meme".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn motivate(
        &self,
        media: Vec<u8>,
        top: Option<String>,
        bottom: Option<String>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        if let Some(t) = top
            && !t.is_empty()
        {
            options.insert("top".to_owned(), t);
        }
        if let Some(b) = bottom
            && !b.is_empty()
        {
            options.insert("bottom".to_owned(), b);
        }

        request.operation("motivate".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn neon(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "neon");

        self.run_flux(request, limits.time).await
    }

    pub async fn overlay(&self, media: Vec<u8>, media2: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);
        request.input(media2);

        request.operation("overlay".to_owned(), HashMap::new());
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn paint(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "paint");

        self.run_flux(request, limits.time).await
    }

    pub async fn ping_pong(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "ping-pong");

        self.run_flux(request, limits.time).await
    }

    pub async fn pixelate(
        &self,
        media: Vec<u8>,
        strength: Option<f32>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        if let Some(s) = strength {
            options.insert("strength".to_owned(), s.to_string());
        }

        request.operation("pixelate".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn rainbow(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "rainbow");

        self.run_flux(request, limits.time).await
    }

    pub async fn resize_absolute(
        &self,
        media: Vec<u8>,
        width: u32,
        height: u32,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        options.insert("width".to_owned(), width.to_string());
        options.insert("height".to_owned(), height.to_string());

        request.operation("resize".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn resize_scale(&self, media: Vec<u8>, scale: f32, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        options.insert("scale".to_owned(), scale.to_string());

        request.operation("resize".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn reverse(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "reverse");

        self.run_flux(request, limits.time).await
    }

    pub async fn rotate(
        &self,
        media: Vec<u8>,
        degrees: Option<u64>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        if let Some(d) = degrees {
            options.insert("degrees".to_owned(), d.to_string());
        }

        request.operation("rotate".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn rubiks(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "rubiks");

        self.run_flux(request, limits.time).await
    }

    pub async fn set_loop(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>, loops: i64) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        options.insert("loops".to_owned(), loops.to_string());

        request.operation("set-loop".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn scramble(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "scramble");

        self.run_flux(request, limits.time).await
    }

    pub async fn siren(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "siren");

        self.run_flux(request, limits.time).await
    }

    pub async fn speech_bubble(&self, media: Vec<u8>, solid: bool, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options = HashMap::new();
        if solid {
            options.insert("solid".to_owned(), "1".to_owned());
        }

        request.operation("speech-bubble".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn speed(
        &self,
        media: Vec<u8>,
        multiplier: Option<f64>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options: HashMap<String, String> = HashMap::new();
        if let Some(m) = multiplier {
            options.insert("multiplier".to_owned(), m.to_string());
        };

        request.operation("speed".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn spin(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "spin");

        self.run_flux(request, limits.time).await
    }

    pub async fn spread(
        &self,
        media: Vec<u8>,
        strength: Option<u64>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options: HashMap<String, String> = HashMap::new();
        if let Some(s) = strength {
            options.insert("strength".to_owned(), s.to_string());
        };

        request.operation("spread".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn sweden(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "sweden");

        self.run_flux(request, limits.time).await
    }

    pub async fn swirl(
        &self,
        media: Vec<u8>,
        strength: Option<f32>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options: HashMap<String, String> = HashMap::new();
        if let Some(s) = strength {
            options.insert("strength".to_owned(), s.to_string());
        };

        request.operation("swirl".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn terraria(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "terraria");

        self.run_flux(request, limits.time).await
    }

    pub async fn toaster(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "toaster");

        self.run_flux(request, limits.time).await
    }

    pub async fn uncaption(
        &self,
        media: Vec<u8>,
        amount: Option<String>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options: HashMap<String, String> = HashMap::new();
        if let Some(a) = amount {
            options.insert("amount".to_owned(), a);
        };

        request.operation("uncaption".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn valentine(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "valentine");

        self.run_flux(request, limits.time).await
    }

    pub async fn wormhole(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "wormhole");

        self.run_flux(request, limits.time).await
    }

    pub async fn zoom(&self, media: Vec<u8>, user_id: u64, guild_id: Option<u64>) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let request = FluxRequest::new_basic(media, &limits, "zoom");

        self.run_flux(request, limits.time).await
    }

    pub async fn zoom_blur(
        &self,
        media: Vec<u8>,
        power: Option<f32>,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> FluxResult {
        let limits = self.get_request_limits(user_id, guild_id).await?;

        let mut request = FluxRequest::new_with_input_and_limits(media, &limits);

        let mut options: HashMap<String, String> = HashMap::new();
        if let Some(p) = power {
            options.insert("power".to_owned(), p.to_string());
        };

        request.operation("zoom-blur".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }
}
