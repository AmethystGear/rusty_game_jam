use gdnative::prelude::*;

/// type alias for nullable GodotObject properties
pub type PropRef<T> = Option<Ref<T>>;

/// return the property, panic if it's null
pub fn get_prop<T>(r: &PropRef<T>) -> &Ref<T>
where
    T: GodotObject,
{
    r.as_ref().expect("property was not assigned in inspector!")
}
