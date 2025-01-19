#[proc_macro_derive(Instance, attributes(instance, field))]
pub fn instance_derive(ts: TokenStream) -> TokenStream {
    let origin: syn::DeriveInput = syn::parse_macro_input!(ts.into());
    todo!()
}