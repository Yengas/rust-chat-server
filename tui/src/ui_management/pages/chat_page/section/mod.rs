pub mod usage;

pub trait SectionActivation {
    fn activate(&mut self);
    fn deactivate(&mut self);
}
