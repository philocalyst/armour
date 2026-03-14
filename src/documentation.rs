// Primitive / shared building blocks

/// A resolved or unresolved reference used in `@see`, `@{ref}`, and backtick refs.
#[derive(Debug, Clone, PartialEq)]
pub struct Ref {
    /// The raw reference string, e.g. `"pl.pretty.write"` or `"myclass:method"`.
    pub target: String,
    /// Optional display text override from `@{ref|text}` syntax.
    pub display: Option<String>,
}

/// A doc-comment source location.
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub file: String,
    pub line: usize,
}

// Tag modifiers  (e.g. `[opt]`, `[optchain]`, `[type=number]`, `[1]`, `[2]`)

/// Every modifier that may appear inside `[…]` on a tag.
#[derive(Debug, Clone, PartialEq)]
pub enum TagModifier {
    /// `[opt]` — the parameter is optional.
    Opt { default: Option<String> },

    /// `[optchain]` — continues an optional chain started by `[opt]`.
    OptChain { default: Option<String> },

    /// `[type=<expr>]` — explicit type expression attached to the tag.
    Type(TypeExpr),

    /// `[<digit>]` — return group number, e.g. `@return[1]` / `@return[2]`.
    ReturnGroup(u8),

    /// Any unrecognised `[key=value]` modifier preserved for custom tooling.
    Custom { key: String, value: String },
}

// Type expressions  (used by tparam / treturn and type modifiers)

/// The structured form of an LDoc type expression.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    /// A plain built-in or user-defined name, e.g. `number`, `string`, `Bonzo`.
    Named(String),

    /// `?T` — the type or nil; shorthand for `?|nil|T`.
    Nullable(Box<TypeExpr>),

    /// `?|T1|T2|…` — union of types.
    Union(Vec<TypeExpr>),

    /// `{T1, T2}` — positional tuple.
    Tuple(Vec<TypeExpr>),

    /// `{A=T1, B=T2}` — named struct.
    Struct(Vec<(String, TypeExpr)>),

    /// `{T, ...}` — array of T.
    Array(Box<TypeExpr>),

    /// `{[K]=V, ...}` — map from K to V.
    Map {
        key: Box<TypeExpr>,
        value: Box<TypeExpr>,
    },

    /// `Container(T)` — generic / parametric type, e.g. `Array(Bonzo)`.
    Generic { name: String, args: Vec<TypeExpr> },
}

// @param / @tparam

/// A single `@param` (or `@tparam`) annotation.
#[derive(Debug, Clone)]
pub struct Param {
    /// Formal parameter name.
    pub name: String,
    /// Free-text description.
    pub description: Option<String>,
    /// Type expression (from `@tparam` or `[type=…]`).
    pub param_type: Option<TypeExpr>,
    /// All modifiers attached to this tag.
    pub modifiers: Vec<TagModifier>,
}

// @return / @treturn / @error

/// A single return-value annotation.
#[derive(Debug, Clone)]
pub struct Return {
    /// Free-text description.
    pub description: Option<String>,
    /// Type expression (from `@treturn` or `[type=…]`).
    pub return_type: Option<TypeExpr>,
    /// Return group digit (from `@return[1]`, `@return[2]`, …).
    pub group: Option<u8>,
}

/// `@error` — shorthand for a `(nil, error_message)` return group.
#[derive(Debug, Clone)]
pub struct ErrorReturn {
    pub description: Option<String>,
}

// @raise

/// `@raise` — documents an unhandled error thrown by the function.
#[derive(Debug, Clone)]
pub struct Raise {
    pub description: String,
}

// @see / @usage

/// `@see` — a cross-reference to another documented item or a custom URL.
#[derive(Debug, Clone)]
pub struct See {
    pub reference: Ref,
}

/// `@usage` — a usage example attached to a function or module.
#[derive(Debug, Clone)]
pub struct Usage {
    /// Raw source code / prose for the example.
    pub code: String,
}

// @field

/// `@field` — a named member of a table or class type.
#[derive(Debug, Clone)]
pub struct Field {
    pub name: String,
    pub description: Option<String>,
    pub field_type: Option<TypeExpr>,
}

// @section / @type / @within

/// `@section` — starts a named grouping section.
#[derive(Debug, Clone)]
pub struct Section {
    /// Section identifier (must be unique within the module).
    pub name: String,
    /// Summary used as the section title.
    pub summary: Option<String>,
    /// Optional extended description shown at the top of section details.
    pub description: Option<String>,
}

/// `@type` — a section describing a class / type and its methods.
#[derive(Debug, Clone)]
pub struct TypeSection {
    pub name: String,
    pub summary: Option<String>,
    pub description: Option<String>,
}

/// `@within` — places an item into an implicit (possibly pre-existing) section.
#[derive(Debug, Clone)]
pub struct Within {
    pub section_name: String,
}

// Annotations (appear inside function bodies)

/// In-body annotations: `@fixme`, `@todo`, `@warning`.
#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    Todo {
        message: String,
        location: Option<Location>,
    },
    Fixme {
        message: String,
        location: Option<Location>,
    },
    Warning {
        message: String,
        location: Option<Location>,
    },
}

// Module-level / project-level tags

