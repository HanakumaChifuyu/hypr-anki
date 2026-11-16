use std::{collections::HashSet, sync::Arc};

use anyhow::{Result, ensure};
use hypranki_storage::{
    models::note::{CreateNoteType, Field, NoteType},
    repository::{note::NoteRepository, note_type::NoteTypeRepository},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, thiserror::Error)]
pub enum NoteTypeError {
    #[error("笔记类型 {0} 不存在")]
    TypeNotFound(i64),
    #[error("{0}")]
    InvalidInput(String),
    #[error("笔记类型 {0} 已被使用，不能修改字段结构")]
    TypeInUseForUpdate(i64),
    #[error("笔记类型 {0} 已被使用，不能删除")]
    TypeInUseForDelete(i64),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, utoipa::ToSchema)]
#[serde(deny_unknown_fields)]
pub struct UpdateNoteType {
    pub name: Option<String>,
    /// 提供时整体替换字段列表；已被使用的类型只允许修改顺序和可编辑属性。
    pub fields: Option<Vec<Field>>,
}

/// 管理笔记类型和字段定义。
#[derive(Debug, Clone)]
pub struct NoteTypeService {
    types: Arc<NoteTypeRepository>,
}

impl NoteTypeService {
    pub fn new(types: Arc<NoteTypeRepository>) -> Self {
        Self { types }
    }

    fn validate(name: &str, fields: &[Field]) -> Result<()> {
        ensure!(
            !name.trim().is_empty(),
            NoteTypeError::InvalidInput("笔记类型名称不能为空".into())
        );
        ensure!(
            !fields.is_empty(),
            NoteTypeError::InvalidInput("请至少添加一个字段".into())
        );
        let mut names = HashSet::new();
        for field in fields {
            ensure!(
                !field.name.trim().is_empty(),
                NoteTypeError::InvalidInput("字段名称不能为空".into())
            );
            ensure!(
                names.insert(&field.name),
                NoteTypeError::InvalidInput("字段名称不能重复".into())
            );
        }
        Ok(())
    }

    fn validate_id(id: i64) -> Result<()> {
        ensure!(
            id > 0,
            NoteTypeError::InvalidInput("请选择有效的笔记类型".into())
        );
        Ok(())
    }

    pub async fn create_type(&self, input: &CreateNoteType) -> Result<NoteType> {
        Self::validate(&input.name, &input.fields)?;
        let input = CreateNoteType {
            name: input.name.trim().to_owned(),
            fields: input.fields.clone(),
        };
        self.types.create(&input).await
    }

    pub async fn get_type(&self, id: i64) -> Result<NoteType> {
        Self::validate_id(id)?;
        self.types
            .find_by_id(id)
            .await?
            .ok_or_else(|| NoteTypeError::TypeNotFound(id).into())
    }

    pub async fn list_types(&self) -> Result<Vec<NoteType>> {
        self.types.find_all().await
    }

    pub async fn update_type(&self, id: i64, input: &UpdateNoteType) -> Result<NoteType> {
        Self::validate_id(id)?;
        // 与创建/修改卡片共用 SQLite 写锁，避免检查引用后出现并发写入。
        let mut tx = self.types.pool().begin_with("BEGIN IMMEDIATE").await?;
        let mut note_type = NoteTypeRepository::find_by_id_on(&mut tx, id)
            .await?
            .ok_or(NoteTypeError::TypeNotFound(id))?;
        let original = note_type.clone();
        if let Some(name) = &input.name {
            note_type.name = name.trim().to_owned();
        }
        if let Some(fields) = &input.fields {
            note_type.fields = fields.clone();
        }
        Self::validate(&note_type.name, &note_type.fields)?;
        let same_structure = original.fields.len() == note_type.fields.len()
            && original.fields.iter().all(|old| {
                note_type
                    .fields
                    .iter()
                    .any(|new| old.name == new.name && old.field_type == new.field_type)
            });
        if !same_structure {
            ensure!(
                !NoteRepository::exists_by_note_type_on(&mut tx, id).await?,
                NoteTypeError::TypeInUseForUpdate(id)
            );
        }
        NoteTypeRepository::update_on(&mut tx, &note_type).await?;
        tx.commit().await?;
        Ok(note_type)
    }

