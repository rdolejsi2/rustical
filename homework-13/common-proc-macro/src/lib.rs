use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

/// A custom derive macro that generates a method to get the variant name of an enum.
/// Surprisingly enough, this is not part of the standard library and any enum
/// operated by Rust always represents itself as its base name.
///
/// This is pretty unusable in case a dynamic lookup of data associated with each
/// enum variant is needed. In this project we use different functions within a map
/// to handle all commands, read their help and other things related to a specific
/// enum variant.
///
/// This macro came to life after a 3-hour tinkering with the Rust runtime and almost
/// hopeless research into the Rust standard library. It is a bit of a hack, but it works.
/// Given Rust seems to not support the dynamic approach out of the box, we might lose
/// the ability and rewrite the processing to a more static approach with matches everywhere,
/// which seems to be the way Rust is designed to be used.
///
/// But for now, we keep it here for posterity and as a reference to how the proc-macro
/// can be used. Btw. it looks like something serde_json uses as well, to be able
/// to deserialize the enum variants by their names (serde.tag annotation).
#[proc_macro_derive(EnumVariantName)]
pub fn enum_variant_name_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let variants = if let Data::Enum(data_enum) = &input.data {
        data_enum
            .variants
            .iter()
            .map(|v| &v.ident)
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    let gen = quote! {
        impl #name {
            pub fn variant_name(&self) -> &'static str {
                match self {
                    #(Self::#variants {..} => stringify!(#variants),)*
                }
            }
        }
    };

    gen.into()
}