/// Tags that are only meaningful on a module-level doc comment.
#[derive(Debug, Clone)]
pub enum ModuleTag {
    /// `@author <text>` — may appear multiple times.
    Author(String),
    /// `@copyright <text>`
    Copyright(String),
    /// `@license <text>`
    License(String),
    /// `@release <text>`
    Release(String),
    /// `@usage <text>` on a module — presented verbatim in a code font.
    Usage(String),
    /// `@export` — marks that explicit exports are listed at the end of the file.
    Export,
    /// `@set key=value` — overrides a config variable for this module only.
    Set { key: String, value: String },
    /// `@charset <enc>` — overrides the default UTF-8 output encoding.
    Charset(String),
    /// `@lookup <module>` — used in topic/readme documents to resolve bare refs.
    Lookup(String),
}

// Item kinds

/// Every first-class item kind recognised by LDoc.
#[derive(Debug, Clone, PartialEq)]
pub enum ItemKind {
    /// `@module` — a Lua library loadable with `require()`.
    Module,
    /// `@classmod` — a module that exports a single class.
    ClassMod,
    /// `@submodule` — contributes its items to a named master module.
    SubModule {
        master: String,
    },
    /// `@script` — a Lua program / CLI tool.
    Script,

    /// `@function` — an exported function.
    Function,
    /// `@lfunction` — a local (unexported) function.
    LFunction,
    /// `@table` — an exported Lua table.
    Table,
    /// `@field` — a module-level exported field / constant.
    Field,

    /// `@section` — an ad-hoc section grouping.
    Section,
    /// `@type` — a class/type section.
    Type,

    Custom(String),
}

// A complete doc-comment block

/// The fully parsed representation of a single LDoc doc-comment.
#[derive(Debug, Clone)]
pub struct DocComment {
    /// One-sentence summary (ends with `.` or `?`).
    pub summary: String,
    /// Optional multi-line description following the summary.
    pub description: Option<String>,
    /// The kind of entity being documented (if determinable).
    pub kind: Option<ItemKind>,
    /// Explicit name from a tag (e.g. `@function my_func`).
    pub name: Option<String>,
    /// Source location of the opening comment delimiter.
    pub location: Option<Location>,

    pub params: Vec<Param>,
    pub returns: Vec<Return>,
    pub errors: Vec<ErrorReturn>,
    pub raises: Vec<Raise>,

    pub fields: Vec<Field>,

    pub see: Vec<See>,
    pub usage: Vec<Usage>,

    /// `@local` — explicitly marks this item as non-exported.
    pub is_local: bool,
    /// `@within <section>` — places item into an implicit section.
    pub within: Option<Within>,
    /// `@section` data if this comment opens a new section.
    pub section: Option<Section>,

    pub annotations: Vec<Annotation>,

    pub module_tags: Vec<ModuleTag>,
}

impl Default for DocComment {
    fn default() -> Self {
        Self {
            summary: String::new(),
            description: None,
            kind: None,
            name: None,
            location: None,
            params: Vec::new(),
            returns: Vec::new(),
            errors: Vec::new(),
            raises: Vec::new(),
            fields: Vec::new(),
            see: Vec::new(),
            usage: Vec::new(),
            is_local: false,
            within: None,
            section: None,
            annotations: Vec::new(),
            module_tags: Vec::new(),
        }
    }
}

// Project / module container

/// A single documented item (function, table, field, …) inside a module.
#[derive(Debug, Clone)]
pub struct Item {
    pub doc: DocComment,
    pub kind: ItemKind,
    pub name: String,
    pub location: Option<Location>,
}

/// A top-level module / script / classmod.
#[derive(Debug, Clone)]
pub struct Module {
    pub doc: DocComment,
    pub kind: ItemKind,
    pub name: String,
    pub file: String,
    pub items: Vec<Item>,
    pub sections: Vec<Section>,
}

/// The root of a processed LDoc project.
#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub description: Option<String>,
    pub modules: Vec<Module>,
}

// Config-level constructs (config.ld equivalents)

/// Markup format for processing doc-comment prose.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum MarkupFormat {
    #[default]
    Plain,
    Markdown,
    Discount,
    Lunamark,
    Backticks,
}

/// Code prettifier backend.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum Prettifier {
    #[default]
    Builtin,
    Lxsh,
}

/// A user-defined tag type registered with `new_type(tag, header, …)`.
#[derive(Debug, Clone)]
pub struct CustomTagType {
    /// Tag name used in source, e.g. `"macro"`.
    pub tag: String,
    /// Section header shown in generated docs, e.g. `"Macros"`.
    pub header: String,
    /// Whether this is a project-level tag (default `false`).
    pub project_level: bool,
    /// Name of the sub-tag used for arguments/fields (e.g. `"param"`).
    pub sub_tag: Option<String>,
}

/// A user-defined display-name handler entry.
#[derive(Debug, Clone)]
pub struct CustomTag {
    pub name: String,
    pub title: Option<String>,
    pub hidden: bool,
}

/// A pattern-based custom `@see` handler.
#[derive(Debug, Clone)]
pub struct CustomSeeHandler {
    /// Lua-style pattern string matched against the see-reference.
    pub pattern: String,
    /// Human description of the handler for diagnostics.
    pub description: Option<String>,
}
