// Where I dump functions temporally that I don't know where to place.
// Don't just be lazy and dump everything in here ~~like me~~.

use cirrus_softbinds::v1::keys::Keys;
use egui::{InputState, Key};

use crate::error::Result;

// TODO: move into cirrus once matured enough.
pub fn get_input_reader_from_soft_binds<KFunc>(key_string: &String, key_map: KFunc) -> Result<impl FnMut(&InputState) -> bool + use<KFunc>>
where
    KFunc: Fn(&InputState, Key) -> bool,
{
    let keys =  Keys::new(key_string)?;

    let (egui_keys, egui_modifiers) = (
        keys.egui_keys()?, keys.egui_modifiers()
    );

    Ok(
        move |i: &InputState| egui_keys.iter().all(|&key| key_map(i, key) && i.modifiers.contains(egui_modifiers))
    )
}