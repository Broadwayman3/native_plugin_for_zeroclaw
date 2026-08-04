mod ar;
mod de;
mod en;
mod es;
mod fr;
mod hi;
mod it;
mod ja;
mod meta;
mod pl;
mod pt;
mod tr;
mod uk;
mod zh;

use once_cell::sync::Lazy;
use std::collections::{BTreeMap, HashMap};

pub static LANG_META: Lazy<BTreeMap<&'static str, (&'static str, &'static str)>> =
    Lazy::new(meta::build);

pub static TRANSLATIONS: Lazy<HashMap<&'static str, HashMap<&'static str, &'static str>>> =
    Lazy::new(|| {
        let mut all = HashMap::new();
        en::register(&mut all);
        uk::register(&mut all);
        pt::register(&mut all);
        es::register(&mut all);
        de::register(&mut all);
        fr::register(&mut all);
        it::register(&mut all);
        pl::register(&mut all);
        tr::register(&mut all);
        ja::register(&mut all);
        zh::register(&mut all);
        ar::register(&mut all);
        hi::register(&mut all);
        all
    });
