use proc_macro2::TokenStream;
use quote::quote;
use syn::Ident;

/// Generates code to compute a polymorphic variant tag hash value
pub fn generate_polytag_hash(variant_ident: &Ident, tag_str: Option<&String>) -> TokenStream {
    let string_to_hash_with_null = match tag_str {
        Some(tag_str) => format!("{tag_str}\0"),
        None => format!("{variant_ident}\0"),
    };
    let static_tag_hash_ident = Ident::new("TAG_HASH", variant_ident.span());
    let static_once_ident = Ident::new("INIT_TAG_HASH", variant_ident.span());

    quote! {{
        static mut #static_tag_hash_ident: ::ocaml_interop::RawOCaml = 0;
        static #static_once_ident: ::std::sync::Once = ::std::sync::Once::new();
        unsafe {
            #static_once_ident.call_once(|| {
                #static_tag_hash_ident = ::ocaml_interop::internal::caml_hash_variant(#string_to_hash_with_null.as_ptr());
            });
            #static_tag_hash_ident
        }
    }}
}
