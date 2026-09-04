#[derive(Debug)]
pub struct EmulatorConfig {
    pub shift_behaviour: ShiftBehaviour,
    pub offset_jump_behaviour: OffsetJumpBehaviour,
}

impl EmulatorConfig {
    pub fn new(shift_behaviour: ShiftBehaviour, offset_jump_behaviour: OffsetJumpBehaviour) -> Self {
        Self {
            shift_behaviour,
            offset_jump_behaviour,
        }
    }
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self::new(ShiftBehaviour::CopyVY, OffsetJumpBehaviour::AddV0)
    }
}

#[derive(Debug)]
pub enum ShiftBehaviour {
    CopyVY,
    IgnoreVY,
}

#[derive(Debug)]
pub enum OffsetJumpBehaviour {
    AddV0,
    AddVX,
}
