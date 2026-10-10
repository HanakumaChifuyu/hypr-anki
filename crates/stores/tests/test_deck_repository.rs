use anyhow::Result;
use stores::db::pool::SqlitePool;
use stores::models::deck::{CreateDeck, UpdateDeck};
use stores::repository::deck::DeckRepository;
use tempfile::tempdir;

async fn setup_test_db() -> Result<(DeckRepository, tempfile::TempDir)> {
    let tempdir = tempdir()?;
    let db_path = tempdir.path().join("test_deck.db");
    let pool = SqlitePool::new(db_path).await?;

    // 运行 migrations 执行全套数据库迁移
    sqlx::migrate!("../../migrations").run(pool.inner()).await?;

    Ok((DeckRepository::new(pool), tempdir))
}

#[tokio::test]
async fn test_deck_crud() -> Result<()> {
    let (repo, _tempdir) = setup_test_db().await?;

    // 1. 测试创建根牌组 (Create)
    let root = repo.create_simple("Default", None, 1).await?;
    assert_eq!(root.id, 1);
    assert_eq!(root.name, "Default");
    assert!(!root.is_collapsed);
    assert_eq!(root.parent_id, None);
    assert_eq!(root.config_id, 1);

    // 2. 测试创建子牌组
    let child_input = CreateDeck::new("Vocab", 1)
        .with_parent(Some(root.id))
        .with_collapsed(true);
    let child = repo.create(&child_input).await?;
    assert_eq!(child.id, 2);
    assert_eq!(child.name, "Vocab");
    assert!(child.is_collapsed);
    assert_eq!(child.parent_id, Some(root.id));

    // 3. 测试查询 (Read)
    let found = repo.find_by_id(root.id).await?;
    assert_eq!(found.unwrap().name, "Default");

    let not_found = repo.find_by_id(999).await?;
    assert!(not_found.is_none());

    let all_decks = repo.find_all().await?;
    assert_eq!(all_decks.len(), 2);

    let root_decks = repo.find_root_decks().await?;
    assert_eq!(root_decks.len(), 1);
    assert_eq!(root_decks[0].name, "Default");

    let children = repo.find_by_parent_id(Some(root.id)).await?;
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].name, "Vocab");

    let found_by_name = repo.find_by_name("Default").await?;
    assert_eq!(found_by_name.unwrap().id, root.id);

    assert!(repo.exists_by_id(root.id).await?);
    assert!(!repo.exists_by_id(999).await?);
    assert_eq!(repo.count().await?, 2);

    // 4. 测试更新 (Update)
    // 更新名称
    let updated = repo.update_name(child.id, "N2").await?;
    assert!(updated);
    let reloaded_child = repo.find_by_id(child.id).await?.unwrap();
    assert_eq!(reloaded_child.name, "N2");

    // 更新折叠状态
    repo.update_collapsed(child.id, false).await?;
    let reloaded_child = repo.find_by_id(child.id).await?.unwrap();
    assert!(!reloaded_child.is_collapsed);

    // 移动牌组（解除父牌组关系，变为根牌组）
    repo.update_parent(child.id, None).await?;
    let root_decks_now = repo.find_root_decks().await?;
    assert_eq!(root_decks_now.len(), 2);

    // 部分更新 (update_partial)
    let update_dto = UpdateDeck {
        name: Some("Advanced".into()),
        config_id: Some(2),
        ..Default::default()
    };
    let partially_updated = repo.update_partial(child.id, &update_dto).await?;
    assert!(partially_updated.is_some());
    let u = partially_updated.unwrap();
    assert_eq!(u.name, "Advanced");
    assert_eq!(u.config_id, 2);

    // 5. 测试删除 (Delete)
    let deleted = repo.delete_by_id(child.id).await?;
    assert!(deleted);
    assert_eq!(repo.count().await?, 1);

    // 批量删除
    let deleted_batch = repo.delete_by_ids(&[root.id, 999]).await?;
    assert_eq!(deleted_batch, 1);
    assert_eq!(repo.count().await?, 0);

    Ok(())
}
