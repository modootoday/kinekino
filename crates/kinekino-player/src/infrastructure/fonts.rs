pub fn korean_font() -> Option<Vec<u8>> {
    std::fs::read(r"C:\Windows\Fonts\malgun.ttf").ok()
}
