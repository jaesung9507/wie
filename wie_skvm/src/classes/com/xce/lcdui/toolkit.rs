use alloc::vec;

use java_class_proto::{JavaFieldProto, JavaMethodProto};
use java_constants::{FieldAccessFlags, MethodAccessFlags};
use jvm::{ClassInstanceRef, Jvm, Result as JvmResult};

use wie_jvm_support::{WieJavaClassProto, WieJvmContext};
use wie_midp::classes::javax::microedition::lcdui::{Display, Font, Graphics};

// class com.xce.lcdui.Toolkit
pub struct Toolkit;

impl Toolkit {
    pub fn as_proto() -> WieJavaClassProto {
        WieJavaClassProto {
            name: "com/xce/lcdui/Toolkit",
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![JavaMethodProto::new("<clinit>", "()V", Self::cl_init, MethodAccessFlags::STATIC)],
            fields: vec![
                JavaFieldProto::new("graphics", "Ljavax/microedition/lcdui/Graphics;", FieldAccessFlags::STATIC),
                JavaFieldProto::new("DEFAULT_FONT", "Ljavax/microedition/lcdui/Font;", FieldAccessFlags::STATIC),
                JavaFieldProto::new("FONT_HEIGHT", "I", FieldAccessFlags::STATIC),
            ],
        }
    }

    async fn cl_init(jvm: &Jvm, _: &mut WieJvmContext) -> JvmResult<()> {
        tracing::debug!("com.xce.lcdui.Toolkit::<clinit>");

        let font: ClassInstanceRef<Font> = jvm
            .invoke_static("javax/microedition/lcdui/Font", "getDefaultFont", "()Ljavax/microedition/lcdui/Font;", ())
            .await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "DEFAULT_FONT", "Ljavax/microedition/lcdui/Font;", font.clone())
            .await?;

        let font_height: i32 = jvm.invoke_virtual(&font, "getHeight", "()I", ()).await?;
        jvm.put_static_field("com/xce/lcdui/Toolkit", "FONT_HEIGHT", "I", font_height).await?;

        let display: ClassInstanceRef<Display> = jvm
            .invoke_static(
                "javax/microedition/lcdui/Display",
                "getDisplay",
                "(Ljavax/microedition/midlet/MIDlet;)Ljavax/microedition/lcdui/Display;",
                (None,),
            )
            .await?;

        let graphics: ClassInstanceRef<Graphics> = jvm.get_field(&display, "screenGraphics", "Ljavax/microedition/lcdui/Graphics;").await?;

        jvm.put_static_field("com/xce/lcdui/Toolkit", "graphics", "Ljavax/microedition/lcdui/Graphics;", graphics)
            .await?;

        Ok(())
    }
}
