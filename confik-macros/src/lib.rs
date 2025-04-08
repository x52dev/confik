use std::fmt::Display;

use darling::{
    ast::{self, NestedMeta, Style},
    util::{Flag, SpannedValue},
    FromDeriveInput, FromField, FromMeta, FromVariant, ToTokens,
};
use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{
    parse2, parse_macro_input, spanned::Spanned, DeriveInput, Expr, Generics, Index, Meta, Path,
    Type, Visibility,
};

#[cfg(test)]
mod tests;

/// Entry point for rustc.
#[proc_macro_derive(Configuration, attributes(confik))]
pub fn derive_macro_builder(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let target_struct = parse_macro_input!(input as DeriveInput);

    match derive_macro_builder_inner(&target_struct) {
        Ok(token_stream) => token_stream,
        Err(err) => err.to_compile_error().into(),
    }
}

/// Handles `from` attributes for dealing with foreign types.
#[derive(Debug)]
struct FieldFrom {
    ty: Type,
}

impl FromMeta for FieldFrom {
    fn from_expr(ty: &Expr) -> darling::Result<Self> {
        let Ok(ty) = parse2(ty.to_token_stream()) else {
            return Err(syn::Error::new(
                ty.span(),
                format!("Unable to parse type from: {}", ty.to_token_stream()),
            )
            .into());
        };

        Ok(Self { ty })
    }
}

/// Handles `try_from` attributes for dealing with foreign types.
#[derive(Debug)]
struct FieldTryFrom {
    ty: Type,
}

impl FromMeta for FieldTryFrom {
    fn from_expr(ty: &Expr) -> darling::Result<Self> {
        let Ok(ty) = parse2(ty.to_token_stream()) else {
            return Err(syn::Error::new(
                ty.span(),
                format!("Unable to parse type from: {}", ty.to_token_stream()),
            )
            .into());
        };

        Ok(Self { ty })
    }
}

/// Handles requesting to forward attributes.
#[derive(Debug)]
struct Forward {
    items: Vec<NestedMeta>,
}

impl ToTokens for Forward {
    fn into_token_stream(self) -> TokenStream {
        self.to_token_stream()
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream());
    }

    fn to_token_stream(&self) -> TokenStream {
        let Self { items } = self;
        quote!(#( #[ #items ] )*)
    }
}

impl FromMeta for Forward {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let items = items.to_vec();

        Ok(Self { items })
    }
}

/// Parser for a default attribute.
#[derive(Debug)]
struct FieldDefaulter {
    expr: Expr,
}

impl FromMeta for FieldDefaulter {
    fn from_word() -> darling::Result<Self> {
        Ok(Self {
            expr: syn::parse_str("Default::default()").unwrap(),
        })
    }