    pub async fn delete_type(&self, id: i64) -> Result<()> {
        Self::validate_id(id)?;
        let mut tx = self.types.pool().begin_with("BEGIN IMMEDIATE").await?;
        NoteTypeRepository::find_by_id_on(&mut tx, id)
            .await?
            .ok_or(NoteTypeError::TypeNotFound(id))?;
        ensure!(
            !NoteRepository::exists_by_note_type_on(&mut tx, id).await?,
            NoteTypeError::TypeInUseForDelete(id)
        );
        NoteTypeRepository::delete_by_id_on(&mut tx, id).await?;
        tx.commit().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hypranki_storage::{
        db::pool::SqlitePool,
        models::note::{CreateNote, FieldType},
    };
    use serde_json::json;

    async fn setup() -> Result<(NoteTypeService, NoteRepository, tempfile::TempDir)> {
        let dir = tempfile::tempdir()?;
        let pool = SqlitePool::new(dir.path().join("notes.db")).await?;
        pool.initialize_database().await?;
        Ok((
            NoteTypeService::new(Arc::new(NoteTypeRepository::new(pool.clone()))),
            NoteRepository::new(pool),
            dir,
        ))
    }

    fn input() -> CreateNoteType {
        CreateNoteType {
            name: " Basic ".into(),
            fields: vec![
                Field {
                    name: "Front".into(),
                    field_type: FieldType::Str,
                    writability: true,
                },
                Field {
                    name: "Data".into(),
                    field_type: FieldType::Json,
                    writability: false,
                },
            ],
        }
    }

    #[tokio::test]
    async fn validates_definitions_without_writes() -> Result<()> {
        let (service, _, _dir) = setup().await?;
        let mut invalid = input();
        invalid.name = " \n\t".into();
        assert!(matches!(
            service
                .create_type(&invalid)
                .await
                .unwrap_err()
                .downcast_ref::<NoteTypeError>(),
            Some(NoteTypeError::InvalidInput(_))
        ));
        invalid = input();
        invalid.fields.clear();
        assert!(service.create_type(&invalid).await.is_err());
        invalid = input();
        invalid.fields[0].name = " ".into();
        assert!(service.create_type(&invalid).await.is_err());
        invalid = input();
        invalid.fields[1].name = "Front".into();
        assert!(service.create_type(&invalid).await.is_err());
        assert!(service.list_types().await?.is_empty());
        let created = service.create_type(&input()).await?;
        let error = service
            .update_type(
                created.id,
                &UpdateNoteType {
                    fields: Some(invalid.fields),
                    ..Default::default()
                },
            )
            .await
            .unwrap_err();
        assert!(matches!(
            error.downcast_ref::<NoteTypeError>(),
            Some(NoteTypeError::InvalidInput(_))
        ));
        assert_eq!(service.get_type(created.id).await?.fields[1].name, "Data");
        for id in [0, -1] {
            assert!(service.get_type(id).await.is_err());
            assert!(
                service
                    .update_type(id, &UpdateNoteType::default())
                    .await
                    .is_err()
            );
            assert!(service.delete_type(id).await.is_err());
        }
        for error in [
            service.get_type(987654).await.unwrap_err(),
            service
                .update_type(987654, &UpdateNoteType::default())
                .await
                .unwrap_err(),
            service.delete_type(987654).await.unwrap_err(),
        ] {
            assert!(matches!(
                error.downcast_ref::<NoteTypeError>(),
                Some(NoteTypeError::TypeNotFound(987654))
            ));
        }
        Ok(())
    }

    #[tokio::test]
    async fn type_lifecycle_protects_referenced_content_and_rolls_back() -> Result<()> {
        let (service, notes, _dir) = setup().await?;
        let created = service.create_type(&input()).await?;
        assert_eq!(created.name, "Basic");
        let second = service.create_type(&input()).await?;
        assert_eq!(
            service
                .list_types()
                .await?
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>(),
            vec![created.id, second.id]
        );
        // 未使用的类型可以自由修改字段结构。
        let mut fields = created.fields.clone();
        fields[0].name = "Question".into();
        service
            .update_type(
                second.id,
                &UpdateNoteType {
                    name: None,
                    fields: Some(fields),
                },
            )
            .await?;
        service.delete_type(second.id).await?;

        let note = notes
            .create(&CreateNote {
                note_type: created.id,
                content: json!({"Front":"hello","Data":{}}),
            })
            .await?;
        for fields in [
            vec![created.fields[0].clone()],
            {
                let mut f = created.fields.clone();
                f.push(Field {
                    name: "Extra".into(),
                    field_type: FieldType::Str,
                    writability: true,
                });
                f
            },
            {
                let mut f = created.fields.clone();
                f[0].name = "Question".into();
                f
            },
            {
                let mut f = created.fields.clone();
                f[0].field_type = FieldType::Json;
                f
            },
        ] {
            let error = service
                .update_type(
                    created.id,
                    &UpdateNoteType {
                        name: Some("Should roll back".into()),
                        fields: Some(fields),
                    },
                )
                .await
                .unwrap_err();
            assert!(matches!(
                error.downcast_ref::<NoteTypeError>(),
                Some(NoteTypeError::TypeInUseForUpdate(_))
            ));
            assert_eq!(service.get_type(created.id).await?.name, "Basic");
            assert_eq!(
                notes.find_by_id(note.id).await?.unwrap().content,
                note.content
            );
        }
        // 字段顺序与可编辑属性不改变已有内容的结构。
        let mut fields = created.fields.clone();
        fields.reverse();
        fields[1].writability = false;
        let updated = service
            .update_type(
                created.id,
                &UpdateNoteType {
                    name: Some(" Renamed ".into()),
                    fields: Some(fields),
                },
            )
            .await?;
        assert_eq!(updated.name, "Renamed");
        assert_eq!(updated.fields[0].name, "Data");
        assert!(!updated.fields[1].writability);
        assert!(
            notes
                .find_by_id(note.id)
                .await?
                .unwrap()
                .validate(updated)?
        );
        let error = service.delete_type(created.id).await.unwrap_err();
        assert!(matches!(
            error.downcast_ref::<NoteTypeError>(),
            Some(NoteTypeError::TypeInUseForDelete(_))
        ));
        assert!(notes.find_by_id(note.id).await?.is_some());
        notes.delete_by_id(note.id).await?;
        service.delete_type(created.id).await?;
        assert!(service.list_types().await?.is_empty());
        Ok(())
    }
}
