mod list;
pub use list::*;

mod lookup;
pub use lookup::*;


pub trait LookupSubtable<E>: Sized {
    fn new_lookup() -> E;
    fn get_lookup_variant(_: &E) -> Option<&Lookup<Self>>;
    fn get_lookup_variant_mut(_: &mut E) -> Option<&mut Lookup<Self>>;
}
