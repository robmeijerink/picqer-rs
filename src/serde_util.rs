use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use crate::tags::Tag;

pub(crate) fn opt_number_or_text<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<f64>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either {
        Number(f64),
        Text(String),
    }
    match Option::<Either>::deserialize(deserializer)? {
        None => Ok(None),
        Some(Either::Number(number)) => Ok(Some(number)),
        Some(Either::Text(text)) => text
            .trim()
            .parse()
            .map(Some)
            .map_err(|_| serde::de::Error::custom(format!("`{text}` is not a number"))),
    }
}

pub(crate) fn text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either {
        Text(String),
        Number(serde_json::Number),
    }
    Ok(match Either::deserialize(deserializer)? {
        Either::Text(text) => text,
        Either::Number(number) => number.to_string(),
    })
}

pub(crate) fn opt_text<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either {
        Text(String),
        Number(serde_json::Number),
        Flag(bool),
    }
    Ok(
        Option::<Either>::deserialize(deserializer)?.map(|value| match value {
            Either::Text(text) => text,
            Either::Number(number) => number.to_string(),
            Either::Flag(flag) => u8::from(flag).to_string(),
        }),
    )
}

pub(crate) fn tag_map<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<BTreeMap<String, Tag>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Either {
        Map(BTreeMap<String, Tag>),
        List(Vec<Tag>),
    }
    Ok(match Either::deserialize(deserializer)? {
        Either::Map(map) => map,
        Either::List(list) => list
            .into_iter()
            .map(|tag| (tag.title.clone(), tag))
            .collect(),
    })
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct Holder {
        #[serde(default, deserialize_with = "super::opt_number_or_text")]
        value: Option<f64>,
    }

    fn read(json: &str) -> Option<f64> {
        serde_json::from_str::<Holder>(json).unwrap().value
    }

    #[test]
    fn numbers_and_numeric_text_are_both_read() {
        assert_eq!(read(r#"{"value":0.5}"#), Some(0.5));
        assert_eq!(read(r#"{"value":"0.036"}"#), Some(0.036));
        assert_eq!(read(r#"{"value":null}"#), None);
        assert_eq!(read("{}"), None);
        assert!(serde_json::from_str::<Holder>(r#"{"value":"abc"}"#).is_err());
    }

    #[derive(Deserialize)]
    struct Texty {
        #[serde(default, deserialize_with = "super::opt_text")]
        value: Option<String>,
    }

    #[test]
    fn text_is_read_from_text_numbers_and_flags() {
        let read = |json: &str| serde_json::from_str::<Texty>(json).unwrap().value;
        assert_eq!(read(r#"{"value":"a"}"#).as_deref(), Some("a"));
        assert_eq!(read(r#"{"value":5}"#).as_deref(), Some("5"));
        assert_eq!(read(r#"{"value":true}"#).as_deref(), Some("1"));
        assert_eq!(read(r#"{"value":null}"#), None);
        assert_eq!(read("{}"), None);
    }

    #[derive(Deserialize)]
    struct Tagged {
        #[serde(default, deserialize_with = "super::tag_map")]
        tags: std::collections::BTreeMap<String, crate::tags::Tag>,
    }

    #[test]
    fn tags_are_read_as_an_object_or_as_a_list_and_empty_as_either() {
        let tag =
            r##"{"idtag":1,"title":"a","color":"#000000","inherit":false,"textColor":"#ffffff"}"##;
        let read = |json: String| serde_json::from_str::<Tagged>(&json).unwrap().tags;
        assert!(read(r#"{"tags":[]}"#.into()).is_empty());
        assert!(read(r#"{"tags":{}}"#.into()).is_empty());
        assert!(read("{}".into()).is_empty());
        assert_eq!(read(format!(r#"{{"tags":{{"a":{tag}}}}}"#))["a"].idtag.0, 1);
        assert_eq!(read(format!(r#"{{"tags":[{tag}]}}"#))["a"].idtag.0, 1);
    }
}
