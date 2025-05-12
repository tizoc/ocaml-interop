use proc_macro::TokenStream;
use syn::ItemFn;

mod core;
mod expansion;
mod parsing;

fn export_internal_logic(
    attr_ts: proc_macro2::TokenStream,
    item_ts: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let input_fn = syn::parse2::<ItemFn>(item_ts.clone()).map_err(|e| {
        syn::Error::new(
            e.span(),
            format!("Failed to parse input item as a function: {e}. Input was: {item_ts}",),
        )
    })?;

    let parsed_data = parsing::parse_export_definition(attr_ts, &input_fn)?;

    expansion::expand_function_from_data(&parsed_data)
}

// --- Proc Macro Entry Point ---
#[proc_macro_attribute]
pub fn export(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_ts2 = proc_macro2::TokenStream::from(attr);
    let item_ts2 = proc_macro2::TokenStream::from(item);

    match export_internal_logic(attr_ts2, item_ts2) {
        Ok(generated_code) => TokenStream::from(generated_code),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

#[cfg(test)]
mod tests;
