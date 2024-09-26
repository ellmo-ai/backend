extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

#[proc_macro_derive(Insertable)]
pub fn derive_insertable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let struct_name = input.ident;

    // Generate the name for the insertable struct
    let insertable_struct_name =
        syn::Ident::new(&format!("Insertable{}", struct_name), struct_name.span());

    // Match on the struct fields and remove the 'id' field if present
    let fields = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Named(ref fields_named) => {
                let fields_without_id: Vec<_> = fields_named
                    .named
                    .iter()
                    .filter(|f| f.ident.as_ref().map(|ident| ident != "id").unwrap_or(true))
                    .collect();
                fields_without_id
            }
            _ => panic!("Expected a struct with named fields"),
        },
        _ => panic!("Insertable macro only works on structs"),
    };

    // Generate the insertable struct definition
    let expanded = quote! {
        #[derive(Insertable, Selectable)]
        #[diesel(table_name = crate::schema::eval)]
        #[diesel(check_for_backend(diesel::pg::Pg))]
        pub struct #insertable_struct_name {
            #(#fields),*
        }

        impl From<#struct_name> for #insertable_struct_name {
            fn from(e: #struct_name) -> Self {
                #insertable_struct_name {
                    #(#fields: e.#fields),*
                }
            }
        }
    };

    TokenStream::from(expanded)
}
