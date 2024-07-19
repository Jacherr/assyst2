use std::collections::HashMap;

use super::limits::LimitData;

/// A step in a Flux execution.
pub enum FluxStep {
    /// Input file. Saves the file and passes to Flux as `-i path`. Input must be the first step.
    Input(Vec<u8>),
    /// Operation. Passes to Flux as `-o operation[k=v]`
    Operation((String, HashMap<String, String>)),
    /// Output. Passes to Flux as `path` at the end. Output must be the last step.
    Output,
    /// Frame limit of inputs. Inputs will have additional frames removed.
    ImagePageLimit(u64),
    /// Resolution limit of input. Input will shrink, preserving aspect ratio, to fit this.
    ResolutionLimit((u64, u64)),
    /// Whether to disable video decoding.
    VideoDecodeDisabled,
    /// Get media info
    Info,
    /// Get version info
    Version,
}

pub struct FluxRequest(pub Vec<FluxStep>);
impl FluxRequest {
    pub fn new() -> Self {
        Self(vec![])
    }

    pub fn new_with_input_and_limits(input: Vec<u8>, limits: &LimitData) -> Self {
        let mut new = Self(vec![]);
        new.input(input);
        new.limits(limits);
        new
    }

    pub fn new_basic(input: Vec<u8>, limits: &LimitData, operation: &str) -> Self {
        let mut new = Self(vec![]);
        new.input(input);
        new.limits(limits);
        new.operation(operation.to_owned(), HashMap::new());
        new.output();
        new
    }

    pub fn input(&mut self, input: Vec<u8>) {
        self.0.push(FluxStep::Input(input));
    }

    pub fn operation(&mut self, name: String, options: HashMap<String, String>) {
        self.0.push(FluxStep::Operation((name, options)));
    }

    pub fn output(&mut self) {
        self.0.push(FluxStep::Output);
    }

    pub fn limits(&mut self, limits: &LimitData) {
        self.0.push(FluxStep::ImagePageLimit(limits.frames));
        self.0.push(FluxStep::ResolutionLimit((limits.size, limits.size)));
        if !limits.video_decode_enabled {
            self.0.push(FluxStep::VideoDecodeDisabled)
        }
    }

    pub fn info(&mut self) {
        self.0.push(FluxStep::Info);
    }

    pub fn version(&mut self) {
        self.0.push(FluxStep::Version);
    }
}
