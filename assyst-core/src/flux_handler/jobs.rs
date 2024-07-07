use std::collections::HashMap;

use crate::flux_handler::flux_request::FluxRequest;

use super::limits::LIMITS;
use super::FluxHandler;

pub type FluxResult = anyhow::Result<Vec<u8>>;

impl FluxHandler {
    pub async fn ahshit(&self, media: Vec<u8>, user_id: u64) -> FluxResult {
        let tier = self.get_request_tier(user_id).await?;

        let limits = &LIMITS[tier];
        let mut request = FluxRequest::new_with_input_and_limits(media, limits);
        request.operation("ah-shit".to_owned(), HashMap::new());
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn aprilfools(&self, media: Vec<u8>, user_id: u64) -> FluxResult {
        let tier = self.get_request_tier(user_id).await?;

        let mut request = FluxRequest::new();
        request.input(media);
        request.operation("april-fools".to_owned(), HashMap::new());
        request.output();

        self.run_flux(request, LIMITS[tier].time).await
    }

    pub async fn bloom(
        &self,
        media: Vec<u8>,
        radius: Option<u64>,
        sharpness: Option<u64>,
        brightness: Option<u64>,
        user_id: u64,
    ) -> FluxResult {
        let tier = self.get_request_tier(user_id).await?;

        let limits = &LIMITS[tier];
        let mut request = FluxRequest::new_with_input_and_limits(media, limits);

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

    pub async fn blur(&self, media: Vec<u8>, power: Option<f32>, user_id: u64) -> FluxResult {
        let tier = self.get_request_tier(user_id).await?;

        let limits = &LIMITS[tier];
        let mut request = FluxRequest::new_with_input_and_limits(media, limits);

        let mut options = HashMap::new();
        if let Some(p) = power {
            options.insert("strength".to_owned(), p.to_string());
        };

        request.operation("blur".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn caption(&self, media: Vec<u8>, text: String, user_id: u64) -> FluxResult {
        let tier = self.get_request_tier(user_id).await?;

        let limits = &LIMITS[tier];
        let mut request = FluxRequest::new_with_input_and_limits(media, limits);

        let mut options = HashMap::new();
        options.insert("text".to_owned(), text);

        request.operation("caption".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn flag(&self, media: Vec<u8>, user_id: u64) -> FluxResult {
        let tier = self.get_request_tier(user_id).await?;

        let limits = &LIMITS[tier];
        let mut request = FluxRequest::new_with_input_and_limits(media, limits);
        request.operation("flag".to_owned(), HashMap::new());
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn resize_absolute(&self, media: Vec<u8>, width: u32, height: u32, user_id: u64) -> FluxResult {
        let tier = self.get_request_tier(user_id).await?;

        let limits = &LIMITS[tier];
        let mut request = FluxRequest::new_with_input_and_limits(media, limits);

        let mut options = HashMap::new();
        options.insert("width".to_owned(), width.to_string());
        options.insert("height".to_owned(), height.to_string());

        request.operation("resize".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }

    pub async fn resize_scale(&self, media: Vec<u8>, scale: f32, user_id: u64) -> FluxResult {
        let tier = self.get_request_tier(user_id).await?;

        let limits = &LIMITS[tier];
        let mut request = FluxRequest::new_with_input_and_limits(media, limits);

        let mut options = HashMap::new();
        options.insert("scale".to_owned(), scale.to_string());

        request.operation("resize".to_owned(), options);
        request.output();

        self.run_flux(request, limits.time).await
    }
}
