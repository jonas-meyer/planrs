use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

/// Derive `TypedId` for a newtype wrapping `u64`.
///
/// ```ignore
/// #[derive(AuthoredId, Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
/// pub struct NodeId(pub u64);
/// ```
#[proc_macro_derive(AuthoredId)]
pub fn derive_authored_id(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    quote! {
        impl bevy_author::TypedId for #name {
            fn from_raw(raw: u64) -> Self {
                Self(raw)
            }
            fn to_raw(self) -> u64 {
                self.0
            }
        }
    }
    .into()
}

/// Derive store enumeration helpers for a model struct.
///
/// Scans fields with `Store<_, _>` type and generates:
/// - `any_store_dirty(&self) -> bool`
/// - `mark_all_stores_dirty(&mut self)`
/// - `diff_all_stores(&mut self, other: &Self)`
///
/// Use these in your `AuthoredModel` impl:
///
/// ```ignore
/// #[derive(Resource, Default, Clone, AuthoredStores)]
/// pub struct CityModel {
///     pub nodes: Store<NodeId, Node>,
///     pub edges: Store<EdgeId, Edge>,
/// }
///
/// impl AuthoredModel for CityModel {
///     fn mark_all_dirty(&mut self) { self.mark_all_stores_dirty(); }
///     fn mark_dirty_diff(&mut self, other: &Self) { self.diff_all_stores(other); }
///     // ... cascade_remove, propagate_dirty, validate (user-written)
/// }
/// ```
#[proc_macro_derive(AuthoredStores)]
pub fn derive_authored_stores(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let fields = match &input.data {
        syn::Data::Struct(s) => &s.fields,
        _ => panic!("AuthoredStores can only be derived for structs"),
    };

    let store_fields: Vec<&syn::Ident> = fields
        .iter()
        .filter(|f| is_store_type(&f.ty))
        .filter_map(|f| f.ident.as_ref())
        .collect();

    if store_fields.is_empty() {
        panic!("AuthoredStores: no Store<_, _> fields found");
    }

    let mark_all = store_fields
        .iter()
        .map(|f| quote! { self.#f.mark_all_dirty(); });
    let diff_all = store_fields
        .iter()
        .map(|f| quote! { self.#f.mark_dirty_diff(&other.#f); });
    let any_dirty = store_fields
        .iter()
        .map(|f| quote! { self.#f.is_dirty() });

    quote! {
        impl #name {
            pub fn any_store_dirty(&self) -> bool {
                #(#any_dirty)||*
            }

            pub fn mark_all_stores_dirty(&mut self) {
                #(#mark_all)*
            }

            pub fn diff_all_stores(&mut self, other: &Self) {
                #(#diff_all)*
            }
        }
    }
    .into()
}

fn is_store_type(ty: &syn::Type) -> bool {
    if let syn::Type::Path(tp) = ty {
        tp.path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "Store")
    } else {
        false
    }
}
