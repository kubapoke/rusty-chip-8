#[derive(Debug)]
pub struct EmulatorConfig {
    pub shift_behaviour: ShiftBehaviour,
    pub offset_jump_behaviour: OffsetJumpBehaviour,
    pub add_to_index_behaviour: AddToIndexBehaviour,
}

impl EmulatorConfig {
    pub fn new(
        shift_behaviour: ShiftBehaviour,
        offset_jump_behaviour: OffsetJumpBehaviour,
        add_to_index_behaviour: AddToIndexBehaviour
    ) -> Self {
        Self {
            shift_behaviour,
            offset_jump_behaviour,
            add_to_index_behaviour,
        }
    }
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self::new(
            ShiftBehaviour::CopyVY,
            OffsetJumpBehaviour::AddV0,
            AddToIndexBehaviour::Overflow
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum ShiftBehaviour {
    CopyVY,
    IgnoreVY,
}

#[derive(Debug, PartialEq)]
pub enum OffsetJumpBehaviour {
    AddV0,
    AddVX,
}

#[derive(Debug, PartialEq)]
pub enum AddToIndexBehaviour {
    Overflow,
    NoOverflow,
}
