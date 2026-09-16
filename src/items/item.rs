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

    /// Short beginner-facing explanation shown on pickup and selection.
    pub fn purpose(self) -> &'static str {
        match self {
            ItemKind::Scrap => "crafting material - used in bandages and splints",
            ItemKind::Cloth => "crafting material - used in bandages and medkits",
            ItemKind::Metal => "crafting material - used in splints and medkits",
            ItemKind::Food => "use with F to restore hunger",
            ItemKind::Water => "use with F to restore thirst",
            ItemKind::Battery => "crafting material - used in medkits",
            ItemKind::Bandage => "use with F to stop active bleeding",
            ItemKind::Splint => "use with F to stabilise a leg fracture",
            ItemKind::Medkit => "use with F to restore blood and treat pain",
        }
    }

    pub fn is_directly_usable(self) -> bool {
        matches!(
            self,
            ItemKind::Food | ItemKind::Water | ItemKind::Bandage | ItemKind::Splint | ItemKind::Medkit
        )
    }

    // how much of the pack's carry-weight budget one unit takes up.
    pub fn weight(self) -> u32 {
        match self {
            ItemKind::Scrap | ItemKind::Cloth | ItemKind::Metal => 1,
            ItemKind::Food | ItemKind::Water | ItemKind::Battery => 1,
            ItemKind::Bandage | ItemKind::Splint => 1,
            ItemKind::Medkit => 2,
        }
    }

    pub fn sprite_path(self) -> &'static str {
        match self {
            ItemKind::Scrap => "sprites/item_scrap.png",
            ItemKind::Cloth => "sprites/item_cloth.png",
            ItemKind::Metal => "sprites/item_metal.png",
            ItemKind::Food => "sprites/item_food.png",
            ItemKind::Water => "sprites/item_water.png",
            ItemKind::Battery => "sprites/item_battery.png",
            ItemKind::Bandage => "sprites/item_bandage.png",
            ItemKind::Splint => "sprites/item_splint.png",
            ItemKind::Medkit => "sprites/item_medkit.png",
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
