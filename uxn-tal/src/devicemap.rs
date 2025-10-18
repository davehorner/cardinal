//! Device map parsing and representation for TAL

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceField {
    pub name: String,
    pub size: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Device {
    pub address: u16,
    pub name: String,
    pub fields: Vec<DeviceField>,
}

impl Device {
    /// Extend the fields of this DeviceMap with another set of fields
    pub fn extend_fields(&mut self, other: Vec<DeviceField>) {
        self.fields.extend(other);
    }
    /// Get the address of a field by name, using field offsets and sizes
    pub fn get_field_address(&self, field_name: &str) -> Option<u16> {
        let mut offset = 0u16;
        for field in &self.fields {
            if field.name == field_name {
                return Some(self.address + offset);
            }
            offset += field.size as u16;
        }
        None
    }
}

/// Parse a device map line, e.g.:
/// |00 @System &vector $2 &wst $1 ...
pub fn parse_device_map_line(line: &str) -> Option<Device> {
    let mut tokens = line.split_whitespace();
    let addr_token = tokens.next()?;
    if !addr_token.starts_with('|') {
        return None;
    }
    let address = u16::from_str_radix(&addr_token[1..], 16).ok()?;
    let name_token = tokens.next()?;
    let name = if let Some(stripped) = name_token.strip_prefix('@') {
        stripped.to_string()
    } else {
        return None;
    };
    let mut fields = Vec::new();
    while let Some(field_token) = tokens.next() {
        if let Some(stripped) = field_token.strip_prefix('&') {
            let field_name = stripped.to_string();
            if let Some(size_token) = tokens.next() {
                if let Some(stripped) = size_token.strip_prefix('$') {
                    if let Ok(size) = u8::from_str_radix(stripped, 16) {
                        fields.push(DeviceField {
                            name: field_name,
                            size,
                        });
                    }
                }
            }
        }
    }
    Some(Device {
        address,
        name,
        fields,
    })
}

/// Parse multiple device map lines from a TAL file
pub fn parse_device_maps(source: &str) -> Vec<Device> {
    source.lines().filter_map(parse_device_map_line).collect()
}

use once_cell::sync::Lazy;

pub static DEVICES_DEFAULT: Lazy<Vec<Device>> = Lazy::new(|| {
    vec![
        Device {
            address: 0x00,
            name: "System".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "wst".into(),
                    size: 1,
                },
                DeviceField {
                    name: "rst".into(),
                    size: 1,
                },
                DeviceField {
                    name: "eaddr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "ecode".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pad".into(),
                    size: 1,
                },
                DeviceField {
                    name: "r".into(),
                    size: 2,
                },
                DeviceField {
                    name: "g".into(),
                    size: 2,
                },
                DeviceField {
                    name: "b".into(),
                    size: 2,
                },
                DeviceField {
                    name: "debug".into(),
                    size: 1,
                },
                DeviceField {
                    name: "halt".into(),
                    size: 1,
                },
                DeviceField {
                    name: "state".into(),
                    size: 1,
                },
            ],
        },
        Device {
            address: 0x10,
            name: "Console".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "read".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pad".into(),
                    size: 5,
                },
                DeviceField {
                    name: "write".into(),
                    size: 1,
                },
                DeviceField {
                    name: "error".into(),
                    size: 1,
                },
            ],
        },
        Device {
            address: 0x20,
            name: "Screen".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "width".into(),
                    size: 2,
                },
                DeviceField {
                    name: "height".into(),
                    size: 2,
                },
                DeviceField {
                    name: "auto".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pad".into(),
                    size: 1,
                },
                DeviceField {
                    name: "x".into(),
                    size: 2,
                },
                DeviceField {
                    name: "y".into(),
                    size: 2,
                },
                DeviceField {
                    name: "addr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "pixel".into(),
                    size: 1,
                },
                DeviceField {
                    name: "sprite".into(),
                    size: 1,
                },
            ],
        },
        Device {
            address: 0x30,
            name: "Audio0".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "position".into(),
                    size: 2,
                },
                DeviceField {
                    name: "output".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pad".into(),
                    size: 3,
                },
                DeviceField {
                    name: "adsr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "length".into(),
                    size: 2,
                },
                DeviceField {
                    name: "addr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "volume".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pitch".into(),
                    size: 1,
                },
            ],
        },
        Device {
            address: 0x40,
            name: "Audio1".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "position".into(),
                    size: 2,
                },
                DeviceField {
                    name: "output".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pad".into(),
                    size: 3,
                },
                DeviceField {
                    name: "adsr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "length".into(),
                    size: 2,
                },
                DeviceField {
                    name: "addr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "volume".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pitch".into(),
                    size: 1,
                },
            ],
        },
        Device {
            address: 0x50,
            name: "Audio2".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "position".into(),
                    size: 2,
                },
                DeviceField {
                    name: "output".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pad".into(),
                    size: 3,
                },
                DeviceField {
                    name: "adsr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "length".into(),
                    size: 2,
                },
                DeviceField {
                    name: "addr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "volume".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pitch".into(),
                    size: 1,
                },
            ],
        },
        Device {
            address: 0x60,
            name: "Audio3".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "position".into(),
                    size: 2,
                },
                DeviceField {
                    name: "output".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pad".into(),
                    size: 3,
                },
                DeviceField {
                    name: "adsr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "length".into(),
                    size: 2,
                },
                DeviceField {
                    name: "addr".into(),
                    size: 2,
                },
                DeviceField {
                    name: "volume".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pitch".into(),
                    size: 1,
                },
            ],
        },
        Device {
            address: 0x80,
            name: "Controller".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "button".into(),
                    size: 1,
                },
                DeviceField {
                    name: "key".into(),
                    size: 1,
                },
                DeviceField {
                    name: "func".into(),
                    size: 1,
                },
            ],
        },
        Device {
            address: 0x90,
            name: "Mouse".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "x".into(),
                    size: 2,
                },
                DeviceField {
                    name: "y".into(),
                    size: 2,
                },
                DeviceField {
                    name: "state".into(),
                    size: 1,
                },
                DeviceField {
                    name: "pad".into(),
                    size: 3,
                },
                DeviceField {
                    name: "scrollx".into(),
                    size: 2,
                },
                DeviceField {
                    name: "scrolly".into(),
                    size: 2,
                },
            ],
        },
        Device {
            address: 0xa0,
            name: "File".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "success".into(),
                    size: 2,
                },
                DeviceField {
                    name: "stat".into(),
                    size: 2,
                },
                DeviceField {
                    name: "delete".into(),
                    size: 1,
                },
                DeviceField {
                    name: "append".into(),
                    size: 1,
                },
                DeviceField {
                    name: "name".into(),
                    size: 2,
                },
                DeviceField {
                    name: "length".into(),
                    size: 2,
                },
                DeviceField {
                    name: "read".into(),
                    size: 2,
                },
                DeviceField {
                    name: "write".into(),
                    size: 2,
                },
            ],
        },
        Device {
            address: 0xb0,
            name: "File1".into(),
            fields: vec![
                DeviceField {
                    name: "vector".into(),
                    size: 2,
                },
                DeviceField {
                    name: "success".into(),
                    size: 2,
                },
                DeviceField {
                    name: "stat".into(),
                    size: 2,
                },
                DeviceField {
                    name: "delete".into(),
                    size: 1,
                },
                DeviceField {
                    name: "append".into(),
                    size: 1,
                },
                DeviceField {
                    name: "name".into(),
                    size: 2,
                },
                DeviceField {
                    name: "length".into(),
                    size: 2,
                },
                DeviceField {
                    name: "read".into(),
                    size: 2,
                },
                DeviceField {
                    name: "write".into(),
                    size: 2,
                },
            ],
        },
        Device {
            address: 0xc0,
            name: "DateTime".into(),
            fields: vec![
                DeviceField {
                    name: "year".into(),
                    size: 2,
                },
                DeviceField {
                    name: "month".into(),
                    size: 1,
                },
                DeviceField {
                    name: "day".into(),
                    size: 1,
                },
                DeviceField {
                    name: "hour".into(),
                    size: 1,
                },
                DeviceField {
                    name: "minute".into(),
                    size: 1,
                },
                DeviceField {
                    name: "second".into(),
                    size: 1,
                },
                DeviceField {
                    name: "dotw".into(),
                    size: 1,
                },
                DeviceField {
                    name: "doty".into(),
                    size: 2,
                },
                DeviceField {
                    name: "isdst".into(),
                    size: 1,
                },
            ],
        },
    ]
});
