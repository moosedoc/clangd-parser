use crate::clangd::ClangdUtility;
use griff::ChunkStream;

#[derive(Debug, Clone, Default, PartialEq)]
#[repr(u8)]
pub enum SymbolKind {
    #[default]
    Unknown = 0,

    Module,
    Namespace,
    NamespaceAlias,
    Macro,
   
    Enum,
    Struct,
    Class,
    Protocol,
    Extension,
    Union,
    TypeAlias,
   
    Function,
    Variable,
    Field,
    EnumConstant,
   
    InstanceMethod,
    ClassMethod,
    StaticMethod,
    InstanceProperty,
    ClassProperty,
    StaticProperty,
   
    Constructor,
    Destructor,
    ConversionFunction,
   
    Parameter,
    Using,
    TemplateTypeParm,
    TemplateTemplateParm,
    NonTypeTemplateParm,
}
impl From<u8> for SymbolKind {
    fn from(b: u8) -> Self {
        use SymbolKind::*;
        match b {
            1 => Module,
            2 => Namespace,
            3 => NamespaceAlias,
            4 => Macro,
            5 => Enum,
            6 => Struct,
            7 => Class,
            8 => Protocol,
            9 => Extension,
            10 => Union,
            11 => TypeAlias,
            12 => Function,
            13 => Variable,
            14 => Field,
            15 => EnumConstant,
            16 => InstanceMethod,
            17 => ClassMethod,
            18 => StaticMethod,
            19 => InstanceProperty,
            20 => ClassProperty,
            21 => StaticProperty,
            22 => Constructor,
            23 => Destructor,
            24 => ConversionFunction,
            25 => Parameter,
            26 => Using,
            27 => TemplateTypeParm,
            28 => TemplateTemplateParm,
            29 => NonTypeTemplateParm,

            _ => Unknown,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[repr(u8)]
pub enum SymbolLanguage {
    #[default]
    C,

    ObjC,
    CXX,
    Swift,
}
impl From<u8> for SymbolLanguage {
    fn from(b: u8) -> Self {
        use SymbolLanguage::*;

        match b {
            1 => ObjC,
            2 => CXX,
            3 => Swift,
            _ => C,
        }
    }
}

#[derive(Debug, Clone, Default)]
#[repr(u8)]
#[allow(dead_code)]
pub enum SymbolSubKind {
    #[default]
    None,
    CXXCopyConstructor,
    CXXMoveConstructor,
    AccessorGetter,
    AccessorSetter,
    UsingTypename,
    UsingValue,
    UsingEnum,
}

#[derive(Debug, Clone, Default)]
#[repr(u16)]
#[allow(dead_code)]
pub enum SymbolProperty {
    #[default]
    Unknown,

    Generic                       = 1 << 0,
    TemplatePartialSpecialization = 1 << 1,
    TemplateSpecialization        = 1 << 2,
    UnitTest                      = 1 << 3,
    IBAnnotated                   = 1 << 4,
    IBOutletCollection            = 1 << 5,
    GKInspectable                 = 1 << 6,
    Local                         = 1 << 7,
}
pub type SymbolPropertySet = u16;

#[derive(Debug, Clone, Default)]
pub struct SymbolInfo {
    pub kind: SymbolKind,
    pub subkind: SymbolSubKind,
    pub lang: SymbolLanguage,
    pub properties: SymbolPropertySet,
}

#[derive(Debug, Clone, Default)]
pub struct SymbolLocation {
    pub start: SymbolPosition,
    pub end: SymbolPosition,
    pub file_uri: String,
}
impl SymbolLocation {
    pub fn get_location(buf: &[u8], string_table: &Vec<String>) -> (usize, Self) {
        let mut loc: SymbolLocation = Default::default();
        let mut bytes_read: usize = 0;
        let (sz, content) = Symbol::get_string(buf, string_table);
        loc.file_uri = content;
        bytes_read += sz;

        let (sz, content) = Symbol::get_varint(buf.get(bytes_read..).unwrap());
        loc.start.line = content;
        bytes_read += sz;
        let (sz, content) = Symbol::get_varint(buf.get(bytes_read..).unwrap());
        loc.start.column = content;
        bytes_read += sz;

        let (sz, content) = Symbol::get_varint(buf.get(bytes_read..).unwrap());
        loc.end.line = content;
        bytes_read += sz;
        let (sz, content) = Symbol::get_varint(buf.get(bytes_read..).unwrap());
        loc.end.column = content;
        bytes_read += sz;

        (bytes_read, loc)
    }
}

#[derive(Debug, Clone, Default)]
pub struct SymbolPosition {
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, Default)]
#[repr(u16)]
#[allow(dead_code)]
pub enum SymbolOrigin {
    #[default]
    Unknown = 0,

    AST = 1 << 0,
    Open = 1 << 1,
    Static = 1 << 2,
    Merge = 1 << 3,
    Identifier = 1 << 4,
    Remote = 1 << 5,
    Preamble = 1 << 6,
    Background = 1 << 8,
    StdLib = 1 << 9,
}

#[derive(Debug, Clone, Default)]
#[repr(u8)]
pub enum SymbolFlags {
    #[default]
    None,

    IndexedForCodeCompletion = 1 << 0,
    Deprecated = 1 << 1,
    ImplementationDetail = 1 << 2,
    VisibleOutsideFile = 1 << 3,
}
impl From<u8> for SymbolFlags {
    fn from(b: u8) -> Self {
        use SymbolFlags::*;

        match b {
            1 => IndexedForCodeCompletion,
            2 => Deprecated,
            4 => ImplementationDetail,
            8 => VisibleOutsideFile,

            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SymbolIncludedHeader {
    pub name: String,
    pub refs: usize,
    pub supported_directives: usize,
}

pub type SymbolId = [u8; 8];

#[derive(Debug, Clone, Default)]
pub struct Symbol {
    pub id: SymbolId,
    pub syminfo: SymbolInfo,
    pub name: String,
    pub scope: String,
    pub definition: SymbolLocation,
    pub canonical_declaration: SymbolLocation,
    pub references: u32,
    pub flags: SymbolFlags,
    pub origin: SymbolOrigin,
    pub signature: String,
    pub template_specialization_args: String,
    pub completion_snippet_suffix: String,
    pub documentation: String,
    pub return_t: String,
    pub t: String,
    pub headers: Vec<SymbolIncludedHeader>,
}
impl ClangdUtility for Symbol {}

impl Symbol {
    pub fn parse(stream: &ChunkStream, string_table: &Vec<String>) -> Vec<Symbol> {
        let mut syms: Vec<Symbol> = vec![];
        let len = stream.data.len();
        let _data = stream.data.clone();
        let data = _data.as_slice();
        if data.len() == 0 {
            return syms;
        }
        let mut cursor: usize = 0;
        let mut idx: u32;
        loop {
            let mut s: Symbol = Default::default();
            s.id = data.get(cursor..cursor+8).unwrap().try_into().unwrap();
            cursor += 8;
            // KIND
            s.syminfo.kind = SymbolKind::from(data[cursor]);
            cursor += 1;
            // LANGUAGE
            s.syminfo.lang = SymbolLanguage::from(data[cursor]);
            cursor += 1;
            let (sz, content) = Self::get_string(&data.get(cursor..).unwrap(), string_table);
            s.name = content;
            cursor += sz;
            // SCOPE
            let (sz, content) = Self::get_string(&data.get(cursor..).unwrap(), string_table);
            s.scope = content;
            cursor += sz;
            // TEMPLATE SPECIALIZATION ARGUMENTS
            let (sz, content) = Self::get_string(&data.get(cursor..).unwrap(), string_table);
            s.template_specialization_args = content;
            cursor += sz;
            // LOCATION
            let (sz, loc) = SymbolLocation::get_location(&data.get(cursor..).unwrap(), string_table);
            s.definition = loc;
            cursor += sz;
            // CANONICAL DECLARATION
            let (sz, loc) = SymbolLocation::get_location(&data.get(cursor..).unwrap(), string_table);
            s.canonical_declaration = loc;
            cursor += sz;
            // REFERENCES
            let (sz, content) = Self::get_varint(&data.get(cursor..).unwrap());
            s.references = content;
            cursor += sz;
            // FLAGS
            s.flags = SymbolFlags::from(*data.get(cursor).unwrap());
            cursor += 1;
            // SIGNATURE
            let (sz, content) = Self::get_string(&data.get(cursor..).unwrap(), string_table);
            s.signature = content;
            cursor += sz;
            // COMPLETION SNIPPET SUFFIX
            let (sz, content) = Self::get_string(&data.get(cursor..).unwrap(), string_table);
            s.completion_snippet_suffix = content;
            cursor += sz;
            // DOCUMENTATION
            let (sz, content) = Self::get_string(&data.get(cursor..).unwrap(), string_table);
            s.documentation = content;
            cursor += sz;
            // RETURN TYPE
            let (sz, content) = Self::get_string(&data.get(cursor..).unwrap(), string_table);
            s.return_t = content;
            cursor += sz;
            // TYPE
            let (sz, content) = Self::get_string(&data.get(cursor..).unwrap(), string_table);
            s.t = content;
            cursor += sz;
            // INCLUDES
            let (sz, h_content) = Self::get_varint(&data.get(cursor..).unwrap());
            cursor += sz;
            idx = 0;
            while idx < h_content {
                let mut hdr: SymbolIncludedHeader = Default::default();
                let (sz, content) = Self::get_string(&data.get(cursor..).unwrap(), string_table);
                hdr.name = content;
                cursor += sz;
                let (sz, content) = Self::get_varint(&data.get(cursor..).unwrap());
                hdr.refs = (content >> 2) as usize;
                hdr.supported_directives = (content & 0x3) as usize;
                s.headers.push(hdr);
                cursor += sz;
                idx += 1;
            }
            syms.push(s.clone());
            if cursor >= len {
                break;
            }
        }
        syms
    }
}
