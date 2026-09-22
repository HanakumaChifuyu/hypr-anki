use anyhow::Result;
pub mod windows;

fn main() -> Result<()> {
    windows::main_app_window::entry_point(|_app| {
        #[cfg(debug_assertions)]
        init_integration_demo(_app);
    })
}

// 主窗口集成演示：仅 debug 构建编译这些数据，release 不注入示例内容。
#[cfg(debug_assertions)]
fn init_integration_demo(app: &windows::main_app_window::AppWindow) {
    use std::rc::Rc;

    use slint::VecModel;
    use windows::main_app_window::FlatNode;

    // 模拟后端返回的可见先序列表；折叠节点的后代不包含在列表里。
    let decks = [
        (1, "Languages", 0, true),
        (2, "Japanese", 1, true),
        (3, "Vocabulary", 2, false),
        (4, "Grammar", 2, false),
        (5, "English", 1, false),
        (6, "Computer science", 0, false),
        (7, "Reading", 0, false),
    ]
    .into_iter()
    .map(|(id, name, level, is_expanded)| FlatNode {
        id,
        name: name.into(),
        level,
        is_expanded,
    })
    .collect::<Vec<_>>();

    app.set_decks(Rc::new(VecModel::from(decks)).into());
    app.set_due_today(54);
    app.set_learning(6);
    app.set_new_cards(17);
    app.set_reviewed_today(12);
    app.set_daily_goal(20);
    app.set_streak_days(7);
}
