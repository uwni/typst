#[doc(inline)]
pub use typst_macros::{scope, ty};

use std::cmp::Ordering;
use std::fmt::{self, Debug, Display, Formatter};
use std::sync::LazyLock;

use ecow::{EcoString, eco_format};
use typst_utils::Static;

use crate::diag::{DeprecationSink, StrResult, bail};
use crate::foundations::{
    AutoValue, Func, NativeFuncData, NoneValue, Repr, Scope, Value, cast, func,
};

/// Describes a kind of value.
///
/// To style your document, you need to work with values of different kinds:
/// Lengths specifying the size of your elements, colors for your text and
/// shapes, and more. Typst categorizes these into clearly defined _types_ and
/// tells you where it expects which type of value.
///
/// Apart from basic types for numeric values and [typical]($int)
/// [types]($float) [known]($str) [from]($array) [programming]($dictionary)
/// languages, Typst provides a special type for [_content._]($content) A value
/// of this type can hold anything that you can enter into your document: Text,
/// elements like headings and shapes, and style information.
///
/// # Example
/// ```example
/// #let x = 10
/// #if type(x) == int [
///   #x is an integer!
/// ] else [
///   #x is another value...
/// ]
///
/// An image is of type
/// #type(image("glacier.jpg")).
/// ```
///
/// The type of `{10}` is `int`. Now, what is the type of `int` or even `type`?
/// ```example
/// #type(int) \
/// #type(type)
/// ```
///
/// Unlike other types like `int`, [none] and [auto] do not have a name
/// representing them. To test if a value is one of these, compare your value to
/// them directly, e.g:
/// ```example
/// #let val = none
/// #if val == none [
///   Yep, it's none.
/// ]
/// ```
///
/// Note that `type` will return [`content`] for all document elements. To
/// programmatically determine which kind of content you are dealing with, see
/// [`content.func`].
#[ty(scope, cast)]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Type(Static<NativeTypeData>);

impl Type {
    /// Get the type for `T`.
    pub fn of<T: NativeType>() -> Self {
        T::ty()
    }

    /// The type's short name, how it is used in code (e.g. `str`).
    pub fn short_name(&self) -> &'static str {
        self.0.name
    }

    /// The type's long name, for use in diagnostics (e.g. `string`).
    pub fn long_name(&self) -> &'static str {
        self.0.long_name
    }

    /// The type's title case name, for use in documentation (e.g. `String`).
    pub fn title(&self) -> &'static str {
        self.0.title
    }

    /// Documentation for the type (as Markdown).
    pub fn docs(&self) -> &'static str {
        self.0.docs
    }

    /// Search keywords for the type.
    pub fn keywords(&self) -> &'static [&'static str] {
        self.0.keywords
    }

    /// This type's constructor function.
    pub fn constructor(&self) -> StrResult<Func> {
        self.0
            .constructor
            .as_ref()
            .map(|lazy| Func::from(*lazy))
            .ok_or_else(|| eco_format!("type {self} does not have a constructor"))
    }

    /// The type's associated scope that holds sub-definitions.
    pub fn scope(&self) -> &'static Scope {
        &(self.0).0.scope
    }

    /// Get a field from this type's scope, if possible.
    pub fn field(
        &self,
        field: &str,
        sink: impl DeprecationSink,
    ) -> StrResult<&'static Value> {
        match self.scope().get(field) {
            Some(binding) => Ok(binding.read_checked(sink)),
            None => bail!("type {self} does not contain field `{field}`"),
        }
    }
}

#[scope]
impl Type {
    /// Determines a value's type.
    ///
    /// ```example
    /// #type(12) \
    /// #type(14.7) \
    /// #type("hello") \
    /// #type(<glacier>) \
    /// #type([Hi]) \
    /// #type(x => x + 1) \
    /// #type(type)
    /// ```
    #[func(constructor)]
    pub fn construct(
        /// The value whose type's to determine.
        value: Value,
    ) -> Type {
        value.ty()
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Type({})", self.long_name())
    }
}

impl Repr for Type {
    fn repr(&self) -> EcoString {
        if *self == Type::of::<AutoValue>() {
            "type(auto)"
        } else if *self == Type::of::<NoneValue>() {
            "type(none)"
        } else {
            self.short_name()
        }
        .into()
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.pad(self.long_name())
    }
}

impl Ord for Type {
    fn cmp(&self, other: &Self) -> Ordering {
        self.long_name().cmp(other.long_name())
    }
}

impl PartialOrd for Type {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A Typst type that is defined by a native Rust type.
pub trait NativeType {
    /// The type's name.
    ///
    /// In contrast to `data()`, this is usable in const contexts.
    const NAME: &'static str;

    /// Get the type for the native Rust type.
    fn ty() -> Type {
        Type::from(Self::data())
    }

    // Get the type data for the native Rust type.
    fn data() -> &'static NativeTypeData;
}

/// Defines a native type.
#[derive(Debug)]
pub struct NativeTypeData {
    /// The type's normal name (e.g. `str`), as exposed to Typst.
    pub name: &'static str,
    /// The type's long name (e.g. `string`), for error messages.
    pub long_name: &'static str,
    /// The function's title case name (e.g. `String`).
    pub title: &'static str,
    /// The documentation for this type as a string.
    pub docs: &'static str,
    /// A list of alternate search terms for this type.
    pub keywords: &'static [&'static str],
    /// The constructor for this type.
    pub constructor: LazyLock<Option<&'static NativeFuncData>>,
    /// Definitions in the scope of the type.
    pub scope: LazyLock<Scope>,
}

impl From<&'static NativeTypeData> for Type {
    fn from(data: &'static NativeTypeData) -> Self {
        Self(Static(data))
    }
}

cast! {
    &'static NativeTypeData,
    self => Type::from(self).into_value(),
}
