use std::collections::HashMap;

use convert_case::{Case, Casing};
use quote::{format_ident, quote};
use syn::*;

#[derive(Clone)]
struct EnumDescription {
    name: Ident,
    attributes: HashMap<String, proc_macro2::TokenStream>,
    repr_type: Type,
    visibility: Visibility,
    variants: Vec<Ident>,
    variant_types: Vec<Ident>,
    variant_fields: Vec<Fields>,
    variant_discriminants: Vec<Expr>,
    variant_attributes: Vec<HashMap<String, proc_macro2::TokenStream>>,
}

impl EnumDescription {
    fn new(ast: syn::DeriveInput) -> Self {
        let name = ast.ident;
        let visibility = ast.vis;
        let syn::Data::Enum(e) = ast.data else {
            panic!("Only enums are supported");
        };
        let mut attributes = attribute_maps(ast.attrs);
        let repr_type =
            parse2(attributes.remove("repr").expect("missing repr")).expect("repr bad type");

        let mut variants = Vec::with_capacity(e.variants.len());
        let mut variant_fields = Vec::with_capacity(e.variants.len());
        let mut variant_types = Vec::with_capacity(e.variants.len());
        let mut variant_discriminants = Vec::with_capacity(e.variants.len());
        let mut variant_attributes = Vec::with_capacity(e.variants.len());

        for Variant {
            attrs,
            ident,
            fields,
            discriminant,
        } in e.variants
        {
            let attributes = attribute_maps(attrs);
            variant_types.push(format_ident!("{}{}", name, ident));
            variants.push(ident);
            variant_fields.push(fields);
            variant_discriminants.push(discriminant.expect("").1);
            variant_attributes.push(attributes);
        }

        Self {
            name,
            attributes,
            repr_type,
            visibility,
            variants,
            variant_types,
            variant_fields,
            variant_discriminants,
            variant_attributes,
        }
    }

    fn variant_match(&self) -> Vec<proc_macro2::TokenStream> {
        self.variant_fields
            .iter()
            .map(|fields| match fields {
                Fields::Named(_) => quote!({ .. }),
                Fields::Unnamed(_) => quote!((_)),
                Fields::Unit => quote!(),
            })
            .collect()
    }

