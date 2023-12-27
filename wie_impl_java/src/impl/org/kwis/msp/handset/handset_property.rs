use alloc::vec;

use crate::{
    base::{JavaClassProto, JavaContext, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::{JavaObjectProxy, JvmClassInstanceProxy},
    r#impl::java::lang::String,
};

// class org.kwis.msp.handset.HandsetProperty
pub struct HandsetProperty {}

impl HandsetProperty {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new(
                "getSystemProperty",
                "(Ljava/lang/String;)Ljava/lang/String;",
                Self::get_system_property,
                JavaMethodFlag::NONE,
            )],
            fields: vec![],
        }
    }

    async fn get_system_property(context: &mut dyn JavaContext, name: JvmClassInstanceProxy<String>) -> JavaResult<JavaObjectProxy<String>> {
        let name = String::to_rust_string(context, &name.class_instance)?;
        tracing::warn!("stub org.kwis.msp.handset.HandsetProperty::getSystemProperty({})", name);

        let result = String::from_rust_string(context, "").await?;
        Ok(result)
    }
}
