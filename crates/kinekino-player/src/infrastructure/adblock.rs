use adblock::{
    Engine,
    lists::{FilterSet, ParseOptions},
    request::Request,
};

pub const RULES: &str = include_str!("../../../../assets/filters.txt");

pub fn engine(extra: &str) -> Engine {
    let mut filters = FilterSet::default();
    filters.add_filter_list(format!("{RULES}\n{extra}"), ParseOptions::default());
    Engine::new_with_filter_set(filters)
}

pub fn blocked(engine: &Engine, url: &str, source: &str, kind: &str, method: &str) -> bool {
    Request::new(url, source, kind, method)
        .is_ok_and(|r| engine.check_network_request(&r).should_block())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocks_observed_ad_hosts_without_breaking_player() {
        let e = engine("");
        for u in [
            "https://googleads.g.doubleclick.net/pagead/ads",
            "https://pxl-eu.runative-syndicate.com/do2/click",
        ] {
            assert!(blocked(
                &e,
                u,
                "https://frame.example.test/",
                "script",
                "GET"
            ));
        }
        assert!(!blocked(
            &e,
            "https://media.example.test/video/123",
            "https://frame.example.test/",
            "subdocument",
            "GET"
        ));
        assert!(!blocked(
            &e,
            "https://cdn.example/video.m3u8",
            "https://media.example.test/",
            "xmlhttprequest",
            "GET"
        ));
    }

    #[test]
    fn supports_user_exception_rules() {
        let e = engine("@@||googleads.g.doubleclick.net^$script");
        assert!(!blocked(
            &e,
            "https://googleads.g.doubleclick.net/a.js",
            "https://catalog.example.test/",
            "script",
            "GET"
        ));
    }
}