    fn tag_enum(&self) -> proc_macro2::TokenStream {
        let Self {
            name,
            repr_type,
            visibility,
            variants,
            variant_discriminants,
            ..
        } = self;
        quote!(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            #[repr(#repr_type)]
            #visibility enum #name {
                #(
                    #variants = #variant_discriminants,
                )*
            }
        )
    }

    fn tag_impl(&self) -> proc_macro2::TokenStream {
        let Self {
            name,
            repr_type,
            variants,
            variant_discriminants,
            ..
        } = self;
        let to_impl = quote!(
            impl From<#name> for #repr_type {
                fn from(value: #name) -> Self {
                    match value {
                        #(
                            #name::#variants => #variant_discriminants,
                        )*
                    }
                }
            }
        );
        let from_impl = quote!(
            impl TryFrom<#repr_type> for #name {
                type Error = ();
                fn try_from(value: #repr_type) -> Result<Self, Self::Error> {
                    match value {
                        #(
                            #variant_discriminants => Ok(#name::#variants),
                        )*
                        _ => Err(()),
                    }
                }
            }
        );
        quote!(
            #to_impl
            #from_impl
        )
    }

    fn tagged_impl(&self) -> proc_macro2::TokenStream {
        let mut tag = self.clone();

        let kind_suffix = if let Some(tokens) = tag.attributes.remove("tagged_enum_kind") {
            parse2::<Ident>(tokens)
                .expect("could not parse tagged_enum_kind")
                .to_string()
        } else {
            "Kind".to_string()
        };
        let kind_func_name = format_ident!(
            "{}",
            kind_suffix.from_case(Case::Pascal).to_case(Case::Snake)
        );

        tag.name = format_ident!("{}{}", tag.name, kind_suffix);
        tag.variant_fields.iter_mut().for_each(|f| {
            *f = Fields::Unit;
        });
        let tag = tag;
        let tag_name = &tag.name;
        let variant_match = self.variant_match();

        let Self {
            name,
            attributes,
            repr_type,
            visibility,
            variants,
            variant_types,
            variant_fields,
            variant_discriminants,
            variant_attributes,
            ..
        } = self;

        let variant_field_names = variant_fields
            .iter()
            .map(|field| match field {
                Fields::Named(FieldsNamed {
                    brace_token: _,
                    named,
                }) => named
                    .iter()
                    .map(|field| field.ident.clone().expect("should be named"))
                    .collect(),
                Fields::Unnamed(FieldsUnnamed {
                    paren_token: _,
                    unnamed,
                }) => unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format_ident!("_value{i}"))
                    .collect(),
                Fields::Unit => Vec::new(),
            })
            .collect::<Vec<_>>();
        let variant_field_names = &variant_field_names;

        let with_serde = enable_serde(attributes);

        let serde = if with_serde {
            let serialize_variant = variants
                .iter()
                .zip(variant_types)
                .zip(variant_fields)
                .zip(variant_field_names)
                .zip(variant_discriminants)
                .map(|((((variant, variant_type), fields), names), discriminant)| match fields {
                    Fields::Named(_) => quote!(
                        Self::#variant { #(#names,)* } => {
                            let discriminant : #repr_type = #discriminant;
                            let variant = ::std::mem::ManuallyDrop::new(#variant_type {
                                #(
                                    #names: unsafe {
                                        let mut value = ::std::mem::MaybeUninit::uninit();
                                        ::std::ptr::copy(#names as *const _, value.as_mut_ptr(), 1);
                                        value.assume_init()
                                    },
                                )*
                            });
                            ::serde::ser::SerializeTuple::serialize_element(&mut state, &discriminant)?;
                            ::serde::ser::SerializeTuple::serialize_element(&mut state, std::ops::Deref::deref(&variant))?;
                        }
                    ),
                    Fields::Unnamed(_) => quote!(
                        Self::#variant ( #(#names,)* ) => {
                            let discriminant : #repr_type = #discriminant;
                            let variant = ::std::mem::ManuallyDrop::new(#variant_type (
                                #(
                                    unsafe {
                                        let mut value = ::std::mem::MaybeUninit::uninit();
                                        ::std::ptr::copy(#names as *const _, value.as_mut_ptr(), 1);
                                        value.assume_init()
                                    },
                                )*
                            ));
                            ::serde::ser::SerializeTuple::serialize_element(&mut state, &discriminant)?;
                            ::serde::ser::SerializeTuple::serialize_element(&mut state, std::ops::Deref::deref(&variant))?;
                        }
                    ),
                    Fields::Unit => quote!(
                        Self::#variant => {
                            let discriminant : #repr_type = #discriminant;
                            ::serde::ser::SerializeTuple::serialize_element(&mut state, &discriminant)?;
                            ::serde::ser::SerializeTuple::serialize_element(&mut state, &#variant_type)?;
                        }
                    ),
                })
                .collect::<Vec<_>>();
            quote!(
                impl ::serde::ser::Serialize for #name {
                    fn serialize<_S>(&self, serializer: _S) -> Result<_S::Ok, _S::Error>
                    where
                        _S: ::serde::ser::Serializer,
                    {
                        let mut state = serializer.serialize_tuple(2)?;
                        match self {
                            #(
                                #serialize_variant
                            )*
                        }
                        ::serde::ser::SerializeTuple::end(state)
                    }
                }
                impl<'de> ::serde::de::Deserialize<'de> for #name {
                    fn deserialize<_D>(deserializer: _D) -> Result<#name, _D::Error>
                    where
                        _D: ::serde::de::Deserializer<'de>,
                    {
                        struct Visitor;
                        impl<'de> serde::de::Visitor<'de> for Visitor {
                            type Value = #name;

                            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                                write!(formatter, "a discriminant followed by a value")
                            }

                            fn visit_seq<_A>(self, mut seq: _A) -> Result<Self::Value, _A::Error>
                            where
                                _A: serde::de::SeqAccess<'de>,
                            {
                                let no_value = ::serde::de::Error::custom("no value found");
                                match seq.next_element::<u8>()? {
                                    #(
                                        Some(#variant_discriminants) => Ok(
                                            seq.next_element::<#variant_types>()?.ok_or(no_value)?.into()
                                        ),
                                    )*
                                    _ => Err(::serde::de::Error::custom("no discriminant found")),
                                }
                            }
                        }
                        deserializer.deserialize_tuple(2, Visitor)
                    }
                }
            )
        } else {
            quote!()
        };

        let derives = quote!();
        let derives = attributes.get("tagged_enum_derives").unwrap_or(&derives);

        let variant_impl = variants
            .iter()
            .zip(variant_types)
            .zip(variant_fields)
            .zip(variant_field_names)
            .zip(variant_attributes)
            .map(|((((variant, variant_type), fields), names), attributes)| {
                let serde = if enable_serde(attributes) {
                    quote!(::serde::Serialize, ::serde::Deserialize,)
                } else {
                    quote!()
                };

                let derives = attributes.get("tagged_enum_derives").unwrap_or(derives);
                match fields {
                    Fields::Named(FieldsNamed {
                        brace_token: _,
                        named: fields,
                    }) => {
                        let fields = fields
                            .iter()
                            .map(|field| {
                                let name = field.ident.as_ref().unwrap();
                                let ty = &field.ty;
                                quote!(#visibility #name: #ty)
                            })
                            .collect::<Vec<_>>();
                        quote!(
                            #[derive(#serde #derives)]
                            #visibility struct #variant_type { #( #fields, )* }

                            impl From<#variant_type> for #name {
                                fn from(value: #variant_type) -> Self {
                                    let #variant_type {#(#names,)*} = value;
                                    Self::#variant {#(#names,)*}
                                }
                            }

                            impl TryFrom<#name> for #variant_type {
                                type Error = ();
                                fn try_from(value: #name) -> Result<Self, Self::Error> {
                                    if let #name::#variant {#(#names,)*} = value {
                                        Ok(#variant_type {#(#names,)*})
                                    } else {
                                        Err(())
                                    }
                                }
                            }
                        )
                    }
                    Fields::Unnamed(FieldsUnnamed {
                        paren_token: _,
                        unnamed: fields,
                    }) => {
                        let fields = fields
                            .iter()
                            .map(|field| {
                                let ty = &field.ty;
                                quote!(#visibility #ty)
                            })
                            .collect::<Vec<_>>();
                        quote!(
                            #[derive(#serde #derives)]
                            #visibility struct #variant_type (#(#fields,)*);

                            impl From<#variant_type> for #name {
                                fn from(value: #variant_type) -> Self {
                                    let #variant_type (#(#names,)*) = value;
                                    Self::#variant (#(#names,)*)
                                }
                            }

                            impl TryFrom<#name> for #variant_type {
                                type Error = ();
                                fn try_from(value: #name) -> Result<Self, Self::Error> {
                                    if let #name::#variant (#(#names,)*) = value {
                                        Ok(#variant_type (#(#names,)*))
                                    } else {
                                        Err(())
                                    }
                                }
                            }
                        )
                    }
                    Fields::Unit => quote!(
                        #[derive(#serde #derives)]
                        #visibility struct #variant_type;

                        impl From<#variant_type> for #name {
                            fn from(_value: #variant_type) -> Self {
                                Self::#variant
                            }
                        }

                        impl TryFrom<#name> for #variant_type {
                            type Error = ();
                            fn try_from(value: #name) -> Result<Self, Self::Error> {
                                if let #name::#variant = value {
                                    Ok(#variant_type)
                                } else {
                                    Err(())
                                }
                            }
                        }
                    ),
                }
            });

        let tag_enum = tag.tag_enum();
        let tag_impl = tag.tag_impl();
        quote!(
            #tag_enum
            #tag_impl
            #(
                #variant_impl
            )*
            #serde

            impl #name {
                #visibility fn #kind_func_name(&self) -> #tag_name {
                    match self {
                        #(
                            Self::#variants #variant_match => #tag_name::#variants,
                        )*
                    }
                }
            }

        )
    }
}

