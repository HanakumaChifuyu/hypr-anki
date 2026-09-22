use std::rc::Rc;

use slint::{ComponentHandle, DataTransfer, Model};

use super::main_app_window::{AppWindow, DeckTreeCallback, FlatNode};

// 私有类型避免把文本 / 外部文件拖放误认为卡组移动。
struct DeckDrag(i32);

fn source_id(data: &DataTransfer) -> Option<i32> {
    data.user_data()?
        .downcast_ref::<DeckDrag>()
        .map(|drag| drag.0)
}

fn can_move(nodes: impl IntoIterator<Item = FlatNode>, source: i32, target: i32) -> bool {
    if source == target {
        return false;
    }
    let nodes: Vec<_> = nodes.into_iter().collect();
    let Some(source_index) = nodes.iter().position(|node| node.id == source) else {
        return false;
    };
    let Some(target_index) = nodes.iter().position(|node| node.id == target) else {
        return false;
    };
    // 可见先序列表中，源节点之后连续的更深节点均是其后代。
    // 禁止拖到自己或自己的后代；持久化时仍由业务后端校验完整树。
    !(target_index > source_index
        && nodes[source_index + 1..=target_index]
            .iter()
            .all(|node| node.level > nodes[source_index].level))
}

/// 给 Slint 的全局回调注册 Rust 处理函数；调用 bind 本身不会执行拖拽或移动卡组。
pub fn bind(app: &AppWindow) {
    let callbacks = app.global::<DeckTreeCallback>();

    // Slint 求值 `DragArea.data: DeckTreeCallback.drag-data(root.node.id)` 时调用。
    // 这是属性绑定中的取值回调，不是“开始拖拽”事件：可能在拖拽前求值，
    // 也可能在依赖变化后重新求值，因此这里只构造数据，不执行任何业务操作。
    callbacks.on_drag_data(|id| {
        let mut data = DataTransfer::default();
        // 将源卡组 ID 包装成私有类型，目标区域随后通过 source_id 解码。
        data.set_user_data(Rc::new(DeckDrag(id)));
        data
    });

    // 回调由窗口持有；闭包只捕获弱引用，避免再次强持有窗口而形成引用环。
    let weak = app.as_weak();

    // 拖拽经过目标卡组时，DropArea.can-drop 会反复调用这里，询问能否放下。
    // 当前 Slint 的 dropped 处理也会在松手时再次调用它。
    // true 会被 UI 转成 DragAction.move，显示落点反馈；false 则拒绝该落点。
    // 此处只校验，不移动卡组；_position 暂未使用，因为三个落点共用这套基础校验。
    callbacks.on_can_drop(move |data, target, _position| {
        let (Some(app), Some(source)) = (weak.upgrade(), source_id(&data)) else {
            // 窗口已释放，或载荷不是本应用的 DeckDrag，都拒绝接收。
            return false;
        };
        can_move(app.get_decks().iter(), source, target)
    });

    let weak = app.as_weak();

    // 在允许的目标上松手后，Slint 的 DropArea.dropped 调用 drop-requested。
    // 取消拖拽或放到不允许的位置时，不会走到这里。
    callbacks.on_drop_requested(move |data, target, position| {
        let (Some(app), Some(source)) = (weak.upgrade(), source_id(&data)) else {
            return;
        };
        // 使用当前列表再次校验，避免根据拖拽途中已经过期的数据提交移动请求。
        if can_move(app.get_decks().iter(), source, target) {
            // TODO: 接入后端卡组移动业务逻辑，处理 source / target / position，
            app.global::<DeckTreeCallback>()
                .invoke_move_requested(source, target, position);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_cycles_and_stale_ids_in_visible_preorder() {
        let nodes = [(10, 0), (20, 1), (30, 2), (40, 1), (50, 0)].map(|(id, level)| FlatNode {
            id,
            level,
            ..Default::default()
        });
        assert!(!can_move(nodes.clone(), 10, 30));
        assert!(!can_move(nodes.clone(), 20, 30));
        assert!(!can_move(nodes.clone(), 20, 20));
        assert!(!can_move(nodes.clone(), 99, 20));
        assert!(!can_move(nodes.clone(), 20, 99));
        assert!(can_move(nodes.clone(), 20, 40));
        assert!(can_move(nodes.clone(), 20, 50));
        assert!(can_move(nodes, 30, 10));
    }
}
