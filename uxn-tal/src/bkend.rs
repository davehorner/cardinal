use crate::AssemblerError;

pub trait AssemblerBackend {
    fn name(&self) -> &'static str;
    fn assemble(&self, tal_file: &str, tal_source: &str) -> Result<AssemblyOutput, AssemblerError>;
}

#[derive(Default)]
pub struct AssemblyOutput {
    pub rom_path: String,
    pub rom_bytes: Vec<u8>,
    pub stdout: String,
    pub disassembly: String,
}
