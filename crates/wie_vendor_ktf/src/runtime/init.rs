use alloc::string::String;
use core::mem::size_of;

use bytemuck::{Pod, Zeroable};

use wie_backend::Backend;
use wie_base::util::{read_generic, write_generic};
use wie_core_arm::{Allocator, ArmCore, PEB_BASE};

use crate::runtime::{
    c::interface::get_wipic_knl_interface,
    java::context::KtfJavaContext,
    java::interface::{get_wipi_jb_interface, java_array_new, java_class_load, java_new, java_throw},
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam0 {
    unk: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam4 {
    fn_get_interface: u32,
    fn_java_throw: u32,
    unk1: u32,
    unk2: u32,
    unk3: u32,
    fn_java_new: u32,
    fn_java_array_new: u32,
    unk6: u32,
    fn_java_class_load: u32,
    unk7: u32,
    unk8: u32,
    fn_alloc: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam1 {
    ptr_unk_struct: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam1Unk {
    unk: [u32; 32],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam2 {
    unk1: u32,
    unk2: u32,
    unk3: u32,
    ptr_vtables: [u32; 64],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct InitParam3 {
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    // java array allocation pool for primitive type
    boolean: u32,
    char: u32,
    float: u32,
    double: u32,
    byte: u32,
    short: u32,
    int: u32,
    long: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct WipiExe {
    ptr_exe_interface: u32,
    ptr_name: u32,
    unk1: u32,
    unk2: u32,
    fn_unk1: u32,
    fn_init: u32,
    unk3: u32,
    unk4: u32,
    fn_unk3: u32,
    unk5: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ExeInterface {
    ptr_functions: u32,
    ptr_name: u32,
    unk1: u32,
    unk2: u32,
    unk3: u32,
    unk4: u32,
    unk5: u32,
    unk6: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ExeInterfaceFunctions {
    unk1: u32,
    unk2: u32,
    fn_init: u32,
    fn_get_default_dll: u32,
    fn_get_class: u32,
    fn_unk2: u32,
    fn_unk3: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct KtfPeb {
    pub ptr_java_context_data: u32,
}

pub struct ModuleInfo {
    pub fn_init: u32,
    pub fn_get_class: u32,
}

pub async fn init(core: &mut ArmCore, backend: &Backend, base_address: u32, bss_size: u32) -> anyhow::Result<ModuleInfo> {
    let wipi_exe = core.run_function(base_address + 1, &[bss_size]).await?;

    log::debug!("Got wipi_exe {:#x}", wipi_exe);

    let ptr_param_0 = Allocator::alloc(core, size_of::<InitParam0>() as u32)?;
    write_generic(core, ptr_param_0, InitParam0 { unk: 0 })?;

    let ptr_unk_struct = Allocator::alloc(core, size_of::<InitParam1Unk>() as u32)?;
    write_generic(core, ptr_unk_struct, InitParam1Unk { unk: [0; 32] })?;

    let ptr_param_1 = Allocator::alloc(core, size_of::<InitParam1>() as u32)?;
    write_generic(core, ptr_param_1, InitParam1 { ptr_unk_struct })?;

    let ptr_param_2 = Allocator::alloc(core, (size_of::<InitParam2>()) as u32)?;
    write_generic(
        core,
        ptr_param_2,
        InitParam2 {
            unk1: 0,
            unk2: 0,
            unk3: 0,
            ptr_vtables: [0; 64],
        },
    )?;

    let param_3 = InitParam3 {
        unk1: 0,
        unk2: 0,
        unk3: 0,
        unk4: 0,
        boolean: b'Z' as u32,
        char: b'C' as u32,
        float: b'F' as u32,
        double: b'D' as u32,
        byte: b'B' as u32,
        short: b'S' as u32,
        int: b'I' as u32,
        long: b'J' as u32,
    };

    let ptr_param_3 = Allocator::alloc(core, size_of::<InitParam3>() as u32)?;
    write_generic(core, ptr_param_3, param_3)?;

    let param_4 = InitParam4 {
        fn_get_interface: core.register_function(get_interface, backend)?,
        fn_java_throw: core.register_function(java_throw, backend)?,
        unk1: 0,
        unk2: 0,
        unk3: 0,
        fn_java_new: core.register_function(java_new, backend)?,
        fn_java_array_new: core.register_function(java_array_new, backend)?,
        unk6: 0,
        fn_java_class_load: core.register_function(java_class_load, backend)?,
        unk7: 0,
        unk8: 0,
        fn_alloc: core.register_function(alloc, backend)?,
    };

    let ptr_param_4 = Allocator::alloc(core, size_of::<InitParam4>() as u32)?;
    write_generic(core, ptr_param_4, param_4)?;

    let ptr_java_context_data = KtfJavaContext::init(core, ptr_param_2)?;
    init_peb(core, KtfPeb { ptr_java_context_data })?;

    let wipi_exe: WipiExe = read_generic(core, wipi_exe)?;
    let exe_interface: ExeInterface = read_generic(core, wipi_exe.ptr_exe_interface)?;
    let exe_interface_functions: ExeInterfaceFunctions = read_generic(core, exe_interface.ptr_functions)?;

    log::debug!("Call init at {:#x}", exe_interface_functions.fn_init);
    let result = core
        .run_function::<u32>(
            exe_interface_functions.fn_init,
            &[ptr_param_0, ptr_param_1, ptr_param_2, ptr_param_3, ptr_param_4],
        )
        .await?;
    if result != 0 {
        return Err(anyhow::anyhow!("Init failed with code {:#x}", result));
    }

    Ok(ModuleInfo {
        fn_init: wipi_exe.fn_init,
        fn_get_class: exe_interface_functions.fn_get_class,
    })
}

async fn get_interface(core: &mut ArmCore, backend: &mut Backend, r#struct: String) -> anyhow::Result<u32> {
    log::trace!("get_interface({})", r#struct);

    match r#struct.as_str() {
        "WIPIC_knlInterface" => get_wipic_knl_interface(core, backend),
        "WIPI_JBInterface" => get_wipi_jb_interface(core, backend),
        _ => {
            log::warn!("Unknown {}", r#struct);

            Ok(0)
        }
    }
}

async fn alloc(core: &mut ArmCore, _: &mut Backend, a0: u32) -> anyhow::Result<u32> {
    log::trace!("alloc({})", a0);

    Allocator::alloc(core, a0)
}

fn init_peb(core: &mut ArmCore, peb: KtfPeb) -> anyhow::Result<()> {
    core.map(PEB_BASE, 0x1000)?;
    write_generic(core, PEB_BASE, peb)?;

    Ok(())
}
