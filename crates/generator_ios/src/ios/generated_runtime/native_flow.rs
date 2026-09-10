#[derive(Clone, Copy, PartialEq, Eq)]
enum NativeFlow {
    Block,
    GridItem,
    Inline,
}

impl NativeFlow {
    fn is_block(self) -> bool {
        matches!(self, Self::Block | Self::GridItem)
    }

    fn is_grid_item(self) -> bool {
        matches!(self, Self::GridItem)
    }

    fn is_flex_item(self) -> bool {
        !self.is_grid_item()
    }

    fn is_inline(self) -> bool {
        matches!(self, Self::Inline)
    }
}
