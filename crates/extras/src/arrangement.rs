/// Logical main-axis arrangement for items inside a horizontal container such as
/// `ButtonGroup` or docked `Toolbar`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ContentArrangement {
    /// Packs items toward the leading edge (respects RTL).
    #[default]
    Start,
    /// Centers items within the available horizontal space.
    Center,
    /// Packs items toward the trailing edge (respects RTL).
    End,
    /// Distributes remaining space evenly between items.
    SpaceBetween,
    /// Distributes remaining space around each item, with half-size outer margins.
    SpaceAround,
    /// Distributes remaining space equally between items and both outer edges.
    SpaceEvenly,
}
