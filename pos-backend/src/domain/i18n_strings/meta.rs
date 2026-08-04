use std::collections::BTreeMap;

pub fn build() -> BTreeMap<&'static str, (&'static str, &'static str)> {
    let mut m = BTreeMap::new();
    m.insert("uk", ("\u{1f1fa}\u{1f1e6}", "Українська"));
    m.insert("en", ("\u{1f1fa}\u{1f1f8}", "English"));
    m.insert("pt", ("\u{1f1e7}\u{1f1f7}", "Português"));
    m.insert("es", ("\u{1f1ea}\u{1f1f8}", "Español"));
    m.insert("de", ("\u{1f1e9}\u{1f1ea}", "Deutsch"));
    m.insert("fr", ("\u{1f1eb}\u{1f1f7}", "Français"));
    m.insert("it", ("\u{1f1ee}\u{1f1f9}", "Italiano"));
    m.insert("pl", ("\u{1f1f5}\u{1f1f1}", "Polski"));
    m.insert("tr", ("\u{1f1f9}\u{1f1f7}", "Türkçe"));
    m.insert("ja", ("\u{1f1ef}\u{1f1f5}", "日本語"));
    m.insert("zh", ("\u{1f1e8}\u{1f1f3}", "中文"));
    m.insert("ar", ("\u{1f1f8}\u{1f1e6}", "العربية"));
    m.insert("hi", ("\u{1f1ee}\u{1f1f3}", "हिन्दी"));
    m
}
