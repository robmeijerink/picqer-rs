//! Custom fields.

use crate::error::ValidationError;
use crate::reported::Reported;

string_enum! {
    /// The type of a custom field.
    ///
    /// Every type here was seen on real product fields. Another type, if Picqer adds one, is kept
    /// as [`Reported::Unknown`].
    pub enum FieldType {
        /// A single line of text.
        Text = "text",
        /// Several lines of text.
        Textarea = "textarea",
        /// One value out of a list, shown as radio buttons.
        Radio = "radio",
        /// A checkbox with a list of values.
        Checkbox = "checkbox",
        /// Several values out of a list, shown as checkboxes.
        Multicheckbox = "multicheckbox",
        /// One value out of a list, shown as a select box.
        Select = "select",
    }
}

pub(crate) const CHOICE_SEPARATOR: char = ';';

pub(crate) fn check_listed(
    what: &'static str,
    field_type: &Option<Reported<FieldType>>,
    values: &Option<Vec<String>>,
    value: &str,
) -> Result<(), ValidationError> {
    let listed = matches!(
        field_type,
        Some(Reported::Known(
            FieldType::Radio | FieldType::Select | FieldType::Checkbox
        ))
    );
    if listed
        && let Some(values) = values
        && !values.iter().any(|candidate| candidate == value)
    {
        return Err(ValidationError::new(
            what,
            format!("`{value}` is not one of {values:?}"),
        ));
    }
    Ok(())
}

pub(crate) fn join_choices<I, S>(
    what: &'static str,
    field_type: &Option<Reported<FieldType>>,
    values: &Option<Vec<String>>,
    choices: I,
) -> Result<Option<String>, ValidationError>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    if *field_type != Some(Reported::Known(FieldType::Multicheckbox)) {
        return Err(ValidationError::new(
            what,
            "the field is not a multicheckbox",
        ));
    }
    let allowed = values.as_deref().unwrap_or_default();
    let choices: Vec<S> = choices.into_iter().collect();
    let mut chosen: Vec<&str> = Vec::new();
    for choice in &choices {
        let choice = choice.as_ref();
        if choice.contains(CHOICE_SEPARATOR) {
            return Err(ValidationError::new(
                what,
                format!("`{choice}` contains `{CHOICE_SEPARATOR}`"),
            ));
        }
        if !allowed.iter().any(|value| value == choice) {
            return Err(ValidationError::new(
                what,
                format!("`{choice}` is not one of {allowed:?}"),
            ));
        }
        if !chosen.contains(&choice) {
            chosen.push(choice);
        }
    }
    Ok((!chosen.is_empty()).then(|| chosen.join(&CHOICE_SEPARATOR.to_string())))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kind(name: &str) -> Option<Reported<FieldType>> {
        Some(serde_json::from_value(serde_json::json!(name)).unwrap())
    }

    #[test]
    fn only_radio_select_and_checkbox_fields_take_a_listed_value() {
        let values = Some(vec!["a".to_owned(), "b".to_owned()]);
        assert!(check_listed("x", &kind("radio"), &values, "a").is_ok());
        assert!(check_listed("x", &kind("select"), &values, "c").is_err());
        assert!(check_listed("x", &kind("text"), &values, "c").is_ok());
        assert!(check_listed("x", &kind("radio"), &None, "c").is_ok());
    }

    #[test]
    fn a_checkbox_takes_one_listed_value() {
        let values = Some(vec!["a".to_owned(), "b".to_owned()]);
        assert!(check_listed("x", &kind("checkbox"), &values, "c").is_err());
        assert!(check_listed("x", &kind("checkbox"), &values, "b").is_ok());
        assert!(check_listed("x", &kind("multicheckbox"), &values, "a;b").is_ok());
    }

    #[test]
    fn choices_are_joined_with_a_semicolon_without_repeats() {
        let values = Some(vec!["a".to_owned(), "b".to_owned()]);
        let kind = kind("multicheckbox");
        assert_eq!(
            join_choices("x", &kind, &values, ["b", "a", "b"]).unwrap(),
            Some("b;a".to_owned())
        );
        assert_eq!(
            join_choices("x", &kind, &values, [] as [&str; 0]).unwrap(),
            None
        );
        assert!(join_choices("x", &kind, &values, ["c"]).is_err());
        assert!(join_choices("x", &super::tests::kind("text"), &values, ["a"]).is_err());
    }
}
