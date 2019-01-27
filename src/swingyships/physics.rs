use b2::{ContactListener};
use wrapped2d::collision::Manifold;
use wrapped2d::dynamics::world::callbacks::{ContactImpulse, ContactAccess};

pub struct FixRestitutionListener {

}

impl<U: wrapped2d::user_data::UserDataTypes> ContactListener<U> for FixRestitutionListener {
    fn begin_contact(&mut self, access:ContactAccess<U>) {
        access.contact.set_restitution(access.fixture_a.restitution() * access.fixture_b.restitution())
    }
    fn end_contact(&mut self, access:ContactAccess<U>) {

    }
    fn pre_solve(&mut self, access:ContactAccess<U>, _: &Manifold) {

    }
    fn post_solve(&mut self, access:ContactAccess<U>, impulse: &ContactImpulse) {
        println!("contact with impulses {:?}, {:?}", impulse.normal_impulses, impulse.tangent_impulses)
    }
}
