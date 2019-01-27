use b2::{ContactListener, Vec2};
use wrapped2d::collision::Manifold;
use wrapped2d::dynamics::world::callbacks::{ContactImpulse, ContactAccess};
use std::cell::RefCell;
use std::rc::Rc;

pub struct FixRestitutionListener {
    pub big_impacts: Rc<RefCell<Vec<(Vec2, f32)>>>
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
        if impulse.normal_impulses[0] > 500. {
            let w_manifold = access.contact.world_manifold();
            self.big_impacts.borrow_mut().push((w_manifold.points[0] + w_manifold.points[1] / 2., impulse.normal_impulses[0]));
        }
    }
}
