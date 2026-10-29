use super::{MAX_MODULES, payload};
use crate::domain::ArtifactPin;
use crate::protocol::domain::ModuleObservation;
use kinekino_media_contracts::api_values::DecimalU64;
use serde::{
    Deserialize, Deserializer,
    de::{self, SeqAccess, Visitor},
};
use std::fmt;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ModuleDto {
    name: String,
    sha256: String,
    bytes: DecimalU64,
    canonical_path: Option<String>,
}
#[derive(Debug, Default)]
pub(super) struct Modules(pub Vec<ModuleObservation>);
impl<'de> Deserialize<'de> for Modules {
    fn deserialize<D: Deserializer<'de>>(decoder: D) -> Result<Self, D::Error> {
        struct ModuleVisitor;
        impl<'de> Visitor<'de> for ModuleVisitor {
            type Value = Modules;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("bounded distinct runtime modules")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut sequence: A) -> Result<Modules, A::Error> {
                let mut modules: Vec<ModuleObservation> = Vec::new();
                modules
                    .try_reserve_exact(MAX_MODULES)
                    .map_err(|_| de::Error::custom("Module allocation failed"))?;
                loop {
                    if modules.len() == MAX_MODULES {
                        if sequence.next_element::<de::IgnoredAny>()?.is_some() {
                            return Err(de::Error::custom("Too many modules"));
                        }
                        break;
                    }
                    let Some(item) = sequence.next_element::<ModuleDto>()? else {
                        break;
                    };
                    if !payload::optional_text(&item.canonical_path, 4096)
                        || modules
                            .iter()
                            .any(|m| m.pin().id().eq_ignore_ascii_case(&item.name))
                    {
                        return Err(de::Error::custom("Invalid/duplicate/oversized modules"));
                    }
                    let pin = ArtifactPin::new(item.name, item.sha256, item.bytes)
                        .map_err(|_| de::Error::custom("Invalid module pin"))?;
                    modules.push(ModuleObservation {
                        pin,
                        canonical_path: item.canonical_path,
                    });
                }
                Ok(Modules(modules))
            }
        }
        decoder.deserialize_seq(ModuleVisitor)
    }
}
