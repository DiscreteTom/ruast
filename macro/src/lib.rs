extern crate proc_macro;

use crate::proc_macro::TokenStream;
use quote::quote;
use syn::{self, DeriveInput};

#[proc_macro_derive(Writable)]
pub fn writable_derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse(input).unwrap();
  let name = &ast.ident;

  let gen = quote! {
    impl Writable for #name {
      fn write(&self, data: Bytes) -> Result<()> {
        let tx = self.tx.clone();
        if tx.is_closed() {
          return Err(Box::new(Error::WriteToClosedChannel));
        }
        tokio::spawn(async move { tx.send(data).await });
        Ok(())
      }
    }
  };
  gen.into()
}

#[proc_macro_derive(Stoppable)]
pub fn stoppable_derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse(input).unwrap();
  let name = &ast.ident;

  let gen = quote! {
    impl Stoppable for #name {
      fn stop(&self) {
        let stop_tx = self.stop_tx.clone();
        tokio::spawn(async move { stop_tx.send(()).await });
      }
    }
  };
  gen.into()
}

#[proc_macro_derive(WritableStoppable)]
pub fn writable_stoppable_derive(input: TokenStream) -> TokenStream {
  let ast: DeriveInput = syn::parse(input).unwrap();
  let name = &ast.ident;

  let gen = quote! {
    impl WritableStoppable for #name {}
  };
  gen.into()
}
