use crate::{op::Op, state::VmState};

pub trait Backend {
    fn run(&mut self, op: Op, vm_state: &VmState);
}

pub struct NoopBackend;

impl Backend for NoopBackend {
    fn run(&mut self, op: Op, vm_state: &VmState) {}
}