    fn from_expr(default: &Expr) -> darling::Result<Self> {
        let default_into_expr = quote_spanned!(default.span() => { #default }.into() );
        let expr = parse2(default_into_expr)
            .expect("expression should still be valid after being wrapped");
        Ok(Self { expr })
    }
}

/// Implemented for enum variants.
#[derive(Debug, FromVariant)]
#[darling(attributes(confik))]
struct VariantImplementer {
    /// The variant name.
    ident: Ident,

    /// The fields.
    fields: ast::Fields<SpannedValue<FieldImplementer>>,

    /// Optional explicit override of the variant's discriminant.
    discriminant: Option<Expr>,

    /// Optional attributes to forward to the builder's variant.
    forward: Option<Forward>,
}

impl VariantImplementer {
    /// Define the builder variant for a given target variant
    fn define_builder(var_impl: &SpannedValue<Self>) -> syn::Result<TokenStream> {
        let Self {
            ident,
            fields,
            discriminant,
            forward,
        } = var_impl.as_ref();

        let field_vec = fields
            .iter()
            .map(FieldImplementer::define_builder)
            .collect::<Result<Vec<_>, _>>()?;
        let fields = ast::Fields::new(fields.style, field_vec).into_token_stream();

        let discriminant = discriminant
            .as_ref()
            .map(|disc| quote_spanned!(disc.span() => = discriminant));

        Ok(quote_spanned! { var_impl.span() =>
            #forward
            #ident #fields #discriminant
        })
    }

    fn impl_merge(var_impl: &SpannedValue<Self>) -> TokenStream {
        let Self { ident, fields, .. } = var_impl.as_ref();

        let style = fields.style;
        let extract_us_fields = fields
            .as_ref()
            .iter()
            .enumerate()
            .map(|(index, field)| FieldImplementer::extract_for_match(index, field, "us"))
            .collect::<Vec<_>>();
        let bracketed_extract_us_fields =
            ast::Fields::new(style, extract_us_fields).into_token_stream();

        let extract_other_fields = fields
            .as_ref()
            .iter()
            .enumerate()
            .map(|(index, field)| FieldImplementer::extract_for_match(index, field, "other"))
            .collect::<Vec<_>>();
        let bracketed_extract_other_fields =
            ast::Fields::new(style, extract_other_fields).into_token_stream();

        let field_merge = fields
            .as_ref()
            .iter()
            .enumerate()
            .map(|(index, field)| FieldImplementer::impl_enum_merge(index, field, style))
            .collect::<Vec<_>>();
        let bracketed_field_merge = ast::Fields::new(style, field_merge).into_token_stream();

        quote_spanned! {var_impl.span() =>
            (Self::#ident #bracketed_extract_us_fields, Self::#ident #bracketed_extract_other_fields) => Self::#ident #bracketed_field_merge
        }
    }

    fn impl_try_build(var_impl: &SpannedValue<Self>) -> TokenStream {
        let Self { ident, fields, .. } = var_impl.as_ref();

        let style = fields.style;
        let extract_us_fields = fields
            .as_ref()
            .iter()
            .enumerate()
            .map(|(index, field)| FieldImplementer::extract_for_match(index, field, "us"))
            .collect::<Vec<_>>();
        let bracketed_extract_us_fields =
            ast::Fields::new(style, extract_us_fields).into_token_stream();

        let try_build = fields
            .as_ref()
            .iter()
            .enumerate()
            .map(|(index, field)| {
                FieldImplementer::impl_try_build(
                    index,
                    field,
                    style,
                    Some("us"),
                    Some(&ident.to_string()),
                )
            })
            .collect::<Vec<_>>();
        let bracketed_try_build = ast::Fields::new(style, try_build).into_token_stream();

        quote_spanned! {var_impl.span() =>
            Self::#ident #bracketed_extract_us_fields => Self::Target::#ident #bracketed_try_build
        }
    }

    fn impl_contains_non_secret_data(var_impl: &SpannedValue<Self>) -> TokenStream {
        let Self { ident, fields, .. } = var_impl.as_ref();

        let style = fields.style;
        let extract_us_fields = fields
            .as_ref()
            .iter()
            .enumerate()
            .map(|(index, field)| FieldImplementer::extract_for_match(index, field, "us"))
            .collect::<Vec<_>>();
        let bracketed_extract_us_fields =
            ast::Fields::new(style, extract_us_fields).into_token_stream();

        let contains_non_secret_data = fields
            .as_ref()
            .iter()
            .enumerate()
            .map(|(index, field)| {
                FieldImplementer::impl_contains_non_secret_data(index, field, Some("us"))
            })
            .collect::<Vec<_>>();

        let string = ident.to_string();

        quote_spanned! {var_impl.span() =>
            Self::#ident #bracketed_extract_us_fields => false #( | #contains_non_secret_data.map_err(|err| err.prepend(#string))? )*
        }
    }
}

/// A field may have an explicit ident, i.e. `struct A { b: () }`, or might use an index,
/// i.e. `struct A(());`. This abstracts over the ident so that either can be used.
enum FieldIdent<'a> {
    Ident(&'a Ident),
    Index(Index),
}

impl Display for FieldIdent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Ident(id) => id.fmt(f),
            Self::Index(ix) => ix.index.fmt(f),
        }
    }
}

