extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{self, DeriveInput};

#[proc_macro_derive(ReaderNode)]
pub fn reader_node_derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse(input).unwrap();
  let name = &ast.ident;

  let gen = quote! {
    impl ReaderNode for #name {
      impl_node!(brx);
    }

    impl #name {
      impl_node!(publish);
    }
  };
  gen.into()
}

#[proc_macro_derive(WriterNode)]
pub fn writer_node_derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse(input).unwrap();
  let name = &ast.ident;

  let gen = quote! {
    impl WriterNode for #name {
      impl_node!(tx);
    }

    impl #name {
      impl_node!(subscribe);
    }
  };
  gen.into()
}
