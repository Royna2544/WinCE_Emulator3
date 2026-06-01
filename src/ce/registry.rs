use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

pub type HKey = u32;

pub const ERROR_SUCCESS: u32 = 0;
pub const ERROR_FILE_NOT_FOUND: u32 = 2;
pub const ERROR_INVALID_HANDLE: u32 = 6;
pub const ERROR_INVALID_PARAMETER: u32 = 87;
pub const ERROR_MORE_DATA: u32 = 234;

pub const HKEY_CLASSES_ROOT: HKey = 0x8000_0000;
pub const HKEY_CURRENT_USER: HKey = 0x8000_0001;
pub const HKEY_LOCAL_MACHINE: HKey = 0x8000_0002;
pub const HKEY_USERS: HKey = 0x8000_0003;

pub const REG_BINARY: u32 = 3;
pub const REG_DWORD: u32 = 4;
pub const REG_MULTI_SZ: u32 = 7;
pub const REG_SZ: u32 = 1;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryDump {
    #[serde(default)]
    pub version: u32,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub keys: BTreeMap<String, RegistryKeyDump>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryKeyDump {
    #[serde(default)]
    pub values: BTreeMap<String, RegistryValue>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct RegistryValue {
    #[serde(rename = "type")]
    pub ty: RegistryType,
    pub data: RegistryData,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RegistryType {
    RegDword,
    RegSz,
    RegMultiSz,
    RegBinary,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum RegistryData {
    Dword(u32),
    String(String),
    MultiString(Vec<String>),
    Binary(Vec<u8>),
}

#[derive(Debug, Clone)]
pub struct Registry {
    keys: BTreeMap<String, RegistryKey>,
    open_handles: BTreeMap<HKey, String>,
    next_handle: HKey,
}

#[derive(Debug, Clone, Default)]
pub struct RegistryKey {
    values: BTreeMap<String, RegistryValue>,
}

impl Registry {
    pub fn from_dump(dump: RegistryDump) -> Self {
        let keys = dump
            .keys
            .into_iter()
            .map(|(path, key)| {
                (
                    normalize_path(&path),
                    RegistryKey {
                        values: normalize_values(key.values),
                    },
                )
            })
            .collect();

        Self {
            keys,
            open_handles: BTreeMap::new(),
            next_handle: 0x1000,
        }
    }

    pub fn reg_open_key_exw(
        &mut self,
        hkey: HKey,
        subkey: Option<&str>,
        _options: u32,
        _sam_desired: u32,
    ) -> RegOpenResult {
        let Some(path) = self.resolve_subkey_path(hkey, subkey) else {
            return RegOpenResult {
                status: ERROR_INVALID_HANDLE,
                hkey: None,
            };
        };

        if !self.keys.contains_key(&path) {
            return RegOpenResult {
                status: ERROR_FILE_NOT_FOUND,
                hkey: None,
            };
        }

        let handle = self.alloc_key_handle(path);
        RegOpenResult {
            status: ERROR_SUCCESS,
            hkey: Some(handle),
        }
    }

    pub fn reg_query_value_exw(
        &self,
        hkey: HKey,
        value_name: Option<&str>,
        data_capacity: Option<usize>,
    ) -> RegQueryValueResult {
        let Some(path) = self.resolve_handle_path(hkey) else {
            return RegQueryValueResult::status(ERROR_INVALID_HANDLE);
        };
        let value_name = normalize_value_name(value_name.unwrap_or_default());
        let Some(value) = self
            .keys
            .get(path)
            .and_then(|key| key.values.get(&value_name))
        else {
            return RegQueryValueResult::status(ERROR_FILE_NOT_FOUND);
        };

        let data = value.to_reg_bytes();
        if data_capacity.is_some_and(|capacity| capacity < data.len()) {
            return RegQueryValueResult {
                status: ERROR_MORE_DATA,
                value_type: Some(value.ty.to_win32_type()),
                required_len: data.len() as u32,
                data: None,
            };
        }

        RegQueryValueResult {
            status: ERROR_SUCCESS,
            value_type: Some(value.ty.to_win32_type()),
            required_len: data.len() as u32,
            data: data_capacity.map(|_| data),
        }
    }

    pub fn reg_set_value_exw(
        &mut self,
        hkey: HKey,
        value_name: Option<&str>,
        value_type: u32,
        data: &[u8],
    ) -> u32 {
        let Some(path) = self.resolve_handle_path(hkey).map(str::to_owned) else {
            return ERROR_INVALID_HANDLE;
        };
        let Some(value_type) = RegistryType::from_win32_type(value_type) else {
            return ERROR_INVALID_PARAMETER;
        };
        let Some(value) = RegistryValue::from_reg_bytes(value_type, data) else {
            return ERROR_INVALID_PARAMETER;
        };

        self.set_value(&path, value_name.unwrap_or_default(), value);
        ERROR_SUCCESS
    }

    pub fn reg_close_key(&mut self, hkey: HKey) -> u32 {
        if predefined_root_path(hkey).is_some() {
            return ERROR_SUCCESS;
        }
        if self.open_handles.remove(&hkey).is_some() {
            ERROR_SUCCESS
        } else {
            ERROR_INVALID_HANDLE
        }
    }

    pub fn create_key(&mut self, path: &str) {
        self.keys.entry(normalize_path(path)).or_default();
    }

    pub fn has_key(&self, path: &str) -> bool {
        self.keys.contains_key(&normalize_path(path))
    }

    pub fn query_value(&self, path: &str, name: &str) -> Result<&RegistryValue> {
        let key_path = normalize_path(path);
        let value_name = normalize_value_name(name);
        let key = self
            .keys
            .get(&key_path)
            .ok_or_else(|| Error::MissingRegistryKey(key_path.clone()))?;

        key.values
            .get(&value_name)
            .ok_or_else(|| Error::MissingRegistryValue {
                key: key_path,
                value: value_name,
            })
    }

    pub fn set_value(&mut self, path: &str, name: &str, value: RegistryValue) {
        self.keys
            .entry(normalize_path(path))
            .or_default()
            .values
            .insert(normalize_value_name(name), value);
    }

    pub fn enum_subkeys(&self, path: &str) -> Vec<String> {
        let prefix = normalize_path(path);
        let prefix_with_sep = format!("{prefix}\\");
        let mut subkeys = Vec::new();

        for key in self.keys.keys() {
            if let Some(rest) = key.strip_prefix(&prefix_with_sep) {
                if let Some(name) = rest.split('\\').next() {
                    if !subkeys.iter().any(|existing| existing == name) {
                        subkeys.push(name.to_owned());
                    }
                }
            }
        }

        subkeys
    }

    pub fn enum_values(&self, path: &str) -> Result<Vec<(String, RegistryValue)>> {
        let key_path = normalize_path(path);
        let key = self
            .keys
            .get(&key_path)
            .ok_or_else(|| Error::MissingRegistryKey(key_path.clone()))?;
        Ok(key
            .values
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect())
    }

    pub fn key_count(&self) -> usize {
        self.keys.len()
    }

    fn alloc_key_handle(&mut self, path: String) -> HKey {
        let handle = self.next_handle;
        self.next_handle += 4;
        self.open_handles.insert(handle, path);
        handle
    }

    fn resolve_subkey_path(&self, hkey: HKey, subkey: Option<&str>) -> Option<String> {
        let base = self.resolve_handle_path(hkey)?;
        let subkey = normalize_path(subkey.unwrap_or_default());
        if subkey.is_empty() {
            Some(base.to_owned())
        } else {
            Some(format!("{base}\\{subkey}"))
        }
    }

    fn resolve_handle_path(&self, hkey: HKey) -> Option<&str> {
        predefined_root_path(hkey).or_else(|| self.open_handles.get(&hkey).map(String::as_str))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegOpenResult {
    pub status: u32,
    pub hkey: Option<HKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegQueryValueResult {
    pub status: u32,
    pub value_type: Option<u32>,
    pub required_len: u32,
    pub data: Option<Vec<u8>>,
}

impl RegQueryValueResult {
    fn status(status: u32) -> Self {
        Self {
            status,
            value_type: None,
            required_len: 0,
            data: None,
        }
    }
}

impl RegistryValue {
    pub fn dword(value: u32) -> Self {
        Self {
            ty: RegistryType::RegDword,
            data: RegistryData::Dword(value),
        }
    }

    pub fn string(value: impl Into<String>) -> Self {
        Self {
            ty: RegistryType::RegSz,
            data: RegistryData::String(value.into()),
        }
    }

    pub fn as_dword(&self) -> Option<u32> {
        match self.data {
            RegistryData::Dword(value) => Some(value),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match &self.data {
            RegistryData::String(value) => Some(value),
            _ => None,
        }
    }

    fn to_reg_bytes(&self) -> Vec<u8> {
        match &self.data {
            RegistryData::Dword(value) => value.to_le_bytes().to_vec(),
            RegistryData::String(value) => encode_utf16_nul(value),
            RegistryData::MultiString(values) => {
                let mut bytes = Vec::new();
                for value in values {
                    bytes.extend(encode_utf16_nul(value));
                }
                bytes.extend(0u16.to_le_bytes());
                bytes
            }
            RegistryData::Binary(value) => value.clone(),
        }
    }

    fn from_reg_bytes(ty: RegistryType, data: &[u8]) -> Option<Self> {
        let data = match ty {
            RegistryType::RegDword => {
                let bytes: [u8; 4] = data.get(..4)?.try_into().ok()?;
                RegistryData::Dword(u32::from_le_bytes(bytes))
            }
            RegistryType::RegSz => RegistryData::String(decode_utf16_nul(data)?),
            RegistryType::RegMultiSz => RegistryData::MultiString(decode_utf16_multi_sz(data)?),
            RegistryType::RegBinary => RegistryData::Binary(data.to_vec()),
        };
        Some(Self { ty, data })
    }
}

impl RegistryType {
    fn to_win32_type(&self) -> u32 {
        match self {
            RegistryType::RegDword => REG_DWORD,
            RegistryType::RegSz => REG_SZ,
            RegistryType::RegMultiSz => REG_MULTI_SZ,
            RegistryType::RegBinary => REG_BINARY,
        }
    }

    fn from_win32_type(value: u32) -> Option<Self> {
        match value {
            REG_DWORD => Some(Self::RegDword),
            REG_SZ => Some(Self::RegSz),
            REG_MULTI_SZ => Some(Self::RegMultiSz),
            REG_BINARY => Some(Self::RegBinary),
            _ => None,
        }
    }
}

fn normalize_values(values: BTreeMap<String, RegistryValue>) -> BTreeMap<String, RegistryValue> {
    values
        .into_iter()
        .map(|(name, value)| (normalize_value_name(&name), value))
        .collect()
}

fn normalize_path(path: &str) -> String {
    path.trim_matches('\\')
        .replace('/', "\\")
        .to_ascii_lowercase()
}

fn normalize_value_name(name: &str) -> String {
    name.to_ascii_lowercase()
}

fn predefined_root_path(hkey: HKey) -> Option<&'static str> {
    match hkey {
        HKEY_CLASSES_ROOT => Some("hkcr"),
        HKEY_CURRENT_USER => Some("hkcu"),
        HKEY_LOCAL_MACHINE => Some("hklm"),
        HKEY_USERS => Some("hku"),
        _ => None,
    }
}

fn encode_utf16_nul(value: &str) -> Vec<u8> {
    value
        .encode_utf16()
        .chain(std::iter::once(0))
        .flat_map(u16::to_le_bytes)
        .collect()
}

fn decode_utf16_nul(data: &[u8]) -> Option<String> {
    let mut words = bytes_to_u16(data)?;
    if words.last() == Some(&0) {
        words.pop();
    }
    String::from_utf16(&words).ok()
}

fn decode_utf16_multi_sz(data: &[u8]) -> Option<Vec<String>> {
    let words = bytes_to_u16(data)?;
    let mut values = Vec::new();

    for part in words.split(|word| *word == 0) {
        if part.is_empty() {
            break;
        }
        values.push(String::from_utf16(part).ok()?);
    }

    Some(values)
}

fn bytes_to_u16(data: &[u8]) -> Option<Vec<u16>> {
    if data.len() % 2 != 0 {
        return None;
    }
    Some(
        data.chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn query_is_case_insensitive() {
        let mut keys = BTreeMap::new();
        keys.insert(
            "HKLM\\Ident".to_owned(),
            RegistryKeyDump {
                values: BTreeMap::from([("Name".to_owned(), RegistryValue::string("nav"))]),
            },
        );

        let registry = Registry::from_dump(RegistryDump {
            version: 1,
            source: None,
            keys,
        });

        assert_eq!(
            registry.query_value("hklm/ident", "name").unwrap().as_str(),
            Some("nav")
        );
    }

    #[test]
    fn reg_api_opens_queries_and_closes_backing_json_key() {
        let mut registry = Registry::from_dump(RegistryDump {
            version: 1,
            source: None,
            keys: BTreeMap::from([(
                "HKLM\\Ident".to_owned(),
                RegistryKeyDump {
                    values: BTreeMap::from([("Name".to_owned(), RegistryValue::string("nav"))]),
                },
            )]),
        });

        let opened = registry.reg_open_key_exw(HKEY_LOCAL_MACHINE, Some("Ident"), 0, 0);
        assert_eq!(opened.status, ERROR_SUCCESS);

        let value = registry.reg_query_value_exw(opened.hkey.unwrap(), Some("Name"), Some(64));
        assert_eq!(value.status, ERROR_SUCCESS);
        assert_eq!(value.value_type, Some(REG_SZ));
        assert_eq!(
            decode_utf16_nul(value.data.as_ref().unwrap()),
            Some("nav".to_owned())
        );

        assert_eq!(registry.reg_close_key(opened.hkey.unwrap()), ERROR_SUCCESS);
    }

    #[test]
    fn reg_query_reports_more_data_without_partial_payload() {
        let mut registry = Registry::from_dump(RegistryDump {
            version: 1,
            source: None,
            keys: BTreeMap::from([(
                "HKLM".to_owned(),
                RegistryKeyDump {
                    values: BTreeMap::from([("Name".to_owned(), RegistryValue::string("nav"))]),
                },
            )]),
        });
        let opened = registry.reg_open_key_exw(HKEY_LOCAL_MACHINE, None, 0, 0);

        let value = registry.reg_query_value_exw(opened.hkey.unwrap(), Some("name"), Some(2));

        assert_eq!(value.status, ERROR_MORE_DATA);
        assert_eq!(value.required_len, 8);
        assert!(value.data.is_none());
    }
}
