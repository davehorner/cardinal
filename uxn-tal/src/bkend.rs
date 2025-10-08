use crate::AssemblerError;




pub trait AssemblerBackend {
    fn name(&self) -> &'static str;
    fn assemble(&self, tal_file: &str, tal_source: &str) -> Result<AssemblyOutput, AssemblerError>;
}

pub struct AssemblyOutput {
    pub rom_path: String,
    pub rom_bytes: Vec<u8>,
    pub stdout: String,
    pub disassembly: String,
}

impl Default for AssemblyOutput {
    fn default() -> Self {
        Self {
            rom_path: String::new(),
            rom_bytes: vec![],
            stdout: String::new(),
            disassembly: String::new(),
        }
    }
}