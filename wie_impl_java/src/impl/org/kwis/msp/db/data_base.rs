use alloc::vec;

use bytemuck::cast_slice;

use jvm::JavaValue;

use crate::{
    array::Array,
    base::{JavaClassProto, JavaContext, JavaFieldAccessFlag, JavaFieldProto, JavaMethodFlag, JavaMethodProto, JavaResult},
    proxy::{JavaObjectProxy, JvmClassInstanceProxy},
    r#impl::java::lang::String,
};

// class org.kwis.msp.db.DataBase
pub struct DataBase {}

impl DataBase {
    pub fn as_proto() -> JavaClassProto {
        JavaClassProto {
            parent_class: Some("java/lang/Object"),
            interfaces: vec![],
            methods: vec![
                JavaMethodProto::new("<init>", "()V", Self::init, JavaMethodFlag::NONE),
                JavaMethodProto::new(
                    "openDataBase",
                    "(Ljava/lang/String;IZ)Lorg/kwis/msp/db/DataBase;",
                    Self::open_data_base,
                    JavaMethodFlag::NONE,
                ),
                JavaMethodProto::new("getNumberOfRecords", "()I", Self::get_number_of_records, JavaMethodFlag::NONE),
                JavaMethodProto::new("closeDataBase", "()V", Self::close_data_base, JavaMethodFlag::NONE),
                JavaMethodProto::new("insertRecord", "([BII)I", Self::insert_record, JavaMethodFlag::NONE),
                JavaMethodProto::new("selectRecord", "(I)[B", Self::select_record, JavaMethodFlag::NONE),
            ],
            fields: vec![JavaFieldProto::new("dbName", "Ljava/lang/String;", JavaFieldAccessFlag::NONE)],
        }
    }
    async fn init(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, data_base_name: JvmClassInstanceProxy<String>) -> JavaResult<()> {
        tracing::warn!(
            "stub org.kwis.msp.db.DataBase::<init>({:#x}, {:#x})",
            context.instance_raw(&this.class_instance),
            context.instance_raw(&data_base_name.class_instance)
        );

        context.jvm().put_field(
            &this.class_instance,
            "dbName",
            "Ljava/lang/String;",
            JavaValue::Object(Some(data_base_name.class_instance)),
        )?;

        Ok(())
    }

    async fn open_data_base(
        context: &mut dyn JavaContext,
        data_base_name: JavaObjectProxy<String>,
        record_size: i32,
        create: i32,
    ) -> JavaResult<JavaObjectProxy<DataBase>> {
        tracing::warn!(
            "stub org.kwis.msp.db.DataBase::openDataBase({:#x}, {}, {})",
            data_base_name.ptr_instance,
            record_size,
            create
        );

        let instance = context.instantiate("Lorg/kwis/msp/db/DataBase;").await?.cast();
        context
            .call_method(&instance.cast(), "<init>", "()V", &[data_base_name.ptr_instance])
            .await?;

        Ok(instance)
    }

    async fn get_number_of_records(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>) -> JavaResult<i32> {
        tracing::debug!(
            "org.kwis.msp.db.DataBase::getNumberOfRecords({:#x})",
            context.instance_raw(&this.class_instance)
        );

        let db_name = context.jvm().get_field(&this.class_instance, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, db_name.as_object().unwrap())?;

        let count = context.backend().database().open(&db_name_str)?.count()?;

        Ok(count as _)
    }

    async fn close_data_base(_: &mut dyn JavaContext, this: JavaObjectProxy<DataBase>) -> JavaResult<()> {
        tracing::warn!("stub org.kwis.msp.db.DataBase::closeDataBase({:#x})", this.ptr_instance);

        Ok(())
    }

    async fn insert_record(
        context: &mut dyn JavaContext,
        this: JvmClassInstanceProxy<Self>,
        data: JavaObjectProxy<Array>,
        offset: i32,
        num_bytes: i32,
    ) -> JavaResult<i32> {
        tracing::debug!(
            "org.kwis.msp.db.DataBase::insertRecord({:#x}, {:#x}, {}, {})",
            context.instance_raw(&this.class_instance),
            data.ptr_instance,
            offset,
            num_bytes
        );

        let db_name = context.jvm().get_field(&this.class_instance, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, db_name.as_object().unwrap())?;

        let data = context.load_array_i8(&data.cast(), offset as _, num_bytes as _)?;

        let id = context.backend().database().open(&db_name_str)?.add(cast_slice(&data))?;

        Ok(id as _)
    }

    async fn select_record(context: &mut dyn JavaContext, this: JvmClassInstanceProxy<Self>, record_id: i32) -> JavaResult<JavaObjectProxy<Array>> {
        tracing::debug!(
            "org.kwis.msp.db.DataBase::selectRecord({:#x}, {})",
            context.instance_raw(&this.class_instance),
            record_id
        );

        let db_name = context.jvm().get_field(&this.class_instance, "dbName", "Ljava/lang/String;")?;
        let db_name_str = String::to_rust_string(context, db_name.as_object().unwrap())?;

        let data = context.backend().database().open(&db_name_str)?.get(record_id as _)?;

        let array = context.instantiate_array("B", data.len() as _).await?;
        context.store_array_i8(&array.cast(), 0, cast_slice(&data))?;

        Ok(array.cast())
    }
}