impl<'a> FieldIdent<'a> {
    /// If the ident exists, use that, otherwise use indexing.
    fn new(ident: &'a Option<Ident>, index: usize) -> Self {
        if let Some(ident) = ident {
            Self::Ident(ident)
        } else {
            Self::Index(Index::from(index))
        }
    }
}

impl ToTokens for FieldIdent<'_> {
    fn into_token_stream(self) -> TokenStream {
        self.to_token_stream()
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream());
    }

    fn to_token_stream(&self) -> TokenStream {
        match self {
            Self::Ident(id) => id.to_token_stream(),
            Self::Index(ix) => ix.to_token_stream(),
        }
    }
}

/// Implementer for struct fields, including those embedded inside an enum, e.g.,
/// `enum A { B { c: () } }`
#[derive(Debug, FromField)]
#[darling(attributes(confik))]
struct FieldImplementer {
    /// Whether to default the field to a value if it's not present.
    default: Option<FieldDefaulter>,

    /// Whether the field is a secret, and should be implemented via `SecretBuilder`.
    secret: Flag,

    /// A type which implements `Configuration`, for which the field implements `From`.
    /// Enables handling foreign types.
    from: Option<FieldFrom>,

    /// A type which implements `Configuration`, for which the field implements `TryFrom`.
    /// Enables handling foreign types.
    try_from: Option<FieldTryFrom>,

    /// The field name, if a named field.
    ///
    /// If not, then you will probably want to enumerate through the list of these and
    /// use that index.
    ident: Option<Ident>,

    /// The field type.
    ty: Type,

    /// Optional attributes to forward to the builder's field.
    forward: Option<Forward>,
}

impl FieldImplementer {
    /// Produces a new ident with a prefix.
    fn prefixed_ident(
        field_index: usize,
        field_impl: &SpannedValue<Self>,
        ident_prefix: &str,
    ) -> Ident {
        Ident::new(
            &format!(
                "{}{}",
                ident_prefix,
                field_impl
                    .ident
                    .as_ref()
                    .map_or(field_index.to_string(), ToString::to_string)
            ),
            field_impl.span(),
        )
    }

