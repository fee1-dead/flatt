#![feature(proc_macro_expand)]
mod embeddable;

#[proc_macro_derive(Embeddable, attributes(flatt))]
pub fn derive_embeddable(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    embeddable::derive_embeddable(ts.into())
        .unwrap_or_else(|e| e.into_compile_error())
        .into()
}

#[proc_macro]
pub fn try_expand(ts: proc_macro::TokenStream) -> proc_macro::TokenStream {
    panic!("{:?}", ts.expand_expr())
}
