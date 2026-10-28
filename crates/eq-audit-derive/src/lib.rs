//! Derive support for safe audit snapshots (AUD-02, SBD-21).

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Index, parse_macro_input, parse_quote};

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldMode {
    Include,
    Redact,
    Skip,
}

/// Derives `eq_audit::Auditable` for a struct with optional redacted or skipped fields.
#[proc_macro_derive(Auditable, attributes(audit))]
pub fn derive_auditable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand(input) {
        Ok(tokens) => tokens.into(),
        Err(error) => error.to_compile_error().into(),
    }
}

fn expand(input: DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = input.ident;
    let fields = match input.data {
        Data::Struct(data) => data.fields,
        Data::Enum(_) | Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "Auditable can only be derived for structs",
            ));
        }
    };

    let mut generics = input.generics;
    let mut inserts = Vec::new();
    let where_clause = generics.make_where_clause();

    for (position, field) in fields.iter().enumerate() {
        let mode = field_mode(field)?;
        if mode == FieldMode::Skip {
            continue;
        }

        let key = field
            .ident
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| position.to_string());
        let accessor = field.ident.as_ref().map_or_else(
            || {
                let index = Index::from(position);
                quote!(#index)
            },
            |ident| quote!(#ident),
        );

        if mode == FieldMode::Redact {
            inserts.push(quote! {
                object.insert(#key.to_owned(), ::serde_json::Value::String("[redacted]".to_owned()));
            });
        } else {
            let field_type = &field.ty;
            where_clause.predicates.push(parse_quote!(
                #field_type: ::eq_audit::__private::serde::Serialize
            ));
            inserts.push(quote! {
                let value = ::eq_audit::__private::serde_json::to_value(&self.#accessor)
                    .unwrap_or_else(|_| ::eq_audit::__private::serde_json::Value::Null);
                object.insert(#key.to_owned(), value);
            });
        }
    }

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();
    Ok(quote! {
        impl #impl_generics ::eq_audit::Auditable for #name #type_generics #where_clause {
            fn audit_snapshot(&self) -> ::eq_audit::__private::serde_json::Value {
                let mut object = ::eq_audit::__private::serde_json::Map::new();
                #(#inserts)*
                ::eq_audit::__private::serde_json::Value::Object(object)
            }
        }
    })
}

fn field_mode(field: &syn::Field) -> syn::Result<FieldMode> {
    let mut mode = FieldMode::Include;
    let mut has_attribute = false;
    for attribute in field
        .attrs
        .iter()
        .filter(|attribute| attribute.path().is_ident("audit"))
    {
        has_attribute = true;
        attribute.parse_nested_meta(|meta| {
            let Some(ident) = meta.path.get_ident() else {
                return Err(meta.error("expected `redact` or `skip`"));
            };
            let next_mode = if ident == "redact" {
                FieldMode::Redact
            } else if ident == "skip" {
                FieldMode::Skip
            } else {
                return Err(meta.error("expected `redact` or `skip`"));
            };
            if mode != FieldMode::Include {
                return Err(meta.error("only one audit field modifier is allowed"));
            }
            mode = next_mode;
            Ok(())
        })?;
    }
    if has_attribute && mode == FieldMode::Include {
        return Err(syn::Error::new(Span::call_site(), "expected `redact` or `skip`"));
    }
    Ok(mode)
}
