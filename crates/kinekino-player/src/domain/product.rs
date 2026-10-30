use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct ProductIdentity {
    pub stable_product_id: &'static str,
    pub display_name: &'static str,
    pub gui_executable: &'static str,
    pub cli_executable: &'static str,
    pub config_schema_version: &'static str,
}

pub const PRODUCT: ProductIdentity = ProductIdentity {
    stable_product_id: "com.kinekino.player",
    display_name: kinekino_core::product::identity::DISPLAY_NAME_EN,
    gui_executable: "player.exe",
    cli_executable: "kinekino.exe",
    config_schema_version: "1",
};
