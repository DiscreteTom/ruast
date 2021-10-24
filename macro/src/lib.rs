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
      fn brx(&self) -> Brx {
        self.btx.subscribe()
      }
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
      fn tx(&self) -> &Tx {
        &self.tx
      }
    }
  };
  gen.into()
}
