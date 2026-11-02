use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::prelude::FromRow;
use strum::Display;
use thiserror::Error;

type Dict = Value;

#[derive(Debug, Clone, Copy, Display, Serialize, Deserialize)]
#[strum(serialize_all = "snake_case")]
pub enum FieldType {
    Str,
    Json,
    Media,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: FieldType,
}

#[derive(Debug, FromRow)]
pub struct NoteType {
    pub id: i64,
    pub name: String,
    #[sqlx(json)]
    pub fields: Vec<Field>,
}

#[derive(Debug, FromRow)]
pub struct Note {
    pub id: i64,
    pub note_type: i64,
    pub content: Dict,
}

/// 创建笔记类型所需的数据。
#[derive(Debug, Clone)]
pub struct CreateNoteType {
    pub name: String,
    pub fields: Vec<Field>,
}

/// 创建笔记所需的数据。
#[derive(Debug, Clone)]
pub struct CreateNote {
    pub note_type: i64,
    pub content: Value,
}

#[derive(Debug, Error)]
pub enum FieldValidError {
    #[error("找不到匹配的笔记类型: {{id: {id}, name:{name}}}")]
    NoteTypeMiss { id: i64, name: String },

    #[error("笔记缺少 {count} 个字段: {missed_field_list}")]
    FieldsMiss {
        count: usize,
        missed_field_list: String,
    },

    #[error("笔记缺少字段: {name}")]
    FieldMiss { name: String },

    #[error("类型 {type_name} 不匹配")]
    TypeDismatch { type_name: String },
}

impl Note {
    pub fn validate(&self, note_type: NoteType) -> Result<bool> {
        if self.note_type != note_type.id {
            bail!(FieldValidError::NoteTypeMiss {
                id: note_type.id,
                name: note_type.name,
            });
        }

        let missing_fields: Vec<_> = note_type
            .fields
            .iter()
            .filter(|item| self.content.get(&item.name).is_none())
            .collect();

        if !missing_fields.is_empty() {
            let fields_list = missing_fields
                .iter()
                .map(|field| format!(" - {}", field.name))
                .collect::<Vec<_>>()
                .join("\n");
            bail!(FieldValidError::FieldsMiss {
                missed_field_list: fields_list,
                count: missing_fields.len()
            })
        }

        for note in note_type.fields.iter() {
            let note_field_type =
                self.content
                    .get(&note.name)
                    .ok_or_else(|| FieldValidError::FieldMiss {
                        name: note.name.clone(),
                    })?;

            let type_valid = match note.field_type {
                FieldType::Str => note_field_type.is_string(),
                FieldType::Json => note_field_type.is_object(),
                // TODO:由于media还没有设计好，所以以后再实现media的校验
                FieldType::Media => note_field_type.is_string(),
            };
            if !type_valid {
                bail!(FieldValidError::TypeDismatch {
                    type_name: note.field_type.to_string()
                });
            }
        }

        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn note_type() -> NoteType {
        NoteType {
            id: 1,
            name: "Basic".into(),
            fields: vec![
                Field {
                    name: "Front".into(),
                    field_type: FieldType::Str,
                },
                Field {
                    name: "Data".into(),
                    field_type: FieldType::Json,
                },
                Field {
                    name: "Audio".into(),
                    field_type: FieldType::Media,
                },
            ],
        }
    }

    fn note() -> Note {
        Note {
            id: 1,
            note_type: 1,
            content: json!({"Front": "hello", "Data": {}, "Audio": "hello.mp3"}),
        }
    }

    #[test]
    fn validate_accepts_matching_fields() -> Result<()> {
        assert!(note().validate(note_type())?);
        Ok(())
    }

    #[test]
    fn validate_rejects_wrong_note_type() {
        let mut note = note();
        note.note_type = 2;
        let error = note.validate(note_type()).unwrap_err();
        assert!(matches!(error.downcast_ref::<FieldValidError>(),
            Some(FieldValidError::NoteTypeMiss { id: 1, name }) if name == "Basic"));
    }

    #[test]
    fn validate_reports_all_missing_fields() {
        let mut note = note();
        note.content = json!({"Front": "hello"});
        let error = note.validate(note_type()).unwrap_err();
        match error.downcast_ref::<FieldValidError>() {
            Some(FieldValidError::FieldsMiss {
                count,
                missed_field_list,
            }) => {
                assert_eq!(*count, 2);
                assert_eq!(missed_field_list, " - Data\n - Audio");
            }
            other => panic!("expected FieldsMiss, got {other:?}"),
        }
    }

    #[test]
    fn validate_rejects_incorrect_field_types() {
        for (field, value, expected_type) in [
            ("Front", json!(42), "str"),
            ("Front", Value::Null, "str"),
            ("Data", json!([]), "json"),
            ("Data", json!("text"), "json"),
            ("Audio", json!(42), "media"),
        ] {
            let mut note = note();
            note.content[field] = value;
            let error = note.validate(note_type()).unwrap_err();
            assert!(
                matches!(error.downcast_ref::<FieldValidError>(),
                Some(FieldValidError::TypeDismatch { type_name }) if type_name == expected_type),
                "unexpected error for {field}: {error}"
            );
        }
    }
}
