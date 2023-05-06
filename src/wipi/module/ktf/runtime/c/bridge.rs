use crate::{
    core::arm::{allocator::Allocator, ArmCore},
    wipi::c::{CBridge, CBridgeMethod, CResult},
};

pub struct KtfCBridge {
    core: ArmCore,
}

impl KtfCBridge {
    pub fn new(core: ArmCore) -> Self {
        Self { core }
    }
}

impl CBridge for KtfCBridge {
    fn alloc(&mut self, size: u32) -> CResult<u32> {
        Allocator::alloc(&mut self.core, size)
    }

    fn write_raw(&mut self, address: u32, data: &[u8]) -> CResult<()> {
        self.core.write_raw(address, data)
    }

    fn register_function(&mut self, method: CBridgeMethod) -> CResult<u32> {
        self.core.register_function(move |core: ArmCore, a0: u32, a1: u32, a2: u32| {
            let mut context = KtfCBridge::new(core);

            let result = method(&mut context, vec![a0, a1, a2])?;

            Ok::<_, anyhow::Error>(result)
        })
    }
}
