//! Material 3 Expressive data-view components (`DataTable`, `Tree`, and `VirtualList`).
//!
//! Enable the optional `data-views` feature on `gpui-material` or `gpui-material-extras`
//! to use these components without pulling in heavier component groups.

mod table;
mod tree;
mod virtual_list;

pub use table::{
    ColumnAlignment, DataTable, DataTableCell, DataTableColors, DataTableColumn, DataTableRow,
    SortDirection,
};
pub use tree::{Tree, TreeColors, TreeEvent, TreeNode, TreeRowInfo, TreeState};
pub use virtual_list::{
    VirtualList, VirtualListColors, VirtualListScrollHandle, VirtualListVariant,
};
