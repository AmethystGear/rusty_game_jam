use crate::animal::blend_animals;
use crate::animal::create_animal;
use crate::animal::BodyGradient;
use crate::animal_templates;
use crate::prop_ref::*;
use gdnative::api::*;
use gdnative::prelude::*;

#[derive(NativeClass)]
#[inherit(Spatial)]
pub struct AnimalSpawner {
    #[property]
    material: PropRef<ShaderMaterial>,
    #[property]
    texture_block_size_x: f32,
    #[property]
    texture_block_size_y: f32,
    #[property]
    animal_script: PropRef<Script>,
}

#[methods]
impl AnimalSpawner {
    fn new(_: &Spatial) -> Self {
        Self {
            material: None,
            animal_script: None,
            texture_block_size_x: 1.0,
            texture_block_size_y: 1.0,
        }
    }

    #[export]
    fn _ready(&mut self, owner: &Spatial) {

        let gradients = [
            BodyGradient(vec![1.0, 1.0, 0.75, 0.2, 0.6, 0.1, 0.0, 0.0]),
            BodyGradient(vec![0.0, 0.0, 0.25, 1.0, 0.4, 0.9, 1.0, 1.0]),
        ];

        let combined = [animal_templates::turtle(), animal_templates::fox()]
            .into_iter()
            .zip(gradients)
            .collect::<Vec<_>>();

        let (animal, size, center) = create_animal(
            &blend_animals(&combined), /*&animal_templates::fox(), */ /*&animal_templates::chicken()*/ /*&animal_templates::turtle(),  &blend_animals(&combined) */
            get_prop(&self.animal_script),
            get_prop(&self.material),
            (self.texture_block_size_x, self.texture_block_size_y),
        );
    

        animal.translate(Vector3::new(0.0, size.y + 2.0, 0.0));

        owner.add_child(animal, false);
    }
}