    /// Extract fields, e.g. in a match statement.
    ///
    /// For a tuple field with index 0, with a prefix of "us", this should look like: `us_0`.
    /// For a struct field with ident field1, with a prefix of "us", this should look like:
    /// `field1: us_field1`.
    fn extract_for_match(
        field_index: usize,
        field_impl: &SpannedValue<Self>,
        ident_prefix: &str,
    ) -> TokenStream {
        let maybe_field_specifier = field_impl
            .ident
            .as_ref()
            .map(|ident| quote_spanned!(ident.span() => #ident: ));

        let ident = Self::prefixed_ident(field_index, field_impl, ident_prefix);

        quote_spanned!(field_impl.span() => #maybe_field_specifier #ident)
    }

    /// Define the builder field for a given target field.
    fn define_builder(field_impl: &SpannedValue<Self>) -> syn::Result<TokenStream> {
        let Self {
            ty,
            ident,
            secret,
            forward,
            from,
            try_from,
            ..
        } = field_impl.as_ref();

        let ident = ident
            .as_ref()
            .map(|ident| quote_spanned!(ident.span() => #ident : ));

        // Builder type based on original field type via [`confik::Configuration`]
        // If `from` is set, then use that type instead.
        let ty = match (from, try_from) {
            (Some(from), Some(try_from)) => {
                let msg = "Cannot support both `try_from` and `from` confik attributes";
                let mut err = syn::Error::new(try_from.ty.span(), msg);
                err.combine(syn::Error::new(from.ty.span(), msg));
                return Err(err);
            }
            (Some(FieldFrom { ty }), None) | (None, Some(FieldTryFrom { ty })) => ty,
            (None, None) => ty,
        };

        let ty = quote_spanned!(ty.span() => <#ty as ::confik::Configuration>::Builder);

        // If secret then wrap in [`confik::SecretBuilder`]
        let ty = if secret.is_present() {
            quote_spanned!(ty.span() => ::confik::SecretBuilder<#ty>)
        } else {
            ty
        };

        Ok(quote_spanned! { ident.span() =>
                #[serde(default)]
                #forward
                #ident #ty
        })
    }

    /// Define how to merge the given field in a struct impl.
    fn impl_struct_merge(
        field_index: usize,
        field_impl: &SpannedValue<Self>,
        style: Style,
    ) -> TokenStream {
        let ident = FieldIdent::new(&field_impl.ident, field_index);

        let merge = quote_spanned! {
            field_impl.span() =>
            self.#ident.merge(other.#ident)
        };

        match style {
            Style::Struct => quote_spanned! { field_impl.span() =>
                #ident: #merge
            },
            Style::Tuple => merge,
            Style::Unit => panic!("Trying to call merge on a field in a unit struct"),
        }
    }

    /// Define how to merge the given field in an enum impl.
    fn impl_enum_merge(
        field_index: usize,
        field_impl: &SpannedValue<Self>,
        style: Style,
    ) -> TokenStream {
        let us_ident = Self::prefixed_ident(field_index, field_impl, "us");
        let other_ident = Self::prefixed_ident(field_index, field_impl, "other");
        let ident = FieldIdent::new(&field_impl.ident, field_index);

        let merge = quote_spanned! {
            field_impl.span() =>
            #us_ident.merge(#other_ident)
        };

        match style {
            Style::Struct => quote_spanned! { field_impl.span() =>
                #ident: #merge
            },
            Style::Tuple => merge,
            Style::Unit => panic!("Trying to call merge on a field in a unit struct"),
        }
    }

    /// Defines how to try to build the given field, including handling defaults.
    fn impl_try_build(
        field_index: usize,
        field_impl: &SpannedValue<Self>,
        style: Style,
        us_ident_prefix: Option<&str>,
        extra_prepend: Option<&str>,
    ) -> TokenStream {
        let ident = FieldIdent::new(&field_impl.ident, field_index);

        let our_field = if let Some(ident_prefix) = us_ident_prefix {
            Self::prefixed_ident(field_index, field_impl, ident_prefix).into_token_stream()
        } else {
            quote!(self.#ident)
        };

        let string = ident.to_string();

        let mut field_build = quote_spanned! {
            field_impl.span() =>
            #our_field.try_build()
        };

        // Default if no data is present
        if let Some(default) = &field_impl.default {
            let default = &default.expr;

            field_build = quote_spanned! {
                default.span() =>
                    if #our_field.contains_non_secret_data().unwrap_or(true) {
                        #field_build
                    } else {
                        Ok(#default)
                    }
            };
        }

        let extra_prepend = extra_prepend.map(|extra_prepend| quote!(.prepend(#extra_prepend)));
        field_build = quote_spanned! {
            field_build.span() => #field_build.map_err(|err| err.prepend(#string)#extra_prepend)?
        };

        // We're going via another type to allow handling the field being a foreign type. Do the conversion.
        if field_impl.from.is_some() {
            field_build = quote_spanned! {
                field_build.span() => #field_build.into()
            }
        } else if field_impl.try_from.is_some() {
            field_build = quote_spanned! {
                field_build.span() => #field_build.try_into().map_err(|e|
                    ::confik::FailedTryInto::new(e)
                )?
            }
        }

        match style {
            Style::Struct => quote_spanned! { field_impl.span() =>
                #ident: #field_build
            },
            Style::Tuple => field_build,
            Style::Unit => panic!("Trying to call merge on a field in a unit struct"),
        }
    }

    /// Defines how to check that the field does not contain secret data.
    fn impl_contains_non_secret_data(
        field_index: usize,
        field_impl: &SpannedValue<Self>,
        us_ident_prefix: Option<&str>,
    ) -> TokenStream {
        let ident = FieldIdent::new(&field_impl.ident, field_index);

        let our_field = if let Some(ident_prefix) = us_ident_prefix {
            Self::prefixed_ident(field_index, field_impl, ident_prefix).into_token_stream()
        } else {
            quote!(self.#ident)
        };

        let string = ident.to_string();

        quote_spanned! {
            field_impl.span() =>
            #our_field.contains_non_secret_data().map_err(|err| err.prepend(#string))
        }
    }
}

/// List of attributes to be derived.
#[derive(Debug)]
struct Derive {
    items: Vec<Path>,
}

impl FromMeta for Derive {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let items = items
            .iter()
            .map(|item| {
                if let NestedMeta::Meta(Meta::Path(path)) = item {
                    Ok(path.clone())
                } else {
                    Err(syn::Error::new(
                        item.span(),
                        format!("Expected a path to a derivable trait, got {item:?}"),
                    ))
                }
            })
            .collect::<Result<Vec<_>, syn::Error>>()?;

        Ok(Self { items })
    }
}

impl ToTokens for Derive {
    fn into_token_stream(self) -> TokenStream {
        self.to_token_stream()
    }

    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.to_token_stream());
    }

    fn to_token_stream(&self) -> TokenStream {
        let Self { items } = self;
        quote!( #( #items ),*)
    }
}

/// Driver for the implementation of `#[derive(Configuration)]`.
#[derive(Debug, FromDeriveInput)]
#[darling(attributes(confik))]
struct RootImplementer {
    /// The ident/name of the target (the struct/enum the derive was called on).
    ///
    /// To get the builder name, see [`RootImplementer::builder_name`].
    ident: Ident,

    // #[darling(rename = "ident")]
    // target_name: Ident,
    //
    // /// The ident/name of the builder that this will use.
    // ///
    // /// In most cases, this will be a new struct/enum, but in some cases a pre-existing builder,
    // /// e.g. `Option` may be used.
    //
    // #[darling(rename = "ident", map = "builder_name")]
    // builder_name: Ident,
    //
    /// Generics from the target, these will be propagated to the builder.
    generics: Generics,

    /// Fields, handled by [`EnumFieldImplementer`] or [`StructFieldImplementer`] depending on
    /// target type.
    data: ast::Data<SpannedValue<VariantImplementer>, SpannedValue<FieldImplementer>>,

    /// `pub`, `pub(crate)`, etc.
    vis: Visibility,

    /// Optional attributes to forward to the builder struct/enum.
    ///
    /// This can be serde attributes e.g. `#[confik(forward(serde(default)))]` but also others like
    /// `#[confik(forward(derive(Hash)))]`
    forward: Option<Forward>,
}

impl RootImplementer {
    /// Check that the type can be instantiated. This currently just checks that the type
    /// is not a variant-less `enum`, e.g.
    ///
    /// ```rust
    /// enum A {}
    /// ```
    fn check_valid(&self) -> syn::Result<()> {
        if matches!(&self.data, ast::Data::Enum(variants) if variants.is_empty()) {
            Err(syn::Error::new(
                self.ident.span(),
                format!(
                    "Cannot create a builder for a type that cannot be instantiated: {}",
                    self.ident
                ),
            ))
        } else {
            Ok(())
        }
    }

    /// What the builder name would be for the target, even if one doesn't exist.
    ///
    /// Use [`Self::is_dataless`] first to determine whether a builder will exist.
    fn builder_name(&self) -> Ident {
        format_ident!("{}ConfigBuilder", self.ident)
    }

    /// Defines the builder for the target.
    fn define_builder(&self) -> syn::Result<TokenStream> {
        let Self {
            ident: target_name,
            data,
            generics,
            vis,
            forward,
            ..
        } = self;

        let builder_name = self.builder_name();

        let enum_or_struct_token = if data.is_struct() {
            syn::token::Struct {
                span: target_name.span(),
            }
            .into_token_stream()
        } else {
            syn::token::Enum {
                span: target_name.span(),
            }
            .into_token_stream()
        };

        let bracketed_data = match &self.data {
            ast::Data::Enum(variants) => {
                let variants = variants
                    .iter()
                    .map(VariantImplementer::define_builder)
                    .collect::<Result<Vec<_>, _>>()?;

                quote_spanned! { target_name.span() =>
                    {
                        #( #variants, )*
                        #[default]
                        ConfigBuilderUndefined,
                    }
                }
            }
            ast::Data::Struct(fields) if fields.is_empty() => {
                quote!({})
            }
            ast::Data::Struct(fields) => {
                let field_vec = fields
                    .iter()
                    .map(FieldImplementer::define_builder)
                    .collect::<Result<Vec<_>, _>>()?;
                ast::Fields::new(fields.style, field_vec).into_token_stream()
            }
        };

        // Tuple structs must end in `;`. However if a normal struct ends in `;` then the `impl` for
        // the builder is not printed by rustc when it calls into this `proc-macro`, even when it is
        // present...
        //
        // Therefore, conditionally add the `;`.
        let terminator = matches!(
            &self.data,
            ast::Data::Struct(fields) if fields.style.is_tuple(),
        )
        .then_some(quote!(;));

        let (_impl_generics, type_generics, where_clause) = generics.split_for_impl();

        Ok(quote_spanned! { target_name.span() =>
            #[derive(::std::default::Default, ::confik::__exports::__serde::Deserialize)]
            #[serde(crate = "::confik::__exports::__serde")]
            #forward
            #vis #enum_or_struct_token #builder_name #type_generics #where_clause
                #bracketed_data
            #terminator
        })
    }

    /// Implement the `ConfigurationBuilder::merge` method for our builder.
    fn impl_merge(&self) -> TokenStream {
        let Self { data, .. } = self;

        let field_merge = match data {
            ast::Data::Struct(fields) if fields.is_empty() => {
                quote!(Self {})
            }
            ast::Data::Struct(fields) => {
                let style = fields.style;
                let fields = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| FieldImplementer::impl_struct_merge(index, field, style))
                    .collect::<Vec<_>>();
                let bracketed_fields = ast::Fields::new(style, fields).into_token_stream();
                quote!(Self #bracketed_fields)
            }
            // Undefined variant must go first to take precedence in the match.
            ast::Data::Enum(variants) => {
                let variants = variants
                    .iter()
                    .map(VariantImplementer::impl_merge)
                    .collect::<Vec<_>>();
                quote!(match (self, other) {
                    (Self::ConfigBuilderUndefined, other) => other,
                    #( #variants, )*
                    (us, _) => us,
                })
            }
        };

        quote! {
            fn merge(self, other: Self) -> Self {
                #field_merge
            }
        }
    }

    /// Implement the `ConfigurationBuilder::try_build` method for our builder.
    fn impl_try_build(&self) -> TokenStream {
        let Self { ident, data, .. } = self;

        let field_build = match data {
            ast::Data::Struct(fields) => {
                let style = fields.style;
                let fields = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        FieldImplementer::impl_try_build(index, field, fields.style, None, None)
                    })
                    .collect::<Vec<_>>();
                let bracketed_fields = ast::Fields::new(style, fields).into_token_stream();
                quote!(Ok(#ident #bracketed_fields))
            }
            ast::Data::Enum(variants) => {
                let variants = variants
                    .iter()
                    .map(VariantImplementer::impl_try_build)
                    .collect::<Vec<_>>();
                quote! {
                    Ok(match self {
                        Self::ConfigBuilderUndefined => return Err(::confik::Error::MissingValue(<::confik::MissingValue as ::std::default::Default>::default())),
                        #( #variants, )*
                    })
                }
            }
        };

        quote! {
            // Allow useless conversions as the default handling may call `.into()` unnecessarily.
            #[allow(clippy::useless_conversion)]
            fn try_build(self) -> ::std::result::Result<Self::Target, ::confik::Error> {
                #field_build
            }
        }
    }

    /// Implement the `ConfigurationBuilder::contains_non_secret_data` method for our builder.
    fn impl_contains_non_secret_data(&self) -> TokenStream {
        let field_check = match &self.data {
            ast::Data::Struct(fields) => {
                let field_check = fields
                    .iter()
                    .enumerate()
                    .map(|(index, field)| {
                        FieldImplementer::impl_contains_non_secret_data(index, field, None)
                    })
                    .collect::<Vec<_>>();
                quote!(false #( | #field_check? )*)
            }
            ast::Data::Enum(variants) => {
                let variant_check = variants
                    .iter()
                    .map(VariantImplementer::impl_contains_non_secret_data)
                    .collect::<Vec<_>>();
                quote! { match self {
                    Self::ConfigBuilderUndefined => false,
                    #( #variant_check, )*
                }}
            }
        };

        quote! {
            fn contains_non_secret_data(&self) -> ::std::result::Result<::std::primitive::bool, ::confik::UnexpectedSecret> {
                Ok(#field_check)
            }
        }
    }

    /// Implement `ConfigurationBuilder` for our builder.
    fn impl_builder(&self) -> TokenStream {
        let Self {
            ident: target_name,
            generics,
            ..
        } = self;
        let builder_name = self.builder_name();

        let merge = self.impl_merge();
        let try_build = self.impl_try_build();

        let contains_non_secret_data = self.impl_contains_non_secret_data();

        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

        quote! {
            impl #impl_generics ::confik::ConfigurationBuilder  for #builder_name #type_generics #where_clause {
                type Target = #target_name #type_generics;

                #merge

                #try_build

                #contains_non_secret_data
            }
        }
    }

    /// Implement `Configuration` for our target.
    fn impl_target(&self) -> TokenStream {
        let Self {
            ident: target_name,
            generics,
            ..
        } = self;
        let builder_name = self.builder_name();
        let builder = quote!(#builder_name #generics);

        let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

        quote! {
            impl #impl_generics ::confik::Configuration for #target_name #type_generics  #where_clause {
                type Builder = #builder;
            }
        }
    }
}

fn derive_macro_builder_inner(target_struct: &DeriveInput) -> syn::Result<proc_macro::TokenStream> {
    let implementer = RootImplementer::from_derive_input(target_struct)?;
    implementer.check_valid()?;
    let builder_struct = implementer.define_builder()?;
    let builder_impl = implementer.impl_builder();
    let target_impl = implementer.impl_target();

    let overall_lint_overrides = quote! {
        #[doc(hidden)] // crate docs should cover builders' uses.
    };

    let impl_lint_overrides = quote! {
        #[allow(clippy::needless_question_mark)] // Some `?` are used to simplify code generation even when they're not needed
        #[automatically_derived] // Turns off some passes that make sense for automatically derived impls.
    };

    // These lints mostly consist of lints that are [allowed by default] but may be enabled by users.
    //
    // [allow by default]: https://doc.rust-lang.org/rustc/lints/listing/allowed-by-default.html
    let struct_lint_overrides = quote! {
        #[allow(
            missing_copy_implementations, // Some builders may be able to be `Copy` but do not benefit from it.
            missing_debug_implementations, // Builders do not need `Debug` by default, can be opted in where needed.
            variant_size_differences, // We add an empty enum varaint (`*Undefined`) which may be much smaller than other variants.
        )]
    };

    let full_derive = quote! {
        #overall_lint_overrides
        const _: () = {
            #impl_lint_overrides
            #target_impl

            #struct_lint_overrides
            #builder_struct

            #impl_lint_overrides
            #builder_impl
        };
    };

    Ok(full_derive.into())
}
