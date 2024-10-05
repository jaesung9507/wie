use alloc::vec;

use java_class_proto::JavaMethodProto;
use java_runtime::classes::java::lang::String;
use jvm::{ClassInstanceRef, Jvm, Result};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};

// class javax.microedition.rms.InvalidRecordIDException
pub struct InvalidRecordIDException;

impl InvalidRecordIDException {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "javax/microedition/rms/InvalidRecordIDException",
            parent_class: Some("javax/microedition/rms/RecordStoreException"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, Default::default()),
                JavaMethodProto::new("<init>", "(Ljava/lang/String;)V", Self::init_with_message, Default::default()),
            ],
            fields: vec![],
        }
    }

    async fn init(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>) -> Result<()> {
        tracing::debug!("javax.microedition.rms.InvalidRecordIDException::<init>({:?})", &this);

        let _: () = jvm
            .invoke_special(&this, "javax/microedition/rms/RecordStoreException", "<init>", "()V", ())
            .await?;

        Ok(())
    }

    async fn init_with_message(jvm: &Jvm, _: &mut WieJvmContext, this: ClassInstanceRef<Self>, message: ClassInstanceRef<String>) -> Result<()> {
        tracing::debug!("javax.microedition.rms.InvalidRecordIDException::<init>({:?}, {:?})", &this, &message);

        let _: () = jvm
            .invoke_special(
                &this,
                "javax/microedition/rms/RecordStoreException",
                "<init>",
                "(Ljava/lang/String;)V",
                (message,),
            )
            .await?;

        Ok(())
    }
}
