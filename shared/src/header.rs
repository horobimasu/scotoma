use std::error::Error;
use std::fs::File;
use std::io::{BufWriter, Read, Write, Result as ResultIO};
use std::path::Path;

pub struct ProcessedFileHeader {
    pub encrypted_name: Vec<u8>,
    pub encrypted_ext: Vec<u8>,
    pub encrypted_chunk_len_mib: u16,
    pub encrypted_chunk_count: u16
}

impl ProcessedFileHeader {
    pub fn new(
        encrypted_name: &[u8],
        encrypted_ext: &[u8],
        encrypted_chunk_len_mib: u16,
        encrypted_chunk_count: u16
    ) -> Result<Self, Box<dyn Error>> {
        if encrypted_name.len() > u16::MAX as usize {
            return Err("encrypted name is too long".into());
        }

        if encrypted_ext.len() > u16::MAX as usize {
            return Err("encrypted extension is too long".into());
        }

        Ok(Self {
            encrypted_name: encrypted_name.to_vec(),
            encrypted_ext: encrypted_ext.to_vec(),
            encrypted_chunk_len_mib,
            encrypted_chunk_count,
        })
    }

    pub fn header_len(&self) -> usize {
        4 +                         // header len
        2 +                         // name len
        2 +                         // ext len
        self.encrypted_name.len() + // encrypted name len
        self.encrypted_ext.len() +  // encrypted ext len
        2 +                         // chunk len
        2                           // chunk count
    }

    pub fn save_to(&self, file_path: impl AsRef<Path>) -> Result<File, Box<dyn Error>> {
        let file = File::create(file_path)
            .map_err(|err| format!("failed to save: {}", err))?;

        let mut writer = BufWriter::new(file);
        self.write_to(&mut writer)?;

        writer.flush()?;

        let the_file = writer
            .into_inner()
            .map_err(|err| err.into_error())?;

        Ok(the_file)
    }

    pub fn load_from(file_path: impl AsRef<Path>) -> Result<Self, Box<dyn Error>> {
        let mut file = File::open(file_path)
            .map_err(|err| format!("failed to load: {}", err))?;

        Self::read_from(&mut file)
    }

    pub fn write_to(&self, writer: &mut impl Write) -> Result<(), Box<dyn Error>> {
        let name_len = self.encrypted_name.len();
        if name_len > u16::MAX as usize {
            return Err("encrypted name is too long".into());
        }

        let ext_len  = self.encrypted_ext.len();
        if ext_len > u16::MAX as usize {
            return Err("encrypted extension is too long".into());
        }

        writer.write_all(&(self.header_len() as u32).to_le_bytes())?;
        writer.write_all(&(name_len as u16).to_le_bytes())?;
        writer.write_all(&(ext_len as u16).to_le_bytes())?;
        writer.write_all(&self.encrypted_name)?;
        writer.write_all(&self.encrypted_ext)?;
        writer.write_all(&self.encrypted_chunk_len_mib.to_le_bytes())?;
        writer.write_all(&self.encrypted_chunk_count.to_le_bytes())?;

        Ok(())
    }

    fn read_from(file: &mut impl Read) -> Result<Self, Box<dyn Error>> {
        let _header_len = read_u32_le(file)?;

        let name_len = read_u16_le(file)? as usize;
        let ext_len = read_u16_le(file)? as usize;

        let encrypted_name = read_vec(file, name_len)?;
        let encrypted_ext = read_vec(file, ext_len)?;

        let encrypted_chunk_len_mib = read_u16_le(file)?;
        let encrypted_chunk_count = read_u16_le(file)?;

        Ok(Self {
            encrypted_name,
            encrypted_ext,
            encrypted_chunk_len_mib,
            encrypted_chunk_count,
        })
    }
}

fn read_u16_le(reader: &mut impl Read) -> ResultIO<u16> {
    let mut buffer = [0u8; 2];
    reader.read_exact(&mut buffer)?;

    Ok(u16::from_le_bytes(buffer))
}

fn read_u32_le(reader: &mut impl Read) -> ResultIO<u32> {
    let mut buffer = [0u8; 4];
    reader.read_exact(&mut buffer)?;

    Ok(u32::from_le_bytes(buffer))
}

fn read_vec(reader: &mut impl Read, len: usize) -> ResultIO<Vec<u8>> {
    let mut buffer = vec![0u8; len];
    reader.read_exact(&mut buffer)?;

    Ok(buffer)
}
