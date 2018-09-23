use common::prelude::*;

use core::*;
use game::prelude::*;
use game::entities::*;

use game::core::*;


use common::hex::AxialCoord;
use common::hex::CartVec;
use common::color::Color;
use common::Rect;
use std::collections::HashMap;

use game::entities::Taxon;
use graphics::ImageIdentifier;
use std::collections::VecDeque;


pub struct ItemRenderer {
    items_to_draw: Vec<Entity>,
    cached_game_time : Option<GameEventClock>,
    item_images_by_taxon : HashMap<Taxon, ImageIdentifier>,
}

impl ItemRenderer {
    pub fn new() -> ItemRenderer {
        ItemRenderer {
            items_to_draw: Vec::new(),
            cached_game_time : None,
            item_images_by_taxon : HashMap::new(),
        }
    }

    fn identify_relevant_entities(&mut self, world : &WorldView) {
        self.items_to_draw.clear();
        for (id, item_data) in world.entities_with_data::<ItemData>() {
            // draw it if either it is in the inventory of a hex (meaning it's on the hex) or it has no holder and it does have position data
            // so it is acting as an entity of its own
            if let Some(in_inventory) = item_data.in_inventory_of {
                if world.has_data::<TileData>(in_inventory) {
                    println!("Item in tile inventory, will draw");
                    self.items_to_draw.push(*id);
                }
            } else if world.has_data::<PositionData>(*id) {
                println!("Pushing item to draw on its own");
                self.items_to_draw.push(*id);
            }
        }
//        for (id, stack_data) in world.entities_with_data::<StackData>() {
//
//        }
    }

    pub fn draw_at(world: &WorldView, item : Entity) -> Option<AxialCoord> {
        if let Some(hex) = world.data_opt::<ItemData>(item).and_then(|item_data| item_data.in_inventory_of).and_then(|inv| world.data_opt::<TileData>(inv)).map(|td| td.position) {
            Some(hex)
        } else if let Some(pos_data) = world.data_opt::<PositionData>(item) {
            Some(pos_data.hex)
        } else {
            None
        }
    }

    pub fn image_for(view : &WorldView, resources : &mut GraphicsResources, kind : &Taxon) -> ImageIdentifier {
        let mut taxon_queue = VecDeque::new();
        taxon_queue.push_front(kind);

        while ! taxon_queue.is_empty() {
            if let Some(taxon) = taxon_queue.pop_back() {
                let image_ident = format!("entities/items/{}", taxon.name());
                if resources.is_valid_texture(image_ident.clone()) {
                    return image_ident;
                } else {
                    for parent in taxon.parents(view) {
                        taxon_queue.push_front(parent);
                    }
                }
            }
        }

        strf("entities/items/default")
    }

    pub fn cached_image_for(view : &WorldView, item_images_by_taxon : &mut HashMap<Taxon, ImageIdentifier>, resources : &mut GraphicsResources, ident_data : &IdentityData) -> ImageIdentifier {
        if let Some(item_image) = item_images_by_taxon.get(ident_data.main_kind()) {
            item_image.clone()
        } else {
            let mut taxon_queue = VecDeque::new();
            taxon_queue.push_front(ident_data.main_kind());

            while ! taxon_queue.is_empty() {
                if let Some(taxon) = taxon_queue.pop_back() {
                    let image_ident = format!("entities/items/{}", taxon.name());
                    if resources.is_valid_texture(image_ident.clone()) {
                        item_images_by_taxon.insert(ident_data.main_kind().clone(), image_ident.clone());
                        return image_ident;
                    } else {
                        for parent in taxon.parents(view) {
                            taxon_queue.push_front(parent);
                        }
                    }
                }
            }

            strf("entities/items/default")
        }
    }

    pub fn render_items(&mut self, world: &WorldView, resources : &mut GraphicsResources, _bounds : Rect<f32>) -> DrawList {
        if self.cached_game_time != Some(world.current_time) {
            self.identify_relevant_entities(world);
            self.cached_game_time = Some(world.current_time);
        }

        let mut quads : Vec<Quad> = vec![];

        for ent in &self.items_to_draw {
            let ent = *ent;

            let ident_data = world.data::<IdentityData>(ent);
            let graphics_data = world.data::<GraphicsData>(ent);

            // Main item display
            if let Some(pos) = ItemRenderer::draw_at(world, ent) {
                let pos = pos.as_cart_vec();
                let img = ItemRenderer::cached_image_for(world, &mut self.item_images_by_taxon, resources, ident_data);
                let quad = Quad::new(img, pos.0).centered().color(graphics_data.color);
                quads.push(quad);
            } else {
                warn!("Tried to draw item {} but could not identify a position to do so", ent);
            }
        }
        DrawList {
            quads,
            ..Default::default()
        }
    }
}