fn attribute_maps(attrs: Vec<Attribute>) -> HashMap<String, proc_macro2::TokenStream> {
    attrs
        .into_iter()
        .filter_map(|attr| {
            let (path, tokens) = match attr.meta {
                Meta::Path(path) => (path, quote!()),
                Meta::List(MetaList {
                    path,
                    delimiter: _,
                    tokens,
                }) => (path, tokens),
                Meta::NameValue(MetaNameValue {
                    path,
                    eq_token: _,
                    value: _value,
                }) => (path, quote!(_value)),
            };
            Some((path.get_ident()?.to_string(), tokens))
        })
        .collect()
}

fn enable_serde(attributes: &HashMap<String, proc_macro2::TokenStream>) -> bool {
    if let Some(tokens) = attributes.get("tagged_enum_serde") {
        let serde_attr = parse2::<Ident>(tokens.clone())
            .expect("could not parse serde tag")
            .to_string()
            .to_lowercase();
        match serde_attr.as_str() {
            "on" | "true" => true,
            "off" | "false" => false,
            _ => panic!("{} is not a valid value for serde_attr", serde_attr),
        }
    } else {
        true
    }
}

#[proc_macro_derive(
    TaggedEnum,
    attributes(tagged_enum_kind, tagged_enum_serde, tagged_enum_derives)
)]
pub fn tagged_enum(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens =
        EnumDescription::new(syn::parse(input).expect("Could not parse enum")).tagged_impl();

    tokens.into()
}
