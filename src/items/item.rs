//! item kinds and stacks. pure data, no bevy.

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ItemKind {
    Scrap,
    Cloth,
    Metal,
    Food,
    Water,
    Battery,
    Bandage,
    Splint,
    Medkit,
}

impl ItemKind {
    pub fn label(self) -> &'static str {
        match self {
            ItemKind::Scrap => "scrap",
            ItemKind::Cloth => "cloth",
            ItemKind::Metal => "metal",
            ItemKind::Food => "food",
            ItemKind::Water => "water",
            ItemKind::Battery => "battery",
            ItemKind::Bandage => "bandage",
            ItemKind::Splint => "splint",
            ItemKind::Medkit => "medkit",
        }
    }

    // how much of an inventory slot's carry weight one unit takes up.
    pub fn weight(self) -> u32 {
        match self {
            ItemKind::Scrap | ItemKind::Cloth | ItemKind::Metal => 1,
            ItemKind::Food | ItemKind::Water | ItemKind::Battery => 1,
            ItemKind::Bandage | ItemKind::Splint => 1,
            ItemKind::Medkit => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemStack {
    pub kind: ItemKind,
    pub quantity: u32,
}

impl ItemStack {
    pub fn new(kind: ItemKind, quantity: u32) -> Self {
        Self { kind, quantity }
    }

    pub fn total_weight(&self) -> u32 {
        self.kind.weight() * self.quantity
    }
}
