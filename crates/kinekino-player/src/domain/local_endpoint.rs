use super::product::PRODUCT;
use kinekino_core::platform::ipc::EndpointId;

/// An explicit local host identity. It is not a URL or an OS path.
#[derive(Clone, Debug)]
pub struct LocalEndpoint {
    value: String,
    pub process_id: u32,
    pub process_start: u64,
}
impl LocalEndpoint {
    pub fn parse(value: &str) -> Result<Self, &'static str> {
        let prefix = format!("pipe:{}.v1.", PRODUCT.stable_product_id);
        let invalid = "Invalid local host endpoint";
        let suffix = value.strip_prefix(&prefix).ok_or(invalid)?;
        let parts: Vec<_> = suffix.split('.').collect();
        if parts.len() != 3 || value.len() > 160 {
            return Err(invalid);
        }
        let pid = parts[0];
        if pid.is_empty()
            || pid.len() > 10
            || !pid.bytes().all(|b| b.is_ascii_digit())
            || pid.starts_with('0')
        {
            return Err(invalid);
        }
        let process_id = pid.parse::<u32>().map_err(|_| invalid)?;
        if parts[1].len() != 16
            || parts[2].len() != 32
            || !parts[1..].iter().all(|p| {
                p.bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            })
        {
            return Err(invalid);
        }
        let process_start = u64::from_str_radix(parts[1], 16).map_err(|_| invalid)?;
        if process_start == 0 {
            return Err(invalid);
        }
        Ok(Self {
            value: value.into(),
            process_id,
            process_start,
        })
    }
    pub fn as_str(&self) -> &str {
        &self.value
    }
    /// OS-neutral port identity. The `pipe:` prefix is the stable public spelling (ADR 0007);
    /// the OS adapter alone decides the real address.
    pub fn id(&self) -> EndpointId {
        EndpointId::parse(
            PRODUCT.stable_product_id,
            self.value
                .strip_prefix("pipe:")
                .expect("validated endpoint"),
        )
        .expect("validated endpoint")
    }
    pub fn from_id(id: &EndpointId) -> Result<Self, &'static str> {
        Self::parse(&format!("pipe:{}", id.name()))
    }
}
