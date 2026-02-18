use std::{
    fs::{File, Permissions},
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
};

use apple_codesign::{MachOSigner, SettingsScope, SigningSettings};
use bytemuck::bytes_of;
use mach_o::{Header, LoadCommand};

use crate::synthesize::arch::MachineCode;

use super::{
    Executable,
    mac::mach_o::{
        DyLinkerCommand, DySymTabCommand, EntryPointCommand, HeaderFlags, LinkEditDataCommand,
        MemoryPermissions, SectionHeader, SegmentCommand, SymTabCommand,
    },
};

mod mach_o;

#[derive(Default)]
pub struct AppleExecutable {
    binary_identifier: Option<String>,
}

impl Executable for AppleExecutable {
    fn build(&self, code: MachineCode, out_path: impl AsRef<Path>) {
        // Mach-O file:
        // Header
        // LC_SEGMENT (__PAGEZERO)
        // LC_SEGMENT (__TEXT)
        // __text section header
        // LC_MAIN
        // LC_LOAD_DYLINKER
        // LC_SEGMENT_64 (__LINKEDIT)
        // LC_CODE_SIGNATURE
        // LC_DYSYMTAB
        // LC_SYMTAB
        // __text section (code)
        // code signature

        let MachineCode {
            instructions,
            entry_point_offset,
        } = code;

        let pagezero_segment = SegmentCommand {
            command: LoadCommand::Segment,
            command_size: size_of::<SegmentCommand>() as u32,
            segment_name: b"__PAGEZERO\0\0\0\0\0\0".to_owned(),
            vmaddr: 0x0,         // Located at 0x0 to catch null pointers
            vmsize: 0x100000000, // u32::MAX + 1 to block lower 32-bit address space
            file_offset: 0x0,
            file_size: 0x0,
            max_prot: MemoryPermissions::empty(),
            init_prot: MemoryPermissions::empty(),
            section_count: 0,
            flags: 0,
        };

        let text_segment_size = (size_of::<SegmentCommand>() + size_of::<SectionHeader>()) as u32;
        let mut text_segment = SegmentCommand {
            command: LoadCommand::Segment,
            command_size: text_segment_size,
            segment_name: b"__TEXT\0\0\0\0\0\0\0\0\0\0".to_owned(),
            vmaddr: pagezero_segment.vmsize,
            vmsize: 0, // filled in later
            file_offset: 0x0,
            file_size: 0, // filled in later
            max_prot: MemoryPermissions::ReadExecute,
            init_prot: MemoryPermissions::ReadExecute,
            section_count: 1,
            flags: 0,
        };

        let mut text_section_header = SectionHeader {
            section_name: b"__text\0\0\0\0\0\0\0\0\0\0".to_owned(),
            segment_name: b"__TEXT\0\0\0\0\0\0\0\0\0\0".to_owned(),
            addr: 0x0, // filled in later
            size: instructions.len() as u64,
            offset: 0x0, // filled in later
            align: 0x2,
            reloff: 0,
            nreloc: 0,
            flags: 0,
            _reserved1: 0,
            _reserved2: 0,
            _reserved3: 0,
        };

        let mut entry_point = EntryPointCommand {
            command: LoadCommand::EntryPoint,
            command_size: size_of::<EntryPointCommand>() as u32,
            main_offset: entry_point_offset,
            stack_size: 0,
        };

        let linker_path = b"/usr/lib/dyld";
        let dylinker_cmd_size = align(size_of::<DyLinkerCommand>() + linker_path.len(), 8);
        let path_len_with_padding = dylinker_cmd_size - size_of::<DyLinkerCommand>();
        let mut padded_linker_path = vec![0u8; path_len_with_padding];
        padded_linker_path[..linker_path.len()].copy_from_slice(linker_path);

        let dylinker = DyLinkerCommand {
            command: LoadCommand::LoadDyLinker,
            command_size: dylinker_cmd_size as u32,
            path_str_offset: size_of::<DyLinkerCommand>() as u32,
        };

        let text_data_offset = (size_of::<Header>()
            + size_of_val(&pagezero_segment)
            + size_of_val(&text_segment)
            + size_of_val(&text_section_header)
            + size_of_val(&entry_point)
            + dylinker.command_size as usize
            + size_of::<SegmentCommand>()
            + size_of::<LinkEditDataCommand>()
            + size_of::<DySymTabCommand>()
            + size_of::<SymTabCommand>()) as u32;

        text_section_header.offset = text_data_offset;
        text_section_header.addr = text_segment.vmaddr;
        entry_point.main_offset += text_data_offset as u64;

        let text_section_end =
            page_align(text_section_header.offset as u64 + text_section_header.size);
        text_segment.file_size = text_section_end;
        text_segment.vmsize = text_section_end;

        let text_seg_padding = text_section_end as usize
            - text_section_header.offset as usize
            - text_section_header.size as usize;

        let mut linkedit_segment = SegmentCommand {
            command: LoadCommand::Segment,
            command_size: size_of::<SegmentCommand>() as u32,
            segment_name: b"__LINKEDIT\0\0\0\0\0\0".to_owned(),
            vmaddr: text_segment.vmaddr + text_segment.vmsize,
            vmsize: 0x4000,
            file_offset: 0, // filled in later
            file_size: 0,   // filled in later
            max_prot: MemoryPermissions::empty(),
            init_prot: MemoryPermissions::empty(),
            section_count: 0,
            flags: 0,
        };

        let mut code_sig_cmd = LinkEditDataCommand {
            command: LoadCommand::CodeSignature,
            command_size: size_of::<LinkEditDataCommand>() as u32,
            data_offset: 0, // filled in later
            data_size: 0,   // filled in later
        };

        let dysymtab = DySymTabCommand {
            command: LoadCommand::DySymTab,
            command_size: size_of::<DySymTabCommand>() as u32,
            ilocalsym: 0,
            nlocalsym: 0,
            iextdefsym: 0,
            nextdefsym: 0,
            iundefsym: 0,
            nundefsym: 0,
            tocoff: 0,
            ntoc: 0,
            modtaboff: 0,
            nmodtab: 0,
            extrefsymoff: 0,
            nextrefsyms: 0,
            indirectsymoff: 0,
            nindirectsyms: 0,
            extreloff: 0,
            nextrel: 0,
            locreloff: 0,
            nlocrel: 0,
        };

        let symtab = SymTabCommand {
            command: LoadCommand::SymTab,
            command_size: size_of::<SymTabCommand>() as u32,
            symoff: 0,
            nsyms: 0,
            stroff: 0,
            strsize: 0,
        };

        let header = Header {
            magic: mach_o::Magic::X64,
            cpu_type: mach_o::CpuType::Arm64,
            cpu_subtype: mach_o::CpuSubtype::Arm,
            file_type: mach_o::FileType::Execute,
            load_cmd_count: 8,
            load_cmd_size: pagezero_segment.command_size
                + text_segment.command_size
                + entry_point.command_size
                + dylinker.command_size
                + linkedit_segment.command_size
                + code_sig_cmd.command_size
                + dysymtab.command_size
                + symtab.command_size,
            flags: HeaderFlags::PIE | HeaderFlags::DyldLink,
            _reserved: 0,
        };

        let mut codesign = [0u8; 16];
        let superblob_len = 12u32;
        let superblob_count = 0u32;
        codesign[0..4].copy_from_slice(&mach_o::CSMAGIC_EMBEDDED_SIGNATURE.to_le_bytes());
        codesign[4..8].copy_from_slice(&superblob_len.to_le_bytes());
        codesign[8..12].copy_from_slice(&superblob_count.to_le_bytes());

        linkedit_segment.file_offset = text_section_end;
        linkedit_segment.file_size = codesign.len() as u64;
        code_sig_cmd.data_offset = linkedit_segment.file_offset as u32;
        code_sig_cmd.data_size = codesign.len() as u32;

        let mut vec: Vec<u8> = Vec::new();
        vec.extend(bytes_of(&header));
        vec.extend(bytes_of(&pagezero_segment));
        vec.extend(bytes_of(&text_segment));
        vec.extend(bytes_of(&text_section_header));
        vec.extend(bytes_of(&entry_point));
        vec.extend(bytes_of(&dylinker));
        vec.extend(&padded_linker_path);
        vec.extend(bytes_of(&linkedit_segment));
        vec.extend(bytes_of(&code_sig_cmd));
        vec.extend(bytes_of(&dysymtab));
        vec.extend(bytes_of(&symtab));
        vec.extend(instructions);
        vec.extend(&vec![0u8; text_seg_padding]);
        vec.extend(&codesign);

        let mut file = File::create(&out_path).unwrap();

        let signer = MachOSigner::new(&vec).unwrap();
        let mut sign_settings = SigningSettings::default();
        sign_settings.set_binary_identifier(
            SettingsScope::Main,
            self.binary_identifier
                .as_ref()
                .expect("apple executables require a binary identifier"),
        );
        signer
            .write_signed_binary(&sign_settings, &mut file)
            .unwrap();

        std::fs::set_permissions(out_path, Permissions::from_mode(0o755)).unwrap();
    }

    fn with_binary_identifier(mut self, ident: String) -> Self {
        self.binary_identifier = Some(ident);
        self
    }
}

fn page_align(addr: u64) -> u64 {
    const PAGE_ALIGN: u64 = 0x4000;
    align(addr, PAGE_ALIGN)
}

fn align<
    N: std::ops::Add<Output = N>
        + std::ops::Sub<Output = N>
        + std::ops::Rem<Output = N>
        + PartialEq<N>
        + From<u8>
        + Copy,
>(
    num: N,
    alignment: N,
) -> N {
    let overshoot = num % alignment;
    if overshoot == N::from(0u8) {
        num
    } else {
        num + (alignment - overshoot)
    }
}
