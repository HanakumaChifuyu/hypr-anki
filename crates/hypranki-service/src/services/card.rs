use anyhow::{Result, anyhow, ensure};
use fsrs::{FSRS, MemoryState};
use hypranki_storage::{
    models::{
        card::{Card, CreateCard, StoredMemoryState},
        note::{Field, Note, NoteType},
        template::Template,
    },
    repository::{
        card::{CardRepository, CardReview},
        deck::DeckRepository,
        note::NoteRepository,
        note_type::NoteTypeRepository,
        template::TemplateRepository,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::SqliteConnection;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Debug, thiserror::Error)]
pub enum CardError {
    #[error("卡片 {0} 不存在")]
    NotFound(i64),
    #[error("{0}")]
    InvalidInput(String),
    #[error("卡片记忆状态和复习时间不一致")]
    InvalidMemoryState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCardInput {
    pub note_type_id: i64,
    pub content: Value,
    pub template_id: i64,
    pub deck_id: i64,
}

/// 面向卡片详情页的数据，不暴露内部内容记录或算法状态。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardDetails {
    pub id: i64,
    pub content: Value,
    /// 动态表单的字段名称、类型及是否允许编辑。
    pub fields: Vec<Field>,
    pub card_type_name: String,
    pub deck_id: i64,
    pub deck_name: String,
    pub template_id: i64,
    pub template_name: String,
    /// 时间均为 Unix 秒；未复习或尚未安排时为 None。
    pub last_reviewed_at: Option<i64>,
    pub due_at: Option<i64>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewAnswer {
    Forgot,
    Remembered,
}

impl ReviewAnswer {
    fn rating(self) -> i64 {
        match self {
            Self::Forgot => 1,
            Self::Remembered => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SchedulingOptions {
    pub desired_retention: f32,
    pub max_interval_days: u32,
}
impl Default for SchedulingOptions {
    fn default() -> Self {
        Self {
            desired_retention: 0.9,
            max_interval_days: 36500,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReviewResult {
    pub card_id: i64,
    pub memory_state: StoredMemoryState,
    pub reviewed_at: i64,
    pub due_at: i64,
}

/// 统一的卡片业务入口。所有 repo 共用 cards 的连接池，以便跨表事务一致。
#[derive(Debug, Clone)]
pub struct CardService {
    cards: Arc<CardRepository>,
    scheduler: SchedulingOptions,
}

fn now() -> Result<i64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs()
        .try_into()?)
}
fn invalid(message: impl Into<String>) -> anyhow::Error {
    CardError::InvalidInput(message.into()).into()
}

impl CardService {
    pub fn new(cards: Arc<CardRepository>) -> Self {
        Self {
            cards,
            scheduler: SchedulingOptions::default(),
        }
    }

    pub fn with_scheduling(
        cards: Arc<CardRepository>,
        scheduler: SchedulingOptions,
    ) -> Result<Self> {
        ensure!(
            scheduler.desired_retention.is_finite()
                && scheduler.desired_retention > 0.0
                && scheduler.desired_retention < 1.0,
            CardError::InvalidInput("目标保持率必须在 0 和 1 之间".into())
        );
        ensure!(
            scheduler.max_interval_days > 0,
            CardError::InvalidInput("最大复习间隔必须大于零".into())
        );
        Ok(Self { cards, scheduler })
    }

    async fn card_on(conn: &mut SqliteConnection, id: i64) -> Result<Card> {
        CardRepository::find_by_id_on(conn, id)
            .await?
            .ok_or_else(|| CardError::NotFound(id).into())
    }
    async fn note_on(conn: &mut SqliteConnection, id: i64) -> Result<Note> {
        NoteRepository::find_by_id_on(conn, id)
            .await?
            .ok_or_else(|| anyhow!("卡片引用的内容 {id} 不存在"))
    }
    async fn type_on(conn: &mut SqliteConnection, id: i64) -> Result<NoteType> {
        NoteTypeRepository::find_by_id_on(conn, id)
            .await?
            .ok_or_else(|| invalid(format!("字段类型 {id} 不存在")))
    }
    async fn deck_on(conn: &mut SqliteConnection, id: i64) -> Result<()> {
        let deck = DeckRepository::find_by_id_on(conn, id)
            .await?
            .ok_or_else(|| invalid(format!("牌组 {id} 不存在")))?;
        ensure!(
            !deck.folder,
            CardError::InvalidInput("文件夹不能直接存放卡片".into())
        );
        Ok(())
    }
    async fn template_on(conn: &mut SqliteConnection, id: i64) -> Result<Template> {
        TemplateRepository::find_by_id_on(conn, id)
            .await?
            .ok_or_else(|| invalid(format!("模板 {id} 不存在")))
    }
    fn validate(note: &Note, note_type: NoteType) -> Result<()> {
        note.validate(note_type)
            .map_err(|error| invalid(error.to_string()))?;
        Ok(())
    }
    async fn create_on(conn: &mut SqliteConnection, input: &CreateCardInput) -> Result<Card> {
        Self::deck_on(conn, input.deck_id).await?;
        Self::template_on(conn, input.template_id).await?;
        let note_type = Self::type_on(conn, input.note_type_id).await?;
        let note = Note {
            id: 0,
            note_type: input.note_type_id,
            content: input.content.clone(),
        };
        Self::validate(&note, note_type)?;
        let saved = NoteRepository::create_on(
            conn,
            &hypranki_storage::models::note::CreateNote {
                note_type: note.note_type,
                content: note.content,
            },
        )
        .await?;
        CardRepository::create_on(
            conn,
            &CreateCard {
                note_id: saved.id,
                template_id: input.template_id,
                deck_id: input.deck_id,
            },
        )
        .await
    }

    pub async fn create(&self, input: &CreateCardInput) -> Result<Card> {
        let mut tx = self.cards.pool().begin_with("BEGIN IMMEDIATE").await?;
        let card = Self::create_on(&mut tx, input).await?;
        tx.commit().await?;
        Ok(card)
    }
    pub async fn get(&self, id: i64) -> Result<CardDetails> {
        let mut tx = self.cards.pool().begin().await?;
        let card = Self::card_on(&mut tx, id).await?;
        let note = Self::note_on(&mut tx, card.note_id).await?;
        let note_type = Self::type_on(&mut tx, note.note_type).await?;
        let template = Self::template_on(&mut tx, card.template_id).await?;
        let deck = DeckRepository::find_by_id_on(&mut tx, card.deck_id)
            .await?
            .ok_or_else(|| anyhow!("卡片引用的牌组 {} 不存在", card.deck_id))?;
        tx.commit().await?;
        Ok(CardDetails {
            id: card.id,
            content: note.content,
            fields: note_type.fields,
            card_type_name: note_type.name,
            deck_id: deck.id,
            deck_name: deck.name,
            template_id: template.id,
            template_name: template.name,
            last_reviewed_at: card.last_reviewed_at,
            due_at: card.due_at,
            created_at: card.created_at,
            updated_at: card.updated_at,
        })
    }
    pub async fn list(&self, deck_id: Option<i64>) -> Result<Vec<Card>> {
        match deck_id {
            Some(id) => self.cards.find_by_deck_id(id).await,
            None => self.cards.find_all().await,
        }
    }
    pub async fn list_due(&self, limit: u32) -> Result<Vec<Card>> {
        self.cards.find_due(now()?, limit).await
    }

    /// 完整替换字段内容；只读字段不能改变。是否重置记忆由调用者明确指定。
    pub async fn update_content(
        &self,
        id: i64,
        content: Value,
        reset_memory: bool,
    ) -> Result<Card> {
        let mut tx = self.cards.pool().begin_with("BEGIN IMMEDIATE").await?;
        let card = Self::card_on(&mut tx, id).await?;
        let mut note = Self::note_on(&mut tx, card.note_id).await?;
        let note_type = Self::type_on(&mut tx, note.note_type).await?;
        for field in &note_type.fields {
            ensure!(
                field.writability || content.get(&field.name) == note.content.get(&field.name),
                CardError::InvalidInput(format!("字段 {} 是只读的", field.name))
            );
        }
        note.content = content;
        Self::validate(&note, note_type)?;
        NoteRepository::update_on(&mut tx, &note).await?;
        // 更新时间但保留当前记忆状态和调度字段。
        CardRepository::update_on(&mut tx, &card).await?;
        if reset_memory {
            CardRepository::reset_memory_on(&mut tx, id).await?;
        }
        let result = Self::card_on(&mut tx, id).await?;
        tx.commit().await?;
        Ok(result)
    }
    pub async fn move_to(&self, id: i64, deck_id: i64) -> Result<Card> {
        let mut tx = self.cards.pool().begin_with("BEGIN IMMEDIATE").await?;
        Self::card_on(&mut tx, id).await?;
        Self::deck_on(&mut tx, deck_id).await?;
        CardRepository::update_deck_on(&mut tx, id, deck_id).await?;
        let card = Self::card_on(&mut tx, id).await?;
        tx.commit().await?;
        Ok(card)
    }
    /// 模板目前没有字段约束，现阶段只验证存在性。
    pub async fn change_template(&self, id: i64, template_id: i64) -> Result<Card> {
        let mut tx = self.cards.pool().begin_with("BEGIN IMMEDIATE").await?;
        Self::card_on(&mut tx, id).await?;
        Self::template_on(&mut tx, template_id).await?;
        CardRepository::update_template_on(&mut tx, id, template_id).await?;
        let card = Self::card_on(&mut tx, id).await?;
        tx.commit().await?;
        Ok(card)
    }
    /// 复制内容而不是共享 Note，副本不继承学习状态。
    pub async fn duplicate(&self, id: i64, deck_id: i64, template_id: i64) -> Result<Card> {
        let mut tx = self.cards.pool().begin_with("BEGIN IMMEDIATE").await?;
        let original = Self::card_on(&mut tx, id).await?;
        let note = Self::note_on(&mut tx, original.note_id).await?;
        let card = Self::create_on(
            &mut tx,
            &CreateCardInput {
                note_type_id: note.note_type,
                content: note.content,
                deck_id,
                template_id,
            },
        )
        .await?;
        tx.commit().await?;
        Ok(card)
    }
    pub async fn delete(&self, id: i64) -> Result<()> {
        let mut tx = self.cards.pool().begin_with("BEGIN IMMEDIATE").await?;
        let card = Self::card_on(&mut tx, id).await?;
        CardRepository::delete_by_id_on(&mut tx, id).await?;
        NoteRepository::delete_by_id_on(&mut tx, card.note_id).await?;
        tx.commit().await?;
        Ok(())
    }
    /// 清空当前状态和到期时间，历史记录保留。
    pub async fn reset_memory(&self, id: i64) -> Result<Card> {
        let mut tx = self.cards.pool().begin_with("BEGIN IMMEDIATE").await?;
        Self::card_on(&mut tx, id).await?;
        CardRepository::reset_memory_on(&mut tx, id).await?;
        let card = Self::card_on(&mut tx, id).await?;
        tx.commit().await?;
        Ok(card)
    }

    fn transition(
        &self,
        card: &Card,
        answer: ReviewAnswer,
        reviewed_at: i64,
    ) -> Result<CardReview> {
        ensure!(
            card.memory_state.is_some() == card.last_reviewed_at.is_some(),
            CardError::InvalidMemoryState
        );
        let elapsed = match card.last_reviewed_at {
            Some(last) => reviewed_at
                .checked_sub(last)
                .filter(|value| *value >= 0)
                .ok_or_else(|| invalid("复习时间早于上次复习时间"))?,
            None => 0,
        };
        if let Some(state) = card.memory_state {
            ensure!(
                state.stability.is_finite()
                    && state.stability > 0.0
                    && state.stability_fast.is_finite()
                    && state.stability_fast > 0.0
                    && state.difficulty.is_finite()
                    && (1.0..=10.0).contains(&state.difficulty),
                CardError::InvalidMemoryState
            );
        }
        let fsrs = FSRS::default();
        let previous = card.memory_state.map(|state| MemoryState {
            stability: state.stability,
            stability_fast: state.stability_fast,
            difficulty: state.difficulty,
        });
        let states = fsrs.next_states_with_elapsed_days(
            previous,
            self.scheduler.desired_retention,
            elapsed as f32 / 86400.0,
        )?;
        let next = match answer {
            ReviewAnswer::Forgot => states.again,
            ReviewAnswer::Remembered => states.good,
        };
        ensure!(
            next.interval.is_finite() && next.interval >= 0.0,
            CardError::InvalidMemoryState
        );
        let seconds = ((next.interval as f64).min(self.scheduler.max_interval_days as f64)
            * 86400.0)
            .ceil()
            .max(1.0) as i64;
        Ok(CardReview {
            id: 0,
            card_id: card.id,
            reviewed_at,
            rating: answer.rating(),
            elapsed_seconds: elapsed,
            scheduled_seconds: seconds,
            memory_before: card.memory_state,
            memory_after: StoredMemoryState {
                stability: next.memory.stability,
                stability_fast: next.memory.stability_fast,
                difficulty: next.memory.difficulty,
            },
            algorithm: "fsrs-7".into(),
            parameters: fsrs::DEFAULT_PARAMETERS.to_vec(),
            desired_retention: self.scheduler.desired_retention,
        })
    }
    pub async fn review(&self, id: i64, answer: ReviewAnswer) -> Result<ReviewResult> {
        let mut tx = self.cards.pool().begin_with("BEGIN IMMEDIATE").await?;
        let card = Self::card_on(&mut tx, id).await?;
        let review = self.transition(&card, answer, now()?)?;
        let updated = CardRepository::save_review_on(&mut tx, &review).await?;
        tx.commit().await?;
        Ok(ReviewResult {
            card_id: id,
            memory_state: review.memory_after,
            reviewed_at: review.reviewed_at,
            due_at: updated.due_at.unwrap(),
        })
    }
    /// 只预览，不写入；提交时始终重新读取、计算。
    pub async fn preview_review(&self, id: i64, answer: ReviewAnswer) -> Result<ReviewResult> {
        let card = self
            .cards
            .find_by_id(id)
            .await?
            .ok_or(CardError::NotFound(id))?;
        let review = self.transition(&card, answer, now()?)?;
        Ok(ReviewResult {
            card_id: id,
            memory_state: review.memory_after,
            reviewed_at: review.reviewed_at,
            due_at: review
                .reviewed_at
                .checked_add(review.scheduled_seconds)
                .ok_or_else(|| invalid("复习时间溢出"))?,
        })
    }
    pub async fn review_history(&self, id: i64) -> Result<Vec<CardReview>> {
        self.cards
            .find_by_id(id)
            .await?
            .ok_or(CardError::NotFound(id))?;
        self.cards.review_history(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hypranki_storage::db::pool::SqlitePool;
    use serde_json::json;

    async fn setup() -> Result<(CardService, tempfile::TempDir)> {
        let dir = tempfile::tempdir()?;
        let pool = SqlitePool::new(dir.path().join("service.db")).await?;
        pool.initialize_database().await?;
        sqlx::raw_sql(
            r#"INSERT INTO DECKS (id, name, folder, config_id, created_at, updated_at)
                 VALUES (1, 'A', 0, 1, 0, 0), (2, 'B', 0, 1, 0, 0), (3, 'Folder', 1, 1, 0, 0);
               INSERT INTO NOTETYPES (id, name, fields) VALUES
                 (1, 'Basic', '[{"name":"Front","field_type":"Str","writability":true},{"name":"Source","field_type":"Str","writability":false}]');
               INSERT INTO TEMPLATES (id, name) VALUES (1, 'A'), (2, 'B');"#,
        ).execute(pool.inner()).await?;
        Ok((CardService::new(Arc::new(CardRepository::new(pool))), dir))
    }
    fn input() -> CreateCardInput {
        CreateCardInput {
            note_type_id: 1,
            content: json!({"Front":"hello", "Source":"book"}),
            template_id: 1,
            deck_id: 1,
        }
    }

    #[tokio::test]
    async fn lifecycle_keeps_independent_content_and_cleans_up() -> Result<()> {
        let (service, _dir) = setup().await?;
        let a = service.create(&input()).await?;
        assert!(a.memory_state.is_none());
        let details = service.get(a.id).await?;
        assert_eq!(details.content, input().content);
        assert_eq!(details.card_type_name, "Basic");
        assert_eq!(details.deck_name, "A");
        assert_eq!(details.template_name, "A");
        assert_eq!(details.fields.len(), 2);
        let json = serde_json::to_value(&details)?;
        for internal in ["note", "note_id", "note_type", "card", "memory_state"] {
            assert!(json.get(internal).is_none());
        }
        assert_eq!(
            serde_json::from_value::<CardDetails>(json)?.content,
            details.content
        );
        assert_eq!(details.template_id, 1);
        service.review(a.id, ReviewAnswer::Remembered).await?;
        let copy = service.duplicate(a.id, 2, 2).await?;
        assert_ne!(a.note_id, copy.note_id);
        assert!(
            copy.memory_state.is_none() && copy.last_reviewed_at.is_none() && copy.due_at.is_none()
        );
        service
            .update_content(copy.id, json!({"Front":"changed", "Source":"book"}), false)
            .await?;
        assert_eq!(service.get(a.id).await?.content["Front"], "hello");
        service.move_to(copy.id, 1).await?;
        service.change_template(copy.id, 1).await?;
        assert_eq!(service.list(Some(1)).await?.len(), 2);
        assert_eq!(service.list(None).await?.len(), 2);
        let updated = service.update_content(a.id, input().content, false).await?;
        assert!(updated.memory_state.is_some());
        let reset = service.update_content(a.id, input().content, true).await?;
        assert!(
            reset.memory_state.is_none()
                && reset.last_reviewed_at.is_none()
                && reset.due_at.is_none()
        );
        assert_eq!(service.review_history(a.id).await?.len(), 1);
        service.delete(a.id).await?;
        assert!(
            service
                .get(a.id)
                .await
                .unwrap_err()
                .downcast_ref::<CardError>()
                .is_some()
        );
        assert!(service.cards.review_history(a.id).await?.is_empty());
        assert!(
            NoteRepository::new(SqlitePool {
                inner_pool: service.cards.pool().clone()
            })
            .find_by_id(a.note_id)
            .await?
            .is_none()
        );
        assert_eq!(service.get(copy.id).await?.content["Front"], "changed");
        Ok(())
    }

    #[tokio::test]
    async fn validations_and_transaction_rollback() -> Result<()> {
        let (service, _dir) = setup().await?;
        for bad in [
            CreateCardInput {
                deck_id: 3,
                ..input()
            },
            CreateCardInput {
                deck_id: 99,
                ..input()
            },
            CreateCardInput {
                template_id: 99,
                ..input()
            },
            CreateCardInput {
                note_type_id: 99,
                ..input()
            },
            CreateCardInput {
                content: json!({"Front":12,"Source":"book"}),
                ..input()
            },
        ] {
            assert!(service.create(&bad).await.is_err());
        }
        assert_eq!(service.cards.count().await?, 0);
        let a = service.create(&input()).await?;
        assert!(
            service
                .update_content(a.id, json!({"Front":"a","Source":"altered"}), false)
                .await
                .is_err()
        );
        assert!(
            service
                .update_content(a.id, json!({"Front":"a","Source":"book","Extra":1}), false)
                .await
                .is_err()
        );
        assert!(service.move_to(a.id, 3).await.is_err());
        assert!(service.change_template(a.id, 99).await.is_err());
        assert!(service.duplicate(a.id, 1, 99).await.is_err());
        assert!(service.delete(999).await.is_err());
        assert!(service.review(999, ReviewAnswer::Forgot).await.is_err());
        assert_eq!(service.get(a.id).await?.content, input().content);
        // 人为令第二步插入失败，验证第一步创建的 Note 回滚。
        sqlx::raw_sql("CREATE TRIGGER fail_card BEFORE INSERT ON CARDS BEGIN SELECT RAISE(ABORT, 'test failure'); END;")
            .execute(service.cards.pool()).await?;
        assert!(service.create(&input()).await.is_err());
        let notes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM NOTES")
            .fetch_one(service.cards.pool())
            .await?;
        assert_eq!(notes, 1);
        assert_eq!(service.cards.count().await?, 1);
        Ok(())
    }

    #[tokio::test]
    async fn review_uses_fsrs_and_preview_does_not_write() -> Result<()> {
        let (service, _dir) = setup().await?;
        let card = service.create(&input()).await?;
        let preview = service
            .preview_review(card.id, ReviewAnswer::Remembered)
            .await?;
        assert!(
            service
                .cards
                .find_by_id(card.id)
                .await?
                .unwrap()
                .memory_state
                .is_none()
        );
        assert!(service.review_history(card.id).await?.is_empty());
        let first = service.review(card.id, ReviewAnswer::Remembered).await?;
        assert_eq!(first.memory_state, preview.memory_state);
        let first_history = service.review_history(card.id).await?;
        assert_eq!(first_history[0].rating, 3);
        assert_eq!(first_history[0].parameters.len(), 34);
        assert_eq!(first_history[0].elapsed_seconds, 0);
        assert!(first_history[0].memory_before.is_none());
        // 模拟 5 分钟后的日内复习，确保没有截断成整数天。
        sqlx::query("UPDATE CARDS SET last_reviewed_at = ? WHERE id = ?")
            .bind(first.reviewed_at - 300)
            .bind(card.id)
            .execute(service.cards.pool())
            .await?;
        let result = service.review(card.id, ReviewAnswer::Forgot).await?;
        let history = service.review_history(card.id).await?;
        assert_eq!(history.len(), 2);
        let log = &history[1];
        let expected = FSRS::default()
            .next_states_with_elapsed_days(
                Some(MemoryState {
                    stability: first.memory_state.stability,
                    stability_fast: first.memory_state.stability_fast,
                    difficulty: first.memory_state.difficulty,
                }),
                0.9,
                log.elapsed_seconds as f32 / 86400.0,
            )?
            .again;
        assert_eq!(result.memory_state.stability, expected.memory.stability);
        assert_eq!(
            result.memory_state.stability_fast,
            expected.memory.stability_fast
        );
        assert_eq!(result.memory_state.difficulty, expected.memory.difficulty);
        assert_eq!(result.due_at - result.reviewed_at, log.scheduled_seconds);
        let saved = service.cards.find_by_id(card.id).await?.unwrap();
        assert_eq!(saved.memory_state, Some(result.memory_state));
        assert_eq!(saved.last_reviewed_at, Some(result.reviewed_at));
        assert_eq!(saved.due_at, Some(result.due_at));
        Ok(())
    }

    #[tokio::test]
    async fn concurrent_reviews_serialize_and_failed_save_rolls_back() -> Result<()> {
        let (service, _dir) = setup().await?;
        let card = service.create(&input()).await?;
        let (a, b) = tokio::join!(
            service.review(card.id, ReviewAnswer::Remembered),
            service.review(card.id, ReviewAnswer::Forgot)
        );
        a?;
        b?;
        let history = service.review_history(card.id).await?;
        assert_eq!(history.len(), 2);
        assert_eq!(history[1].memory_before, Some(history[0].memory_after));
        let before = service.cards.find_by_id(card.id).await?.unwrap();
        sqlx::raw_sql("CREATE TRIGGER fail_update BEFORE UPDATE ON CARDS BEGIN SELECT RAISE(ABORT, 'test failure'); END;")
            .execute(service.cards.pool()).await?;
        assert!(
            service
                .review(card.id, ReviewAnswer::Remembered)
                .await
                .is_err()
        );
        assert_eq!(service.cards.find_by_id(card.id).await?.unwrap(), before);
        assert_eq!(service.review_history(card.id).await?.len(), 2);
        Ok(())
    }

    #[tokio::test]
    async fn scheduling_limits_and_invalid_states() -> Result<()> {
        let (service, _dir) = setup().await?;
        assert!(
            CardService::with_scheduling(
                service.cards.clone(),
                SchedulingOptions {
                    desired_retention: f32::NAN,
                    max_interval_days: 1
                }
            )
            .is_err()
        );
        assert!(
            CardService::with_scheduling(
                service.cards.clone(),
                SchedulingOptions {
                    desired_retention: 0.9,
                    max_interval_days: 0
                }
            )
            .is_err()
        );
        let limited = CardService::with_scheduling(
            service.cards.clone(),
            SchedulingOptions {
                desired_retention: 0.9,
                max_interval_days: 1,
            },
        )?;
        let mut card = service.create(&input()).await?;
        let result = limited.review(card.id, ReviewAnswer::Remembered).await?;
        assert!(result.due_at - result.reviewed_at <= 86400);
        card = service.cards.find_by_id(card.id).await?.unwrap();
        assert!(
            service
                .transition(
                    &card,
                    ReviewAnswer::Forgot,
                    card.last_reviewed_at.unwrap() - 1
                )
                .is_err()
        );
        card.last_reviewed_at = None;
        assert!(
            service
                .transition(&card, ReviewAnswer::Forgot, now()?)
                .is_err()
        );
        service.reset_memory(card.id).await?;
        let fresh = service.review(card.id, ReviewAnswer::Remembered).await?;
        assert_eq!(fresh.memory_state, result.memory_state);
        Ok(())
    }

    #[tokio::test]
    async fn due_queue_and_delete_rollback() -> Result<()> {
        let (service, _dir) = setup().await?;
        let card = service.create(&input()).await?;
        assert!(service.list_due(10).await?.is_empty());
        service.review(card.id, ReviewAnswer::Remembered).await?;
        assert!(service.list_due(10).await?.is_empty());
        sqlx::query("UPDATE CARDS SET due_at = 0 WHERE id = ?")
            .bind(card.id)
            .execute(service.cards.pool())
            .await?;
        assert_eq!(service.list_due(1).await?[0].id, card.id);
        assert!(service.list_due(0).await?.is_empty());
        sqlx::raw_sql("CREATE TRIGGER fail_note_delete BEFORE DELETE ON NOTES BEGIN SELECT RAISE(ABORT, 'test failure'); END;")
            .execute(service.cards.pool()).await?;
        assert!(service.delete(card.id).await.is_err());
        assert_eq!(
            service.cards.find_by_id(card.id).await?.unwrap().note_id,
            card.note_id
        );
        assert_eq!(service.review_history(card.id).await?.len(), 1);
        Ok(())
    }
}
