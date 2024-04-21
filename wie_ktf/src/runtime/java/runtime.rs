use alloc::{boxed::Box, sync::Arc, vec};
use core::time::Duration;

use wie_backend::{AsyncCallable, System};
use wie_core_arm::ArmCore;

use java_runtime::{File, IOError, Runtime};
use jvm::{Jvm, JvmCallback};

#[derive(Clone)]
pub struct KtfRuntime {
    core: ArmCore,
    system: System,
    jvm: Arc<Jvm>,
}

impl KtfRuntime {
    pub fn new(core: &mut ArmCore, system: &mut System, jvm: Arc<Jvm>) -> Self {
        Self {
            core: core.clone(),
            system: system.clone(),
            jvm,
        }
    }
}

#[async_trait::async_trait]
impl Runtime for KtfRuntime {
    async fn sleep(&self, duration: Duration) {
        let now = self.system.platform().now();
        let until = now + duration.as_millis() as u64;

        self.system.clone().sleep(until).await; // TODO remove clone
    }

    async fn r#yield(&self) {
        self.system.yield_now().await;
    }

    fn spawn(&self, callback: Box<dyn JvmCallback>) {
        struct SpawnProxy {
            jvm: Arc<Jvm>,
            callback: Box<dyn JvmCallback>,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<u32, anyhow::Error> for SpawnProxy {
            async fn call(mut self) -> Result<u32, anyhow::Error> {
                self.callback.call(&self.jvm, vec![].into_boxed_slice()).await?;

                Ok(0) // TODO
            }
        }

        self.core.clone().spawn(SpawnProxy {
            jvm: self.jvm.clone(),
            callback,
        });
    }

    fn now(&self) -> u64 {
        self.system.platform().now().raw()
    }

    fn stdin(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    fn stdout(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    fn stderr(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    async fn open(&self, _path: &str) -> Result<Box<dyn File>, IOError> {
        todo!()
    }
}
