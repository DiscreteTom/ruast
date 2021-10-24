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

    impl #name {
      /// self.btx => other.tx
      pub fn publish(self, other: &impl WriterNode) -> Self {
        let mut brx = self.brx();
        let tx = other.tx().clone();

        tokio::spawn(async move {
          loop {
            match brx.recv().await {
              Ok(e) => {
                if tx.send(e).await.is_err() {
                  break;
                }
              }
              Err(_) => break,
            }
          }
        });
        self
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

    impl #name {
      /// other.btx => self.tx
      pub fn subscribe(self, other: &impl ReaderNode) -> Self {
        let mut brx = other.brx();
        let tx = self.tx().clone();

        tokio::spawn(async move {
          loop {
            match brx.recv().await {
              Ok(e) => {
                if tx.send(e).await.is_err() {
                  break;
                }
              }
              Err(_) => break,
            }
          }
        });
        self
      }
    }
  };
  gen.into()
}
