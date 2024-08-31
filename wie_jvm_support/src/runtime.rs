use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::{
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use java_runtime::{File, FileStat, IOError, Runtime, RuntimeClassProto, SpawnCallback};
use jvm::{ClassDefinition, JavaError, Jvm, Result as JvmResult};

use wie_backend::{AsyncCallable, System};

use crate::JvmImplementation;

#[derive(Clone)]
pub struct JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    system: System,
    implementation: T,
}

impl<T> JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    pub fn new(system: System, implementation: T) -> Self {
        Self { system, implementation }
    }
}

#[async_trait::async_trait]
impl<T> Runtime for JvmRuntime<T>
where
    T: JvmImplementation + Sync + Send + 'static,
{
    async fn sleep(&self, duration: Duration) {
        let now = self.system.platform().now();
        let until = now + duration.as_millis() as u64;

        self.system.clone().sleep(until).await; // TODO remove clone
    }

    async fn r#yield(&self) {
        self.system.yield_now().await;
    }

    fn spawn(&self, callback: Box<dyn SpawnCallback>) {
        struct SpawnProxy {
            callback: Box<dyn SpawnCallback>,
        }

        #[async_trait::async_trait]
        impl AsyncCallable<Result<u32, JavaError>> for SpawnProxy {
            async fn call(mut self) -> Result<u32, JavaError> {
                self.callback.call().await;

                Ok(0) // TODO
            }
        }

        self.system.clone().spawn(SpawnProxy { callback });
    }

    fn now(&self) -> u64 {
        self.system.platform().now().raw()
    }

    fn current_task_id(&self) -> u64 {
        self.system.current_task_id()
    }

    fn stdin(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    fn stdout(&self) -> Result<Box<dyn File>, IOError> {
        #[derive(Clone)]
        struct StdoutFile {
            system: System,
        }

        #[async_trait::async_trait]
        impl File for StdoutFile {
            async fn read(&mut self, _buf: &mut [u8]) -> Result<usize, IOError> {
                Err(IOError::Unsupported)
            }

            async fn write(&mut self, buf: &[u8]) -> Result<usize, IOError> {
                self.system.platform().write_stdout(buf);

                Ok(buf.len())
            }
        }

        Ok(Box::new(StdoutFile { system: self.system.clone() }))
    }

    fn stderr(&self) -> Result<Box<dyn File>, IOError> {
        Err(IOError::Unsupported)
    }

    async fn open(&self, path: &str) -> Result<Box<dyn File>, IOError> {
        #[derive(Clone)]
        struct FileImpl {
            data: Arc<Vec<u8>>,
            cursor: Arc<AtomicU64>,
        }

        #[async_trait::async_trait]
        impl File for FileImpl {
            async fn read(&mut self, buf: &mut [u8]) -> Result<usize, IOError> {
                let cursor = self.cursor.load(Ordering::SeqCst) as usize;

                if cursor < self.data.len() {
                    let to_read = core::cmp::min(buf.len(), self.data.len() - cursor);
                    buf[..to_read].copy_from_slice(&self.data[cursor..cursor + to_read]);

                    self.cursor.fetch_add(to_read as u64, Ordering::SeqCst);

                    Ok(to_read)
                } else {
                    Ok(0)
                }
            }

            async fn write(&mut self, _buf: &[u8]) -> Result<usize, IOError> {
                Err(IOError::Unsupported)
            }
        }

        let filesystem = self.system.filesystem();

        let file = filesystem.read(path);

        file.map(|x| {
            Box::new(FileImpl {
                data: Arc::new(x.to_vec()),
                cursor: Arc::new(AtomicU64::new(0)),
            }) as _
        })
        .ok_or(IOError::NotFound)
    }

    async fn stat(&self, path: &str) -> Result<FileStat, IOError> {
        let filesystem = self.system.filesystem();

        let file = filesystem.read(path);

        file.map(|x| FileStat { size: x.len() as _ }).ok_or(IOError::NotFound)
    }

    async fn define_class_rust(&self, jvm: &Jvm, proto: RuntimeClassProto) -> JvmResult<Box<dyn ClassDefinition>> {
        self.implementation.define_class_rust(jvm, proto, Box::new(self.clone()) as Box<_>).await
    }

    async fn define_class_java(&self, jvm: &Jvm, data: &[u8]) -> JvmResult<Box<dyn ClassDefinition>> {
        self.implementation.define_class_java(jvm, data).await
    }

    async fn define_array_class(&self, _jvm: &Jvm, element_type_name: &str) -> JvmResult<Box<dyn ClassDefinition>> {
        self.implementation.define_array_class(_jvm, element_type_name).await
    }
}
