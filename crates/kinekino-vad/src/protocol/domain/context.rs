use super::DecodedDescriptor;
use crate::domain::{ArtifactPin, VadInput, VadProfile};

/// Parent-held expected data, not an SDK identity or a settlement credential.
#[derive(Debug, Clone)]
pub struct ExpectedWorkerContext {
    request: ArtifactPin,
    descriptor: DecodedDescriptor,
}
impl ExpectedWorkerContext {
    pub fn new(request: ArtifactPin, descriptor: DecodedDescriptor) -> Self {
        Self {
            request,
            descriptor,
        }
    }
    pub fn request(&self) -> &ArtifactPin {
        &self.request
    }
    pub fn descriptor(&self) -> &DecodedDescriptor {
        &self.descriptor
    }
    pub fn profile(&self) -> &VadProfile {
        self.descriptor.profile()
    }
    pub fn input(&self) -> &VadInput {
        self.descriptor.input()
    }
    pub(crate) fn matches(&self, profile: &VadProfile, input: &VadInput) -> bool {
        let expected = self.input();
        self.profile() == profile
            && expected.wav() == input.wav()
            && expected.pcm() == input.pcm()
            && expected.samples() == input.samples()
            && expected.start() == input.start()
            && expected.end() == input.end()
    }
}
