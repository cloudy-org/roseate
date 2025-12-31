// Where I dump functions temporally that I don't know where to place.
// Don't just be lazy and dump everything in here ~~like me~~.

use cirrus_softbinds::v1::keys::Keys;
use egui::{Context, InputState, Key, Modifiers};

use crate::error::Result;

// TODO: move into cirrus once matured enough.
pub fn ctx_input_with_soft_binds<KFunc, MFunc>(ctx: &Context, key_string: &String, key_map: KFunc, modifiers_map: MFunc) -> Result<bool>
where
    KFunc: Fn(&InputState, Key) -> bool,
    MFunc: FnOnce(&InputState, Modifiers) -> bool,
{
    let keys =  Keys::new(key_string)?;

    let (egui_keys, egui_modifiers) = (
        keys.egui_keys()?, keys.egui_modifiers()
    );

    Ok(
        ctx.input(|i| {
            egui_keys.iter().all(|&key| key_map(i, key)) && modifiers_map(i, egui_modifiers)
        })
    )
}