use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::parse::Parse;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, Token};

struct DeriveConfig {
    trait_name: Ident,
    _comma: Token![,],
    container_name: Ident,
}

impl Parse for DeriveConfig {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            trait_name: input.parse()?,
            _comma: input.parse()?,
            container_name: input.parse()?,
        })
    }
}

pub struct FieldInfo {
    f: Field,
}

impl FieldInfo {
    pub fn reader_ty(&self) -> Ident {
        let name = self.f.ident.as_ref().unwrap().to_string();
        Ident::new(&format!("{name}Reader"), Span::mixed_site())
    }
}

pub fn derive_embeddable(ts: TokenStream) -> syn::Result<TokenStream> {
    let di = syn::parse2::<DeriveInput>(ts)?;
    let mut config = None;
    for attr in di.attrs {
        if !attr.path().is_ident("flatt") {
            continue;
        }

        let cfg = attr.parse_args::<DeriveConfig>()?;
        config = Some(cfg);
        break;
    }

    let ident = di.ident;

    let Some(DeriveConfig {
        trait_name,
        container_name,
        ..
    }) = config
    else {
        return Err(syn::Error::new_spanned(&ident, "missing `#[flatt]`"));
    };

    match di.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(named),
            ..
        }) => {
            let fi = named
                .named
                .into_iter()
                .map(|f| FieldInfo { f })
                .collect::<Vec<_>>();
            let assoc_tys = fi.iter().map(|fi| {
                let reader_ty = fi.reader_ty();
                let field_ty = &fi.f.ty;
                quote! {
                    type #reader_ty: ::flatt::IsFieldReader<Type = #field_ty>;
                }
            });
            let generics = di.generics;

            let mut container_generics_in_trait = generics.clone();
            container_generics_in_trait.params.insert(
                0,
                syn::GenericParam::Type(syn::TypeParam {
                    attrs: vec![],
                    ident: Ident::new("Self", Span::mixed_site()),
                    colon_token: None,
                    bounds: Default::default(),
                    eq_token: None,
                    default: None,
                }),
            );
            let (_, type_generics, _) = container_generics_in_trait.split_for_impl();
            let trait_def = quote! {
                pub unsafe trait #trait_name #generics {
                    #(#assoc_tys)*
                    fn as_container(&self) -> &#container_name #type_generics where Self: Sized {
                        unsafe { &*(self as *const Self as *const #container_name #type_generics) }
                    }
                    fn as_container_mut(&mut self) -> &mut #container_name #type_generics where Self: Sized {
                        unsafe { &mut *(self as *mut Self as *mut #container_name #type_generics) }
                    }
                }
            };

            let field_readers = fi
                .iter()
                .map(|x| {
                    let vis = &x.f.vis;
                    let ident = x.f.ident.as_ref().unwrap();
                    let ty = x.reader_ty();
                    quote! {
                        #vis #ident: ::flatt::Zstizer<Inner__::#ty>,
                    }
                })
                .collect::<Vec<_>>();

            let (_, type_generics, _) = generics.split_for_impl();
            let mut container_generics = generics.clone();
            container_generics
                .params
                .insert(0, syn::parse_quote!(Inner__: #trait_name #type_generics));

            let (impl_generics, type_generics, where_clause) = container_generics.split_for_impl();

            let container = quote! {
                #[repr(transparent)]
                pub struct #container_name #container_generics #where_clause {
                    #(#field_readers)*
                    inner__: ::flatt::Inaccessible<Inner__>,
                    __phantom: ::core::marker::PhantomData<#ident #generics>
                }

                // prevent fields from being moved out of this container
                impl #impl_generics Drop for #container_name #type_generics #where_clause {
                    fn drop(&mut self) {}
                }
            };

            Ok(quote! {
                #trait_def
                #container
            })
        }
        _ => unimplemented!(),
    }
}
