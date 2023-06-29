use bevy::{
    prelude::{Component, Deref, DerefMut, Entity, Res, Resource},
    reflect::{FromReflect, Reflect},
    sprite::TextureAtlasSprite,
};

// TODO: `Inventory` and `Item` should be moduled separately in `src/items/mod.rs`.
// Contains vector of Items. This Component is under the Player/NPC's root Bundle.
#[derive(Default, Component, Reflect)]
pub struct Inventory {
    pub is_opening: bool,
    pub fully_opened: bool,
    // TODO: Maybe we should consider about the size of Inventory, or data structure of it. (Array, BTreeMap, HashMap, etc...)
    // Currently I choose Array with fixed size, because I think static size is enough for implementing inventory.
    pub items: [Item; 32],
}

impl Inventory {
    // If Item is drop into certain index of Inventory UI, it will be pushed to `Inventory` struct.
    // If an item already exists in that index, it will be popped out.
    pub fn push(&mut self, index: Option<usize>, mut item: Item) -> Option<Item> {
        if let Some(index) = index {
            if let Some(prev_item) = self.items.get_mut(index) {
                item.location = Location::Inventory;

                // If inventory of provided index is empty, item will be pushed and nothing returned.
                if prev_item.entity == Entity::from_raw(0) {
                    *prev_item = item;

                    None
                } else {
                    // If an item already exists in provided index, previous item will be popped out.
                    let mut popped_item = *prev_item;
                    popped_item.location = Location::MousePickup;
                    *prev_item = item;

                    Some(popped_item)
                }
            } else {
                // if index is out of range, pushed item will be returned.
                Some(item)
            }
        } else {
            // If index is None, it will be pushed to the first empty index.
            let mut full_inventory_flag = true;
            for items in self.items.iter_mut() {
                if items.entity == Entity::from_raw(0) {
                    item.location = Location::Inventory;
                    *items = item;
                    full_inventory_flag = false;
                    break;
                }
            }

            // If inventory is full, pushed item will be returned.
            if full_inventory_flag {
                Some(item)
            } else {
                None
            }
        }
    }

    pub fn pop(&mut self, index: usize) -> Option<Item> {
        if let Some(item) = self.items.get_mut(index) {
            if item.entity == Entity::from_raw(0) {
                // If inventory of provided index is empty, nothing will be returned.
                None
            } else {
                // If an item exists in provided index, it will be popped out.
                let mut popped_item = *item;
                popped_item.location = Location::MousePickup;
                *item = Item::default();

                Some(popped_item)
            }
        } else {
            None
        }
    }

    pub fn get_item_info<'a>(
        &self,
        inventory_index: usize,
        item_dictionary: &'a ItemDictionary,
    ) -> Option<&'a ItemInfo> {
        if let Some(item) = self.items.get(inventory_index) {
            if item.entity == Entity::from_raw(0) {
                None
            } else {
                item_dictionary.0.get(item.item_info_index)
            }
        } else {
            None
        }
    }
}

// TODO: Maybe it should be `ItemBundle` with Sprite images.
#[derive(Copy, Clone, Component, Reflect)]
pub struct Item {
    pub location: Location,
    // iten index from `ItemDictionary`
    pub item_info_index: usize,
    pub count: u32,
    pub entity: Entity,
}

#[derive(Copy, Clone, Reflect)]
pub enum Location {
    // drop on the field.
    FieldDropped,
    // installed on the field like furniture.
    FieldInstalled,
    Inventory,
    Equipped,
    // inside the monster.
    Loot,
    // picked up by mouse.
    MousePickup,
}

#[derive(Deref, DerefMut, Resource)]
pub struct ItemDictionary(Vec<ItemInfo>);

// `ItemInfo` stored in item_info_index 0 of `ItemDictionary` is empty info.
impl Default for ItemDictionary {
    fn default() -> Self {
        ItemDictionary(vec![ItemInfo::default()])
    }
}

#[derive(Default)]
pub struct ItemInfo {
    pub name: String,
    pub description: String,
    pub icon: TextureAtlasSprite,
    pub item_type: ItemType,
}

#[derive(Default, Copy, Clone, Reflect)]
pub enum ItemType {
    #[default]
    Consumable,
    Weapon,
    Armor,
}

impl Default for Item {
    // Default Item means empty.
    fn default() -> Self {
        Item {
            location: Location::FieldDropped,
            item_info_index: 0,
            count: 0,
            entity: Entity::from_raw(0),
        }
    }
}

// Change State to `InventoryOpenedState` when push I key.
pub fn open_inventory() {}

// Draw Inventory UI when `InventoryOpenedState` with popping out animation from right side.
pub fn draw_inventory_ui() {}
